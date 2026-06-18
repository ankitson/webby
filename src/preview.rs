use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::app::{list_apps, AppEntry};
use crate::config::Bag;
use crate::{err, Result};

pub fn capture_previews(
    bag: &Bag,
    force: bool,
    width: u32,
    height: u32,
    timeout: Duration,
) -> Result<()> {
    let apps = list_apps(bag)?;
    if apps.is_empty() {
        println!("✓ no apps in {}", bag.dir.display());
        return Ok(());
    }

    let out_dir = bag.dir.join(".webby-previews");
    fs::create_dir_all(&out_dir)?;

    let chrome = chrome_binary()?;
    let mut captured = 0usize;
    let mut skipped = 0usize;

    for app in apps {
        let out = out_dir.join(format!("{}.jpg", preview_slug(&app.name)));
        if out.exists() && !force {
            skipped += 1;
            println!("skip {} ({})", app.name, out.display());
            continue;
        }

        let url = capture_url(bag, &app);
        print!("capture {} ... ", app.name);
        let _ = std::io::stdout().flush();
        match capture_with_chrome(&chrome, &url, &out, width, height, timeout) {
            Ok(()) => {
                captured += 1;
                println!("{}", out.display());
            }
            Err(error) => {
                println!("failed");
                eprintln!("  ! {}: {}", app.name, error);
            }
        }
    }

    println!(
        "✓ previews: {} captured, {} skipped in {}",
        captured,
        skipped,
        out_dir.display()
    );
    Ok(())
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

fn capture_with_chrome(
    chrome: &str,
    url: &str,
    out: &Path,
    width: u32,
    height: u32,
    timeout: Duration,
) -> Result<()> {
    let profile_dir = temp_profile_dir()?;
    let chrome_timeout_ms = timeout.as_millis().max(1).to_string();
    let mut child = Command::new(chrome)
        .args([
            "--headless=new",
            &format!("--timeout={chrome_timeout_ms}"),
            "--disable-gpu",
            "--hide-scrollbars",
            "--ignore-certificate-errors",
            "--allow-file-access-from-files",
            "--no-first-run",
            "--no-default-browser-check",
            "--run-all-compositor-stages-before-draw",
            &format!("--user-data-dir={}", profile_dir.display()),
            &format!("--window-size={width},{height}"),
            "--force-device-scale-factor=1",
            &format!("--screenshot={}", out.display()),
            url,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| err(format!("failed to start {chrome}: {e}")))?;

    let deadline = Instant::now() + timeout + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait()? {
            let _ = fs::remove_dir_all(&profile_dir);
            if status.success() && out.exists() {
                return Ok(());
            }
            return Err(err(format!("Chrome exited with {status}")));
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_dir_all(&profile_dir);
            return Err(err(format!(
                "Chrome screenshot timed out after {:?}",
                timeout
            )));
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn capture_url(bag: &Bag, app: &AppEntry) -> String {
    let rel = app.href.trim_start_matches("./").trim_end_matches('/');
    let path = if app.is_dir {
        bag.dir.join(rel).join("index.html")
    } else {
        bag.dir.join(rel)
    };
    file_url(&path)
}

fn file_url(path: &Path) -> String {
    let absolute = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let raw = absolute.to_string_lossy();
    format!("file://{}", percent_encode_path(&raw))
}

fn percent_encode_path(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn temp_profile_dir() -> Result<PathBuf> {
    let path = std::env::temp_dir().join(format!(
        "webby-preview-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn chrome_binary() -> Result<String> {
    for name in ["google-chrome", "chromium", "chromium-browser"] {
        if command_exists(name) {
            return Ok(name.to_string());
        }
    }
    Err(err("no Chrome/Chromium binary found on PATH"))
}

fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .and_then(|path| {
            std::env::split_paths(&path)
                .map(|dir| dir.join(name))
                .find(|path| path.exists())
        })
        .is_some()
}
