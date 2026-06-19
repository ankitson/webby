use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::app::{list_apps, AppEntry};
use crate::config::Bag;
use crate::{err, Result};

pub fn capture_previews(
    bag: &Bag,
    force: bool,
    width: u32,
    height: u32,
    timeout: Duration,
    app_filter: Option<&str>,
) -> Result<()> {
    let mut apps = list_apps(bag)?;
    if let Some(name) = app_filter {
        apps = filter_apps(apps, name);
        if apps.is_empty() {
            return Err(err(format!("no app named '{name}' in {} bag", bag.label)));
        }
    }
    if apps.is_empty() {
        println!("✓ no apps in {}", bag.dir.display());
        return Ok(());
    }

    let out_dir = bag.dir.join(".webby-previews");
    fs::create_dir_all(&out_dir)?;

    let mut captured = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for app in apps {
        let out = out_dir.join(format!("{}.jpg", preview_slug(&app.name)));
        if out.exists() && !force {
            skipped += 1;
            println!("skip {} ({})", app.name, out.display());
            continue;
        }

        let path = capture_path(bag, &app);
        print!("capture {} ... ", app.name);
        let _ = std::io::stdout().flush();
        match capture_with_shot_scraper(&path, &out, width, height, timeout) {
            Ok(()) => {
                captured += 1;
                println!("{}", out.display());
            }
            Err(error) => {
                failed += 1;
                println!("failed");
                eprintln!("  ! {}: {}", app.name, error);
            }
        }
    }

    println!(
        "✓ previews: {} captured, {} skipped, {} failed in {}",
        captured,
        skipped,
        failed,
        out_dir.display()
    );
    Ok(())
}

fn filter_apps(apps: Vec<AppEntry>, name: &str) -> Vec<AppEntry> {
    let clean = clean_app_name(name);
    apps.into_iter()
        .filter(|app| clean_app_name(&app.name) == clean)
        .collect()
}

fn clean_app_name(name: &str) -> String {
    name.trim()
        .trim_end_matches('/')
        .trim_end_matches(".html")
        .to_string()
}

pub fn preview_slug(value: &str) -> String {
    let mut slug = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            slug.push(ch.to_ascii_lowercase());
        } else {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_string()
}

fn capture_with_shot_scraper(
    path: &Path,
    out: &Path,
    width: u32,
    height: u32,
    timeout: Duration,
) -> Result<()> {
    let input = path_to_arg(path);
    let output = path_to_arg(out);
    let width = width.to_string();
    let height = height.to_string();
    let timeout_ms = timeout.as_millis().max(1).to_string();

    let result = Command::new("uvx")
        .args([
            "shot-scraper",
            "shot",
            &input,
            "--output",
            &output,
            "--width",
            &width,
            "--height",
            &height,
            "--quality",
            "82",
            "--wait",
            "2000",
            "--timeout",
            &timeout_ms,
            "--silent",
        ])
        .output()
        .map_err(|e| err(format!("failed to run `uvx shot-scraper`: {e}")))?;

    if result.status.success() && out.exists() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
    let detail = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("shot-scraper exited with {}", result.status)
    };
    Err(err(detail))
}

fn capture_path(bag: &Bag, app: &AppEntry) -> PathBuf {
    let rel = app.href.trim_start_matches("./").trim_end_matches('/');
    if app.is_dir {
        bag.dir.join(rel).join("index.html")
    } else {
        bag.dir.join(rel)
    }
}

fn path_to_arg(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}
