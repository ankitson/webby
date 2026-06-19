use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::app::generate_index;
use crate::config::{Bag, Host};
use crate::{Result, err};

pub fn base_url(bag: &Bag, port_override: Option<u16>) -> String {
    match &bag.host {
        Host::Local { url: Some(url), .. }
        | Host::Caddy { url: Some(url) }
        | Host::Command { url: Some(url), .. }
        | Host::CloudflarePages { url: Some(url), .. } => url.trim_end_matches('/').to_string(),
        Host::TailscaleServe {
            url: Some(url),
            path,
            ..
        }
        | Host::TailscaleFunnel {
            url: Some(url),
            path,
            ..
        } => with_path(url, path.as_deref()),
        Host::Local { port, .. } => format!(
            "http://localhost:{}",
            port_override.or(*port).unwrap_or(8765)
        ),
        Host::TailscaleServe { path, .. } | Host::TailscaleFunnel { path, .. } => {
            tailscale_hostname()
                .map(|name| with_path(&format!("https://{name}"), path.as_deref()))
                .unwrap_or_else(|| "(url unknown until tailscale is configured)".to_string())
        }
        Host::CloudflarePages { project, .. } => {
            format!(
                "https://{}.pages.dev",
                project.as_deref().unwrap_or("webby")
            )
        }
        Host::Caddy { .. } | Host::Command { .. } => {
            "(url unknown until host is configured)".to_string()
        }
    }
}

pub fn deploy_bag(bag: &Bag, port_override: Option<u16>) -> Result<()> {
    let apps = generate_index(bag)?;
    match &bag.host {
        Host::Local { .. } => {
            println!("✓ ready: {} app(s) in {}", apps.len(), bag.dir.display());
            println!("  serve: webby serve -b {}", bag.label);
            println!("  url: {}", base_url(bag, port_override));
            Ok(())
        }
        Host::Caddy { .. } => {
            println!("✓ generated: {}/browse.html", bag.dir.display());
            println!("✓ live: {}", base_url(bag, None));
            Ok(())
        }
        Host::TailscaleServe {
            path, background, ..
        } => run_tailscale(
            "serve",
            &bag.dir,
            path.as_deref(),
            background.unwrap_or(true),
            bag,
        ),
        Host::TailscaleFunnel {
            path, background, ..
        } => run_tailscale(
            "funnel",
            &bag.dir,
            path.as_deref(),
            background.unwrap_or(true),
            bag,
        ),
        Host::CloudflarePages {
            project,
            account_id,
            token_env,
            token_ref,
            token_command,
            branch,
            ..
        } => run_cloudflare_pages(
            bag,
            project.as_deref().unwrap_or("webby"),
            account_id.as_deref(),
            token_env.as_deref().unwrap_or("CLOUDFLARE_API_TOKEN"),
            token_ref.as_deref(),
            token_command.as_deref(),
            branch.as_deref().unwrap_or("main"),
            apps.len(),
        ),
        Host::Command { deploy, .. } => {
            let Some(template) = deploy else {
                return Err(err(format!("bag '{}' has no deploy command", bag.label)));
            };
            run_command_template(template, bag)?;
            println!("✓ live: {}", base_url(bag, None));
            Ok(())
        }
    }
}

pub fn after_add(bag: &Bag) -> Result<()> {
    if let Host::Command {
        after_add: Some(template),
        ..
    } = &bag.host
    {
        run_command_template(template, bag)?;
    }
    Ok(())
}

pub fn attach_domain(bag: &Bag, hostname: &str) -> Result<()> {
    let Host::CloudflarePages {
        project,
        account_id,
        token_env,
        token_ref,
        token_command,
        ..
    } = &bag.host
    else {
        return Err(err(format!(
            "bag '{}' is not a Cloudflare Pages bag",
            bag.label
        )));
    };
    let project = project.as_deref().unwrap_or("webby");
    let account_id = account_id
        .clone()
        .or_else(|| env::var("CLOUDFLARE_ACCOUNT_ID").ok())
        .or_else(|| env::var("CF_ACCOUNT_ID").ok())
        .ok_or_else(|| {
            err("missing Cloudflare account id. Set CLOUDFLARE_ACCOUNT_ID or accountId.")
        })?;
    let token = read_cloudflare_token(
        token_env.as_deref().unwrap_or("CLOUDFLARE_API_TOKEN"),
        token_ref.as_deref(),
        token_command.as_deref(),
    )?
    .ok_or_else(|| {
        err("missing Cloudflare token. Set CLOUDFLARE_API_TOKEN, tokenRef, or tokenCommand.")
    })?;
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/pages/projects/{}/domains",
        account_id, project
    );
    let body = serde_json::json!({ "name": hostname }).to_string();
    let response = ureq::post(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/json")
        .send_string(&body);
    match response {
        Ok(resp) if (200..300).contains(&resp.status()) => {}
        Ok(resp) => {
            return Err(err(format!(
                "Cloudflare domain attach failed with HTTP {}",
                resp.status()
            )));
        }
        Err(ureq::Error::Status(code, _)) => {
            return Err(err(format!(
                "Cloudflare domain attach failed with HTTP {code}"
            )));
        }
        Err(error) => return Err(err(format!("Cloudflare domain attach failed: {error}"))),
    }
    println!("✓ attached {hostname} to Pages project '{project}'");
    Ok(())
}

