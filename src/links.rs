use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::Bag;
use crate::metadata::{MetadataOverrides, apply_app_metadata_overrides};
use crate::{Result, err};

pub const LINKS_FILE: &str = ".webby-links.json";
const LINK_MARKER: &str = ".webby-link";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkManifest {
    pub links: Vec<LinkEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub tmp: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkResult {
    pub name: String,
    pub is_dir: bool,
    pub source: PathBuf,
    pub materialized: PathBuf,
}

pub fn link_app(
    src_arg: &Path,
    bag: &Bag,
    name: Option<&str>,
    tmp: bool,
    metadata: &MetadataOverrides,
) -> Result<LinkResult> {
    let (source, is_dir) = validate_app_source(src_arg)?;
    let mut app_name = name
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| infer_name(&source, is_dir));
    validate_link_name(&app_name)?;
    if tmp && !app_name.starts_with("tmp") {
        app_name = format!("tmp-{app_name}");
    }

    fs::create_dir_all(&bag.dir)?;
    let mut manifest = read_manifest(bag)?;
    let entry = LinkEntry {
        name: app_name.clone(),
        path: source.clone(),
        is_dir,
        tmp: app_name.starts_with("tmp"),
    };
    let existing_link = manifest.links.iter().any(|entry| entry.name == app_name);
    ensure_link_target_available(bag, &entry, existing_link)?;
    apply_app_metadata_overrides(&source, is_dir, metadata)?;
    let materialized = materialize_link(bag, &entry, existing_link)?;
    upsert_link(&mut manifest, entry.clone());
    write_manifest(bag, &manifest)?;

    Ok(LinkResult {
        name: app_name,
        is_dir,
        source,
        materialized,
    })
}

pub fn unlink_app(bag: &Bag, name: &str) -> Result<LinkResult> {
    let clean = clean_name(name);
    let mut manifest = read_manifest(bag)?;
    let Some(index) = manifest.links.iter().position(|entry| entry.name == clean) else {
        return Err(err(format!(
            "no linked app named '{name}' in {} bag",
            bag.label
        )));
    };
    let entry = manifest.links.remove(index);
    write_manifest(bag, &manifest)?;
    let materialized = materialized_path(bag, &entry.name, entry.is_dir);
    remove_materialized_link(&materialized)?;
    Ok(LinkResult {
        name: entry.name,
        is_dir: entry.is_dir,
        source: entry.path,
        materialized,
    })
}

pub fn unlink_app_if_exists(bag: &Bag, name: &str) -> Result<Option<LinkResult>> {
    let clean = clean_name(name);
    let manifest = read_manifest(bag)?;
    if manifest.links.iter().any(|entry| entry.name == clean) {
        return unlink_app(bag, name).map(Some);
    }
    Ok(None)
}

pub fn sync_links(bag: &Bag) -> Result<Vec<LinkEntry>> {
    let manifest = read_manifest(bag)?;
    let mut active = Vec::new();
    for entry in manifest.links {
        if !entry.path.exists() {
            eprintln!(
                "  ! linked app '{}' source is missing: {}",
                entry.name,
                entry.path.display()
            );
            let _ = remove_materialized_link(&materialized_path(bag, &entry.name, entry.is_dir));
            continue;
        }
        materialize_link(bag, &entry, true)?;
        active.push(entry);
    }
    Ok(active)
}

