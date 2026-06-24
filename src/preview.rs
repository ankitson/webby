use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::app::{AppEntry, list_apps};
use crate::config::Bag;
use crate::{Result, err};

pub const PREVIEW_DIR: &str = "webby-previews";
pub const PREVIEW_EXT: &str = "webp";

const OPTIMIZED_WIDTH: u32 = 960;
const WEBP_QUALITY: u8 = 78;
const WEBP_OPTIMIZE_SCRIPT: &str = r#"
import sys
from pathlib import Path
from PIL import Image

source = Path(sys.argv[1])
output = Path(sys.argv[2])
width = int(sys.argv[3])
quality = int(sys.argv[4])

image = Image.open(source).convert("RGB")
if image.width > width:
    height = round(image.height * width / image.width)
    resampling = getattr(getattr(Image, "Resampling", Image), "LANCZOS")
    image = image.resize((width, height), resampling)

image.save(output, format="WEBP", quality=quality, method=6)
"#;

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

    let out_dir = bag.dir.join(PREVIEW_DIR);
    fs::create_dir_all(&out_dir)?;

    let mut captured = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for app in apps {
        let out = out_dir.join(format!("{}.{}", preview_slug(&app.name), PREVIEW_EXT));
        if out.exists() && !force {
            skipped += 1;
            println!("skip {} ({})", app.name, out.display());
            continue;
        }

        let path = capture_path(bag, &app);
        let capture_out = out.with_extension("capture.jpg");
        print!("capture {} ... ", app.name);
        let _ = std::io::stdout().flush();
        match capture_optimized_preview(&path, &capture_out, &out, width, height, timeout) {
            Ok(()) => {
                captured += 1;
                remove_legacy_preview_jpegs(&out);
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

fn remove_legacy_preview_jpegs(out: &Path) {
    let _ = fs::remove_file(out.with_extension("jpg"));
    let _ = fs::remove_file(out.with_extension("jpeg"));
}

fn capture_optimized_preview(
    path: &Path,
    capture_out: &Path,
    out: &Path,
    width: u32,
    height: u32,
    timeout: Duration,
) -> Result<()> {
    capture_with_shot_scraper(path, capture_out, width, height, timeout)?;
    let result = optimize_preview(capture_out, out);
    let _ = fs::remove_file(capture_out);
    result
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

fn optimize_preview(source: &Path, out: &Path) -> Result<()> {
    let source_arg = path_to_arg(source);
    let output_arg = path_to_arg(out);
    let width = OPTIMIZED_WIDTH.to_string();
    let quality = WEBP_QUALITY.to_string();

    let result = Command::new("uvx")
        .args([
            "--with",
            "pillow",
            "python",
            "-c",
            WEBP_OPTIMIZE_SCRIPT,
            &source_arg,
            &output_arg,
            &width,
            &quality,
        ])
        .output()
        .map_err(|e| err(format!("failed to run `uvx --with pillow python`: {e}")))?;

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
        format!("preview optimization exited with {}", result.status)
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
