mod app;
mod cards;
mod config;
mod metadata;
mod preview;
mod providers;
mod render;

use clap::{CommandFactory, Parser, Subcommand};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;

use crate::app::{app_url, generate_index, list_apps, remove_app, stage_app};
use crate::config::{Config, Host, sample_config};
use crate::metadata::{MetadataOverrides, apply_app_metadata_overrides};
use crate::preview::capture_previews;
use crate::providers::{after_add, attach_domain, base_url, deploy_bag, open_app};

pub type Result<T> = std::result::Result<T, WebbyError>;

#[derive(Debug)]
pub struct WebbyError(String);

impl fmt::Display for WebbyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for WebbyError {}

impl From<std::io::Error> for WebbyError {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}

pub fn err(message: impl Into<String>) -> WebbyError {
    WebbyError(message.into())
}

#[derive(Parser)]
#[command(name = "webby")]
#[command(about = "Drop a static HTML app into a local, tailnet, or public URL.")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Stage a .html file or folder with index.html into a bag.
    Add {
        path: PathBuf,
        #[arg(short = 'b', long = "bag")]
        bag: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        tmp: bool,
        /// Set the generated card title in the staged app metadata.
        #[arg(long)]
        title: Option<String>,
        /// Set the generated card description in the staged app metadata.
        #[arg(long)]
        description: Option<String>,
        /// Set an app metadata property as KEY=VALUE. Can be repeated.
        #[arg(long = "property", value_name = "KEY=VALUE")]
        properties: Vec<String>,
        /// Do not write the bag root index.html for this generation.
        #[arg(long)]
        no_index: bool,
    },
    /// Stage into the public bag and deploy it.
    Pub {
        path: PathBuf,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        tmp: bool,
        /// Set the generated card title in the staged app metadata.
        #[arg(long)]
        title: Option<String>,
        /// Set the generated card description in the staged app metadata.
        #[arg(long)]
        description: Option<String>,
        /// Set an app metadata property as KEY=VALUE. Can be repeated.
        #[arg(long = "property", value_name = "KEY=VALUE")]
        properties: Vec<String>,
        /// Do not write the public bag root index.html for this publish.
        #[arg(long)]
        no_index: bool,
    },
    /// Deploy or activate a bag.
    Deploy {
        #[arg(short = 'b', long = "bag")]
        bag: String,
        #[arg(long)]
        port: Option<u16>,
        /// Do not write the bag root index.html for this deploy.
        #[arg(long)]
        no_index: bool,
    },
    /// Serve a bag on localhost.
    Serve {
        #[arg(short = 'b', long = "bag")]
        bag: Option<String>,
        #[arg(long)]
        port: Option<u16>,
        /// Do not write the bag root index.html before serving.
        #[arg(long)]
        no_index: bool,
    },
    /// List all bags, or one selected bag.
    Ls {
        #[arg(short = 'b', long = "bag")]
        bag: Option<String>,
    },
    /// Remove an app from a bag.
    Rm {
        name: String,
        #[arg(short = 'b', long = "bag")]
        bag: Option<String>,
    },
    /// Print and open an app URL.
    Open {
        name: String,
        #[arg(short = 'b', long = "bag")]
        bag: Option<String>,
    },
    /// Attach a custom domain to a Cloudflare Pages bag.
    Domain {
        hostname: String,
        #[arg(short = 'b', long = "bag")]
        bag: String,
    },
    /// Print configured bags and paths.
    Where,
    /// Write a starter config file.
    Init {
        #[arg(long)]
        force: bool,
    },
    /// Capture static screenshot previews for a bag.
    Preview {
        /// Optional app name to preview instead of the whole bag.
        app: Option<String>,
        #[arg(short = 'b', long = "bag")]
        bag: Option<String>,
        #[arg(long)]
        force: bool,
        #[arg(long, default_value_t = 1200)]
        width: u32,
        #[arg(long, default_value_t = 750)]
        height: u32,
        #[arg(long, default_value_t = 8)]
        timeout_secs: u64,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("✗ {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let Some(command) = cli.command else {
        Cli::command().print_help().map_err(WebbyError::from)?;
        println!();
        return Ok(());
    };

    match command {
        Command::Init { force } => cmd_init(force),
        Command::Where => cmd_where(&Config::load()?),
        Command::Ls { bag } => cmd_ls(&Config::load()?, bag.as_deref()),
        Command::Add {
            path,
            bag,
            name,
            tmp,
            title,
            description,
            properties,
            no_index,
        } => cmd_add(
            &Config::load()?,
            path,
            bag.as_deref(),
            name.as_deref(),
            tmp,
            metadata_overrides(title, description, properties)?,
            no_index,
        ),
        Command::Pub {
            path,
            name,
            tmp,
            title,
            description,
            properties,
            no_index,
        } => cmd_pub(
            &Config::load()?,
            path,
            name.as_deref(),
            tmp,
            metadata_overrides(title, description, properties)?,
            no_index,
        ),
        Command::Deploy {
            bag,
            port,
            no_index,
        } => {
            let cfg = Config::load()?;
            let bag = with_no_index(cfg.bag(&bag)?, no_index);
            deploy_bag(&bag, port)
        }
        Command::Serve {
            bag,
            port,
            no_index,
        } => cmd_serve(&Config::load()?, bag.as_deref(), port, no_index),
        Command::Rm { name, bag } => cmd_rm(&Config::load()?, &name, bag.as_deref()),
        Command::Open { name, bag } => cmd_open(&Config::load()?, &name, bag.as_deref()),
        Command::Domain { hostname, bag } => {
            let cfg = Config::load()?;
            attach_domain(cfg.bag(&bag)?, &hostname)
        }
        Command::Preview {
            app,
            bag,
            force,
            width,
            height,
            timeout_secs,
        } => {
            let cfg = Config::load()?;
            let bag = select_bag(&cfg, bag.as_deref())?;
            capture_previews(
                bag,
                force,
                width,
                height,
                std::time::Duration::from_secs(timeout_secs),
                app.as_deref(),
            )
        }
    }
}

