use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{err, Result};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Host {
    Local {
        url: Option<String>,
        port: Option<u16>,
    },
    Caddy {
        url: Option<String>,
    },
    TailscaleServe {
        url: Option<String>,
        path: Option<String>,
        background: Option<bool>,
    },
    TailscaleFunnel {
        url: Option<String>,
        path: Option<String>,
        background: Option<bool>,
    },
    CloudflarePages {
        url: Option<String>,
        project: Option<String>,
        #[serde(rename = "accountId", alias = "account_id")]
        account_id: Option<String>,
        #[serde(rename = "tokenEnv", alias = "token_env")]
        token_env: Option<String>,
        #[serde(rename = "tokenRef", alias = "token_ref")]
        token_ref: Option<String>,
        #[serde(rename = "tokenCommand", alias = "token_command")]
        token_command: Option<String>,
        branch: Option<String>,
    },
    Command {
        url: Option<String>,
        deploy: Option<String>,
        #[serde(rename = "afterAdd", alias = "after_add")]
        after_add: Option<String>,
        open: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bag {
    pub label: String,
    pub dir: PathBuf,
    pub host: Host,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub default_bag: String,
    pub bags: Vec<Bag>,
    pub config_path: PathBuf,
    pub config_loaded: bool,
    pub data_dir: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
struct RawConfig {
    #[serde(rename = "defaultBag", alias = "default_bag")]
    default_bag: Option<String>,
    bags: Option<BTreeMap<String, RawBag>>,
}

#[derive(Clone, Debug, Deserialize)]
struct RawBag {
    dir: Option<String>,
    url: Option<String>,
    host: Option<Host>,
}

impl Config {
    pub fn load() -> Result<Self> {
        load_env_file()?;

        let data_dir = default_data_dir();
        let config_path = default_config_path();
        let raw = read_user_config(&config_path)?;
        let config_loaded = raw.is_some();
        let mut bags = built_in_bags(&data_dir);
        add_legacy_bags(&mut bags);

        if let Some(raw) = &raw {
            merge_user_config(&mut bags, raw)?;
        }

        let default_bag = env::var("WEBBY_DEFAULT_BAG")
            .ok()
            .or_else(|| raw.as_ref().and_then(|r| r.default_bag.clone()))
            .unwrap_or_else(|| "local".to_string());

        Ok(Self {
            default_bag,
            bags,
            config_path,
            config_loaded,
            data_dir,
        })
    }

    pub fn bag(&self, label: &str) -> Result<&Bag> {
        self.bags.iter().find(|b| b.label == label).ok_or_else(|| {
            err(format!(
                "unknown bag '{label}' (have: {})",
                self.labels().join(", ")
            ))
        })
    }

    pub fn default_bag(&self) -> Result<&Bag> {
        self.bag(&self.default_bag)
    }

    pub fn labels(&self) -> Vec<String> {
        self.bags.iter().map(|b| b.label.clone()).collect()
    }
}

pub fn default_config_path() -> PathBuf {
    env::var("WEBBY_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| config_home().join("webby").join("config.json"))
}

pub fn default_data_dir() -> PathBuf {
    env::var("WEBBY_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| data_home().join("webby"))
}

pub fn sample_config() -> String {
    let sample = serde_json::json!({
        "defaultBag": "local",
        "bags": {
            "local": {
                "dir": "~/.local/share/webby/local",
                "host": { "type": "local", "port": 8765 }
            },
            "tailnet": {
                "dir": "~/.local/share/webby/tailnet",
                "host": { "type": "tailscale-serve", "path": "/", "background": true }
            },
            "funnel": {
                "dir": "~/.local/share/webby/funnel",
                "host": { "type": "tailscale-funnel", "path": "/", "background": true }
            },
            "public": {
                "dir": "~/.local/share/webby/public",
                "host": {
                    "type": "cloudflare-pages",
                    "project": "webby",
                    "tokenEnv": "CLOUDFLARE_API_TOKEN"
                }
            }
        }
    });
    format!(
        "{}\n",
        serde_json::to_string_pretty(&sample).expect("sample config")
    )
}

pub fn expand_path(path: impl AsRef<str>) -> PathBuf {
    let path = path.as_ref();
    let home = home_dir();
    if path == "~" {
        return home;
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return home.join(rest);
    }
    PathBuf::from(path.replace("$HOME", &home.to_string_lossy()))
}

fn read_user_config(path: &Path) -> Result<Option<RawConfig>> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path)
        .map_err(|e| err(format!("failed to read {}: {e}", path.display())))?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|e| err(format!("failed to parse {}: {e}", path.display())))
}

fn built_in_bags(data_dir: &Path) -> Vec<Bag> {
    vec![
        bag(
            "local",
            data_dir.join("local"),
            Host::Local {
                url: env::var("WEBBY_LOCAL_URL").ok(),
                port: env::var("WEBBY_LOCAL_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .or(Some(8765)),
            },
        ),
        bag(
            "tailnet",
            data_dir.join("tailnet"),
            Host::TailscaleServe {
                url: env::var("TAILSCALE_URL").ok(),
                path: env::var("TAILSCALE_PATH").ok().or(Some("/".to_string())),
                background: Some(true),
            },
        ),
        bag(
            "funnel",
            data_dir.join("funnel"),
            Host::TailscaleFunnel {
                url: env::var("FUNNEL_URL").ok(),
                path: env::var("FUNNEL_PATH").ok().or(Some("/".to_string())),
                background: Some(true),
            },
        ),
        bag(
            "public",
            env::var("PUBLIC_DIR")
                .map(expand_path)
                .unwrap_or_else(|_| data_dir.join("public")),
            Host::CloudflarePages {
                url: env::var("PUBLIC_URL").ok(),
                project: env::var("PUBLIC_PROJECT")
                    .ok()
                    .or(Some("webby".to_string())),
                account_id: env::var("CLOUDFLARE_ACCOUNT_ID")
                    .or_else(|_| env::var("CF_ACCOUNT_ID"))
                    .ok(),
                token_env: env::var("CF_TOKEN_ENV")
                    .ok()
                    .or(Some("CLOUDFLARE_API_TOKEN".to_string())),
                token_ref: env::var("CF_TOKEN_REF").ok(),
                token_command: env::var("CF_TOKEN_COMMAND").ok(),
                branch: Some("main".to_string()),
            },
        ),
    ]
}

fn add_legacy_bags(bags: &mut Vec<Bag>) {
    if env::var("INTERNAL_URL").is_ok() || env::var("INTERNAL_DIR").is_ok() {
        upsert_bag(
            bags,
            bag(
                "internal",
                env::var("INTERNAL_DIR")
                    .map(expand_path)
                    .unwrap_or_else(|_| default_data_dir().join("internal")),
                Host::Caddy {
                    url: env::var("INTERNAL_URL").ok(),
                },
            ),
        );
    }
}

fn merge_user_config(bags: &mut Vec<Bag>, raw: &RawConfig) -> Result<()> {
    let Some(raw_bags) = &raw.bags else {
        return Ok(());
    };

    for (label, raw_bag) in raw_bags {
        let existing = bags.iter().find(|b| &b.label == label);
        let dir = raw_bag
            .dir
            .as_deref()
            .map(expand_path)
            .or_else(|| existing.map(|b| b.dir.clone()))
            .unwrap_or_else(|| default_data_dir().join(label));
        let mut host = raw_bag
            .host
            .clone()
            .or_else(|| existing.map(|b| b.host.clone()))
            .unwrap_or(Host::Local {
                url: None,
                port: Some(8765),
            });
        if let Some(url) = &raw_bag.url {
            set_host_url(&mut host, url.clone());
        }
        upsert_bag(bags, bag(label, dir, host));
    }
    Ok(())
}

fn set_host_url(host: &mut Host, url: String) {
    match host {
        Host::Local { url: u, .. }
        | Host::Caddy { url: u }
        | Host::TailscaleServe { url: u, .. }
        | Host::TailscaleFunnel { url: u, .. }
        | Host::CloudflarePages { url: u, .. }
        | Host::Command { url: u, .. } => *u = Some(url),
    }
}

fn upsert_bag(bags: &mut Vec<Bag>, next: Bag) {
    if let Some(existing) = bags.iter_mut().find(|b| b.label == next.label) {
        *existing = next;
    } else {
        bags.push(next);
    }
}

fn bag(label: impl Into<String>, dir: PathBuf, host: Host) -> Bag {
    Bag {
        label: label.into(),
        dir,
        host,
    }
}

fn load_env_file() -> Result<()> {
    let file = match env::var("WEBBY_ENV") {
        Ok(path) => {
            let path = PathBuf::from(path);
            if path.exists() {
                Some(path)
            } else {
                None
            }
        }
        Err(_) => {
            let local = PathBuf::from(".env.secret");
            if local.exists() {
                Some(local)
            } else {
                None
            }
        }
    };
    let Some(file) = file else {
        return Ok(());
    };

    let text = fs::read_to_string(&file)
        .map_err(|e| err(format!("failed to read {}: {e}", file.display())))?;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if env::var_os(key.trim()).is_some() {
            continue;
        }
        let mut value = value.trim().to_string();
        if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            value = value[1..value.len() - 1].to_string();
        }
        env::set_var(key.trim(), value);
    }
    Ok(())
}

fn home_dir() -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn config_home() -> PathBuf {
    env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".config"))
}

fn data_home() -> PathBuf {
    env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".local").join("share"))
}