pub fn read_manifest(bag: &Bag) -> Result<LinkManifest> {
    let path = manifest_path(bag);
    if !path.exists() {
        return Ok(LinkManifest::default());
    }
    let text = fs::read_to_string(&path)
        .map_err(|e| err(format!("failed to read {}: {e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| err(format!("failed to parse {}: {e}", path.display())))
}

fn write_manifest(bag: &Bag, manifest: &LinkManifest) -> Result<()> {
    fs::create_dir_all(&bag.dir)?;
    let text = serde_json::to_string_pretty(manifest)
        .map_err(|e| err(format!("failed to serialize linked apps: {e}")))?;
    fs::write(manifest_path(bag), format!("{text}\n")).map_err(Into::into)
}

fn manifest_path(bag: &Bag) -> PathBuf {
    bag.dir.join(LINKS_FILE)
}

fn upsert_link(manifest: &mut LinkManifest, entry: LinkEntry) {
    if let Some(existing) = manifest
        .links
        .iter_mut()
        .find(|existing| existing.name == entry.name)
    {
        *existing = entry;
    } else {
        manifest.links.push(entry);
    }
    manifest.links.sort_by(|a, b| a.name.cmp(&b.name));
}

fn validate_app_source(src_arg: &Path) -> Result<(PathBuf, bool)> {
    if !src_arg.exists() {
        return Err(err(format!("not found: {}", src_arg.display())));
    }
    let source = fs::canonicalize(src_arg)?;
    let meta = fs::metadata(&source)?;
    let is_dir = meta.is_dir();
    if is_dir {
        if !source.join("index.html").exists() {
            return Err(err(format!(
                "linked app directory must contain index.html: {}",
                source.display()
            )));
        }
    } else if source
        .extension()
        .and_then(OsStr::to_str)
        .map(|extension| extension.eq_ignore_ascii_case("html"))
        != Some(true)
    {
        return Err(err("a linked standalone app must be a .html file"));
    }
    Ok((source, is_dir))
}

fn materialize_link(bag: &Bag, entry: &LinkEntry, allow_replace: bool) -> Result<PathBuf> {
    fs::create_dir_all(&bag.dir)?;
    let target = materialized_path(bag, &entry.name, entry.is_dir);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    ensure_link_target_available(bag, entry, allow_replace)?;
    if target.exists() || fs::symlink_metadata(&target).is_ok() {
        let meta = fs::symlink_metadata(&target)?;
        if entry.is_dir && meta.is_dir() && target.join(LINK_MARKER).exists() {
            fs::remove_dir_all(&target)?;
        } else if meta.file_type().is_symlink() {
            fs::remove_file(&target)?;
        } else {
            return Err(err(format!(
                "cannot link '{}' because {} already exists and is not a symlink",
                entry.name,
                target.display()
            )));
        }
    }
    if entry.is_dir {
        mirror_link_dir(&entry.path, &target)?;
    } else {
        create_symlink(&entry.path, &target, false)?;
    }
    Ok(target)
}

fn ensure_link_target_available(bag: &Bag, entry: &LinkEntry, allow_replace: bool) -> Result<()> {
    let target = materialized_path(bag, &entry.name, entry.is_dir);
    if !target.exists() && fs::symlink_metadata(&target).is_err() {
        return Ok(());
    }
    let meta = fs::symlink_metadata(&target)?;
    let is_link_mount = meta.file_type().is_symlink()
        || (entry.is_dir && meta.is_dir() && target.join(LINK_MARKER).exists());
    if allow_replace && is_link_mount {
        return Ok(());
    }
    Err(err(format!(
        "cannot link '{}' because {} already exists; remove it first",
        entry.name,
        target.display()
    )))
}

fn remove_materialized_link(path: &Path) -> Result<()> {
    if !path.exists() && fs::symlink_metadata(path).is_err() {
        return Ok(());
    }
    let meta = fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        fs::remove_file(path)?;
        return Ok(());
    }
    if meta.is_dir() && path.join(LINK_MARKER).exists() {
        fs::remove_dir_all(path)?;
        return Ok(());
    }
    Err(err(format!(
        "refusing to remove {} because it is not a webby link mount",
        path.display()
    )))
}

fn materialized_path(bag: &Bag, name: &str, is_dir: bool) -> PathBuf {
    if is_dir {
        bag.dir.join(name)
    } else {
        bag.dir.join(format!("{name}.html"))
    }
}

fn clean_name(name: &str) -> String {
    name.trim_end_matches('/')
        .trim_end_matches(".html")
        .to_string()
}

fn validate_link_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name == "index"
        || name.starts_with('.')
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name == "webby-card-grid"
        || name == "webby-cards"
    {
        return Err(err(format!("invalid linked app name '{name}'")));
    }
    Ok(())
}

fn mirror_link_dir(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    fs::write(
        target.join(LINK_MARKER),
        format!("source={}\n", source.display()),
    )?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if should_skip_link(&name) {
            continue;
        }
        let from = entry.path();
        let to = target.join(&name);
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            mirror_link_dir(&from, &to)?;
        } else if file_type.is_file() || file_type.is_symlink() {
            create_symlink(&from, &to, false)?;
        }
    }
    Ok(())
}

fn should_skip_link(name: &OsStr) -> bool {
    matches!(
        name.to_str(),
        Some(".git" | ".wrangler" | ".DS_Store" | "node_modules" | "logs")
    ) || name
        .to_str()
        .map(|name| name == ".env" || name.starts_with(".env."))
        .unwrap_or(false)
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

#[cfg(unix)]
fn create_symlink(source: &Path, target: &Path, _is_dir: bool) -> Result<()> {
    std::os::unix::fs::symlink(source, target).map_err(Into::into)
}

#[cfg(windows)]
fn create_symlink(source: &Path, target: &Path, is_dir: bool) -> Result<()> {
    if is_dir {
        std::os::windows::fs::symlink_dir(source, target).map_err(Into::into)
    } else {
        std::os::windows::fs::symlink_file(source, target).map_err(Into::into)
    }
}