fn cmd_add(
    cfg: &Config,
    path: PathBuf,
    bag: Option<&str>,
    name: Option<&str>,
    tmp: bool,
    metadata: MetadataOverrides,
    no_index: bool,
) -> Result<()> {
    let bag = with_no_index(select_bag(cfg, bag)?, no_index);
    let staged = stage_app(&path, &bag, name, tmp)?;
    apply_app_metadata_overrides(
        &staged_app_path(&bag, &staged.name, staged.is_dir),
        staged.is_dir,
        &metadata,
    )?;
    generate_index(&bag)?;
    after_add(&bag)?;

    println!("✓ {} → {} bag", staged.name, bag.label);
    match &bag.host {
        Host::CloudflarePages { .. } => {
            println!(
                "  staged: {}  (run `webby deploy -b {}` to publish)",
                app_url(&base_url(&bag, None), &staged.name, staged.is_dir),
                bag.label
            );
        }
        Host::Local { .. } => {
            println!(
                "  staged: {}  (run `webby serve -b {}` for {})",
                staged.relative_path,
                bag.label,
                base_url(&bag, None)
            );
        }
        Host::TailscaleServe { .. } | Host::TailscaleFunnel { .. } => {
            println!(
                "  url: {}",
                app_url(&base_url(&bag, None), &staged.name, staged.is_dir)
            );
            println!(
                "  run `webby deploy -b {}` to activate {}",
                bag.label,
                host_name(&bag.host)
            );
        }
        _ => println!(
            "  url: {}",
            app_url(&base_url(&bag, None), &staged.name, staged.is_dir)
        ),
    }
    Ok(())
}

fn cmd_pub(
    cfg: &Config,
    path: PathBuf,
    name: Option<&str>,
    tmp: bool,
    metadata: MetadataOverrides,
    no_index: bool,
) -> Result<()> {
    let bag = with_no_index(cfg.bag("public")?, no_index);
    let staged = stage_app(&path, &bag, name, tmp)?;
    apply_app_metadata_overrides(
        &staged_app_path(&bag, &staged.name, staged.is_dir),
        staged.is_dir,
        &metadata,
    )?;
    println!("✓ {} → public bag", staged.name);
    deploy_bag(&bag, None)
}

fn staged_app_path(bag: &config::Bag, name: &str, is_dir: bool) -> PathBuf {
    if is_dir {
        bag.dir.join(name)
    } else {
        bag.dir.join(format!("{name}.html"))
    }
}

fn metadata_overrides(
    title: Option<String>,
    description: Option<String>,
    properties: Vec<String>,
) -> Result<MetadataOverrides> {
    let mut overrides = MetadataOverrides {
        title,
        description,
        ..MetadataOverrides::default()
    };
    for property in properties {
        let Some((key, value)) = property.split_once('=') else {
            return Err(err(format!(
                "invalid --property '{property}' (expected KEY=VALUE)"
            )));
        };
        let key = key.trim();
        if key.is_empty() {
            return Err(err(format!(
                "invalid --property '{property}' (key must not be empty)"
            )));
        }
        overrides.properties.insert(
            key.to_string(),
            serde_json::Value::String(value.trim().to_string()),
        );
    }
    Ok(overrides)
}

fn cmd_ls(cfg: &Config, bag: Option<&str>) -> Result<()> {
    if let Some(label) = bag {
        return print_bag_apps(cfg.bag(label)?);
    }
    for (idx, bag) in cfg.bags.iter().enumerate() {
        if idx > 0 {
            println!();
        }
        print_bag_apps(bag)?;
    }
    Ok(())
}

