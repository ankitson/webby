use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Bag;
use crate::metadata::{AppMetadata, read_app_metadata};
use crate::preview::PREVIEW_DIR;
use crate::{Result, err};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppEntry {
    pub name: String,
    pub is_dir: bool,
    pub href: String,
    pub tmp: bool,
    pub metadata: AppMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageResult {
    pub name: String,
    pub is_dir: bool,
    pub relative_path: String,
}

pub fn stage_app(src_arg: &Path, bag: &Bag, name: Option<&str>, tmp: bool) -> Result<StageResult> {
    if !src_arg.exists() {
        return Err(err(format!("not found: {}", src_arg.display())));
    }
    fs::create_dir_all(&bag.dir)?;

    let src = fs::canonicalize(src_arg)?;
    let meta = fs::metadata(&src)?;
    let is_dir = meta.is_dir();
    let mut app_name = name
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| infer_name(&src, is_dir));

    if !is_dir
        && src
            .extension()
            .and_then(OsStr::to_str)
            .map(|e| e.eq_ignore_ascii_case("html"))
            != Some(true)
    {
        return Err(err(
            "a standalone app must be a .html file, or pass a directory with index.html",
        ));
    }
    if tmp && !app_name.starts_with("tmp") {
        app_name = format!("tmp-{app_name}");
    }

    if is_dir {
        if !src.join("index.html").exists() {
            eprintln!("  ! {app_name}/ has no index.html");
        }
        let dest = bag.dir.join(&app_name);
        remove_any(&dest)?;
        copy_dir_all(&src, &dest)?;
        Ok(StageResult {
            name: app_name.clone(),
            is_dir: true,
            relative_path: format!("{app_name}/"),
        })
    } else {
        let dest_name = format!("{app_name}.html");
        fs::copy(&src, bag.dir.join(&dest_name))?;
        Ok(StageResult {
            name: app_name,
            is_dir: false,
            relative_path: dest_name,
        })
    }
}

pub fn list_apps(bag: &Bag) -> Result<Vec<AppEntry>> {
    if !bag.dir.exists() {
        return Ok(Vec::new());
    }
    let mut apps = Vec::new();
    for entry in fs::read_dir(&bag.dir)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.starts_with('.') || file_name == PREVIEW_DIR {
            continue;
        }
        let path = entry.path();
        let meta = fs::metadata(&path).ok();
        if meta.as_ref().map(|m| m.is_dir()).unwrap_or(false) {
            let tmp = file_name.starts_with("tmp");
            let metadata = read_app_metadata(&path, true)?;
            apps.push(AppEntry {
                name: file_name.clone(),
                is_dir: true,
                href: format!("./{file_name}/"),
                tmp,
                metadata,
            });
        } else if file_name.to_lowercase().ends_with(".html") && file_name != "index.html" {
            let name = file_name.trim_end_matches(".html").to_string();
            let tmp = name.starts_with("tmp");
            let metadata = read_app_metadata(&path, false)?;
            apps.push(AppEntry {
                name: name.clone(),
                is_dir: false,
                href: format!("./{file_name}"),
                tmp,
                metadata,
            });
        }
    }
    apps.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(apps)
}

pub fn remove_app(bag: &Bag, name: &str) -> Result<PathBuf> {
    let clean = name.trim_end_matches('/').trim_end_matches(".html");
    let dir = bag.dir.join(clean);
    let file = bag.dir.join(format!("{clean}.html"));
    let target = if dir.is_dir() {
        dir
    } else if file.exists() {
        file
    } else {
        return Err(err(format!("no app named '{name}' in {} bag", bag.label)));
    };
    remove_any(&target)?;
    Ok(target)
}

pub fn generate_index(bag: &Bag) -> Result<Vec<AppEntry>> {
    fs::create_dir_all(&bag.dir)?;
    let apps = list_apps(bag)?;
    fs::write(
        bag.dir.join("webby-cards.json"),
        crate::render::render_card_manifest(&apps),
    )?;
    fs::write(
        bag.dir.join("webby-card-grid.js"),
        crate::render::web_component_js(),
    )?;
    if bag.no_index {
        remove_any(&bag.dir.join("index.html"))?;
    } else {
        fs::write(
            bag.dir.join("index.html"),
            crate::render::render_index(&apps, "webby"),
        )?;
    }
    Ok(apps)
}

pub fn app_url(base_url: &str, name: &str, is_dir: bool) -> String {
    if base_url.starts_with('(') {
        return base_url.to_string();
    }
    format!(
        "{}/{}{}",
        base_url.trim_end_matches('/'),
        name.trim_end_matches('/'),
        if is_dir { "/" } else { ".html" }
    )
}

pub fn remove_any(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let meta = fs::symlink_metadata(path)?;
    if meta.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn infer_name(src: &Path, is_dir: bool) -> String {
    let file_name = src.file_name().and_then(OsStr::to_str).unwrap_or("app");
    if is_dir {
        file_name.to_string()
    } else {
        Path::new(file_name)
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or(file_name)
            .to_string()
    }
}

fn copy_dir_all(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        if should_skip_copy(&entry.file_name()) {
            continue;
        }
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn should_skip_copy(name: &OsStr) -> bool {
    matches!(
        name.to_str(),
        Some(".git" | ".wrangler" | ".DS_Store" | "node_modules" | "logs")
    )
}