pub fn open_app(url: &str) {
    let _ = Command::new("xdg-open")
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn run_tailscale(
    mode: &str,
    dir: &Path,
    path: Option<&str>,
    background: bool,
    bag: &Bag,
) -> Result<()> {
    let mut args = vec![mode.to_string()];
    if background {
        args.push("--bg".to_string());
    }
    if let Some(path) = path.filter(|p| *p != "/") {
        args.push("--set-path".to_string());
        args.push(path.to_string());
    }
    args.push(dir.to_string_lossy().to_string());
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    run_command("tailscale", &refs, &[])?;
    println!("✓ live: {}", base_url(bag, None));
    Ok(())
}

fn run_cloudflare_pages(
    bag: &Bag,
    project: &str,
    account_id: Option<&str>,
    token_env: &str,
    token_ref: Option<&str>,
    token_command: Option<&str>,
    branch: &str,
    app_count: usize,
) -> Result<()> {
    let mut envs = Vec::new();
    let account_id = account_id
        .map(str::to_string)
        .or_else(|| env::var("CLOUDFLARE_ACCOUNT_ID").ok());
    if let Some(account_id) = account_id {
        envs.push(("CLOUDFLARE_ACCOUNT_ID".to_string(), account_id.to_string()));
    }
    if let Some(token) = read_cloudflare_token(token_env, token_ref, token_command)? {
        envs.push(("CLOUDFLARE_API_TOKEN".to_string(), token));
    }

    println!("  deploying {app_count} app(s) to Pages project '{project}'...");
    let dir = bag.dir.to_string_lossy().to_string();
    run_wrangler(
        &[
            "pages",
            "deploy",
            &dir,
            "--project-name",
            project,
            "--branch",
            branch,
            "--commit-dirty=true",
        ],
        &envs,
    )?;
    println!("✓ live: {}", base_url(bag, None));
    Ok(())
}

fn read_cloudflare_token(
    token_env: &str,
    token_ref: Option<&str>,
    token_command: Option<&str>,
) -> Result<Option<String>> {
    if let Ok(token) = env::var(token_env) {
        return Ok(Some(token));
    }
    if let Some(token_ref) = token_ref {
        let output = Command::new("op").args(["read", token_ref]).output()?;
        if !output.status.success() {
            return Err(err(format!("op read failed for {token_ref}")));
        }
        return Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ));
    }
    if let Some(command) = token_command {
        let output = Command::new("sh").args(["-c", command]).output()?;
        if !output.status.success() {
            return Err(err("token command failed"));
        }
        return Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ));
    }
    Ok(None)
}

fn in_path(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|p| p.join(name).is_file()))
        .unwrap_or(false)
}

fn run_wrangler(args: &[&str], envs: &[(String, String)]) -> Result<()> {
    if in_path("wrangler") {
        run_command("wrangler", args, envs)
    } else {
        let full: Vec<&str> = ["--yes", "wrangler"]
            .into_iter()
            .chain(args.iter().copied())
            .collect();
        run_command("npx", &full, envs)
    }
}

fn run_command(program: &str, args: &[&str], envs: &[(String, String)]) -> Result<()> {
    let mut command = Command::new(program);
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    let status = command
        .status()
        .map_err(|e| err(format!("failed to run {program}: {e}")))?;
    if !status.success() {
        return Err(err(format!("{program} exited with {status}")));
    }
    Ok(())
}

fn run_command_template(template: &str, bag: &Bag) -> Result<()> {
    let command = template
        .replace("{dir}", &bag.dir.to_string_lossy())
        .replace("{label}", &bag.label)
        .replace("{url}", &base_url(bag, None));
    let status = Command::new("sh").args(["-c", &command]).status()?;
    if !status.success() {
        return Err(err(format!("command failed: {command}")));
    }
    Ok(())
}

fn tailscale_hostname() -> Option<String> {
    let output = Command::new("tailscale")
        .args(["status", "--json"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    value
        .pointer("/Self/DNSName")
        .and_then(|v| v.as_str())
        .map(|dns| dns.trim_end_matches('.').to_string())
}

fn with_path(base: &str, path: Option<&str>) -> String {
    let base = base.trim_end_matches('/');
    match path {
        Some("/") | None => base.to_string(),
        Some(path) => format!("{}/{}", base, path.trim_matches('/')),
    }
}