fn print_bag_apps(bag: &config::Bag) -> Result<()> {
    let apps = list_apps(bag)?;
    if apps.is_empty() {
        println!("({} bag is empty)", bag.label);
        return Ok(());
    }
    println!("{} bag — {}", bag.label, base_url(bag, None));
    for app in apps {
        println!(
            "  {}{}{}",
            app.name,
            if app.is_dir { "/" } else { ".html" },
            if app.tmp { " ·tmp" } else { "" }
        );
    }
    Ok(())
}

fn cmd_rm(cfg: &Config, name: &str, bag: Option<&str>) -> Result<()> {
    let bag = select_bag(cfg, bag)?;
    remove_app(bag, name)?;
    generate_index(&bag)?;
    println!("✓ removed {name} from {} bag", bag.label);
    if matches!(bag.host, Host::CloudflarePages { .. }) {
        println!(
            "  run `webby deploy -b {}` to update the live site",
            bag.label
        );
    }
    Ok(())
}

fn cmd_open(cfg: &Config, name: &str, bag: Option<&str>) -> Result<()> {
    let bag = select_bag(cfg, bag)?;
    let clean = name.trim_end_matches('/').trim_end_matches(".html");
    let is_dir = list_apps(bag)?
        .iter()
        .find(|app| app.name == clean)
        .map(|app| app.is_dir)
        .unwrap_or_else(|| !name.ends_with(".html"));
    let url = app_url(&base_url(bag, None), clean, is_dir);
    println!("{url}");
    open_app(&url);
    Ok(())
}

fn cmd_where(cfg: &Config) -> Result<()> {
    println!("webby bags — drop an app into one of these dirs:\n");
    for bag in &cfg.bags {
        println!("  {:<9} {}", bag.label, bag.dir.display());
        println!(
            "            → {} ({})",
            base_url(bag, None),
            host_name(&bag.host)
        );
    }
    println!("\n  default bag: {}", cfg.default_bag);
    println!("  data dir: {}", cfg.data_dir.display());
    println!(
        "  config: {}{}",
        cfg.config_path.display(),
        if cfg.config_loaded {
            ""
        } else {
            " (not created)"
        }
    );
    Ok(())
}

fn cmd_init(force: bool) -> Result<()> {
    let path = config::default_config_path();
    if path.exists() && !force {
        return Err(err(format!(
            "config already exists: {} (use --force to overwrite)",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, sample_config())?;
    println!("✓ wrote {}", path.display());
    println!("  start with: webby add ./app.html && webby serve");
    Ok(())
}

fn cmd_serve(cfg: &Config, bag: Option<&str>, port: Option<u16>, no_index: bool) -> Result<()> {
    let bag = with_no_index(select_bag(cfg, bag)?, no_index);
    let port = port
        .or_else(|| match bag.host {
            Host::Local { port, .. } => port,
            _ => None,
        })
        .unwrap_or(8765);
    generate_index(&bag)?;
    println!("webby {} — {}", bag.label, bag.dir.display());
    println!("serving: http://localhost:{port}");
    println!("Press Ctrl+C to stop.");
    serve_dir(&bag.dir, port)
}

fn serve_dir(root: &std::path::Path, port: u16) -> Result<()> {
    let server = tiny_http::Server::http(("127.0.0.1", port))
        .map_err(|e| err(format!("failed to bind localhost:{port}: {e}")))?;
    for request in server.incoming_requests() {
        let path = request.url().split('?').next().unwrap_or("/");
        let relative = path.trim_start_matches('/');
        if relative.split('/').any(|part| part == "..") {
            let _ = request
                .respond(tiny_http::Response::from_string("Forbidden\n").with_status_code(403));
            continue;
        }
        let mut target = root.join(relative);
        if target.is_dir() {
            target = target.join("index.html");
        }
        if !target.exists() {
            let _ = request
                .respond(tiny_http::Response::from_string("Not found\n").with_status_code(404));
            continue;
        }
        let file = fs::File::open(&target)?;
        let _ = request.respond(tiny_http::Response::from_file(file));
    }
    Ok(())
}

fn select_bag<'a>(cfg: &'a Config, label: Option<&str>) -> Result<&'a config::Bag> {
    match label {
        Some(label) => cfg.bag(label),
        None => cfg.default_bag(),
    }
}

fn with_no_index(bag: &config::Bag, no_index: bool) -> std::borrow::Cow<'_, config::Bag> {
    if no_index && !bag.no_index {
        let mut bag = bag.clone();
        bag.no_index = true;
        std::borrow::Cow::Owned(bag)
    } else {
        std::borrow::Cow::Borrowed(bag)
    }
}

fn host_name(host: &Host) -> &'static str {
    match host {
        Host::Local { .. } => "local",
        Host::Caddy { .. } => "caddy",
        Host::TailscaleServe { .. } => "tailscale-serve",
        Host::TailscaleFunnel { .. } => "tailscale-funnel",
        Host::CloudflarePages { .. } => "cloudflare-pages",
        Host::Command { .. } => "command",
    }
}
