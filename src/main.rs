mod app;
mod config;
mod providers;
mod render;

use clap::{CommandFactory, Parser, Subcommand};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;

use crate::app::{app_url, generate_index, list_apps, remove_app, stage_app};
use crate::config::{sample_config, Config, Host};
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
    },
    /// Stage into the public bag and deploy it.
    Pub {
        path: PathBuf,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        tmp: bool,
    },
    /// Deploy or activate a bag.
    Deploy {
        #[arg(short = 'b', long = "bag")]
        bag: String,
        #[arg(long)]
        port: Option<u16>,
    },
    /// Serve a bag on localhost.
    Serve {
        #[arg(short = 'b', long = "bag")]
        bag: Option<String>,
        #[arg(long)]
        port: Option<u16>,
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
        } => cmd_add(&Config::load()?, path, bag.as_deref(), name.as_deref(), tmp),
        Command::Pub { path, name, tmp } => cmd_pub(&Config::load()?, path, name.as_deref(), tmp),
        Command::Deploy { bag, port } => {
            let cfg = Config::load()?;
            deploy_bag(cfg.bag(&bag)?, port)
        }
        Command::Serve { bag, port } => cmd_serve(&Config::load()?, bag.as_deref(), port),
        Command::Rm { name, bag } => cmd_rm(&Config::load()?, &name, bag.as_deref()),
        Command::Open { name, bag } => cmd_open(&Config::load()?, &name, bag.as_deref()),
        Command::Domain { hostname, bag } => {
            let cfg = Config::load()?;
            attach_domain(cfg.bag(&bag)?, &hostname)
        }
    }
}

fn cmd_add(
    cfg: &Config,
    path: PathBuf,
    bag: Option<&str>,
    name: Option<&str>,
    tmp: bool,
) -> Result<()> {
    let bag = select_bag(cfg, bag)?;
    let staged = stage_app(&path, bag, name, tmp)?;
    generate_index(bag)?;
    after_add(bag)?;

    println!("✓ {} → {} bag", staged.name, bag.label);
    match &bag.host {
        Host::CloudflarePages { .. } => {
            println!(
                "  staged: {}  (run `webby deploy -b {}` to publish)",
                app_url(&base_url(bag, None), &staged.name, staged.is_dir),
                bag.label
            );
        }
        Host::Local { .. } => {
            println!(
                "  staged: {}  (run `webby serve -b {}` for {})",
                staged.relative_path,
                bag.label,
                base_url(bag, None)
            );
        }
        Host::TailscaleServe { .. } | Host::TailscaleFunnel { .. } => {
            println!(
                "  url: {}",
                app_url(&base_url(bag, None), &staged.name, staged.is_dir)
            );
            println!(
                "  run `webby deploy -b {}` to activate {}",
                bag.label,
                host_name(&bag.host)
            );
        }
        _ => println!(
            "  url: {}",
            app_url(&base_url(bag, None), &staged.name, staged.is_dir)
        ),
    }
    Ok(())
}

fn cmd_pub(cfg: &Config, path: PathBuf, name: Option<&str>, tmp: bool) -> Result<()> {
    let bag = cfg.bag("public")?;
    let staged = stage_app(&path, bag, name, tmp)?;
    println!("✓ {} → public bag", staged.name);
    deploy_bag(bag, None)
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
    generate_index(bag)?;
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

fn cmd_serve(cfg: &Config, bag: Option<&str>, port: Option<u16>) -> Result<()> {
    let bag = select_bag(cfg, bag)?;
    let port = port
        .or_else(|| match bag.host {
            Host::Local { port, .. } => port,
            _ => None,
        })
        .unwrap_or(8765);
    generate_index(bag)?;
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
