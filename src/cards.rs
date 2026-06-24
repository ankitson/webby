use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::app::AppEntry;
use crate::preview::{PREVIEW_DIR, PREVIEW_EXT, preview_slug};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardItem {
    pub id: String,
    pub title: String,
    pub href: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub properties: BTreeMap<String, Value>,
    pub tmp: bool,
    pub preview_url: Option<String>,
    pub icon: Option<String>,
}

#[cfg(test)]
pub fn from_app_entry(app: &AppEntry) -> CardItem {
    from_app_entry_with_preview(app, default_preview_url(&app.name))
}

pub fn from_app_entry_with_preview(app: &AppEntry, preview_url: Option<String>) -> CardItem {
    CardItem {
        id: app.name.clone(),
        title: display_title(app),
        href: app.href.clone(),
        description: app.metadata.description.clone(),
        category: property_string(&app.metadata.properties, "category"),
        properties: app.metadata.properties.clone(),
        tmp: app.tmp,
        preview_url,
        icon: None,
    }
}

pub fn existing_preview_url(bag_dir: &Path, app_name: &str) -> Option<String> {
    let slug = preview_slug(app_name);
    [PREVIEW_EXT, "jpg", "jpeg", "png"]
        .into_iter()
        .find(|extension| {
            bag_dir
                .join(PREVIEW_DIR)
                .join(format!("{slug}.{extension}"))
                .exists()
        })
        .map(|extension| format!("./{PREVIEW_DIR}/{slug}.{extension}"))
}

#[cfg(test)]
fn default_preview_url(app_name: &str) -> Option<String> {
    Some(format!(
        "./{}/{}.{}",
        PREVIEW_DIR,
        preview_slug(app_name),
        PREVIEW_EXT
    ))
}

fn display_title(app: &AppEntry) -> String {
    if app.tmp {
        app.name
            .strip_prefix("tmp-")
            .or_else(|| app.name.strip_prefix("tmp_"))
            .unwrap_or(&app.name)
            .to_string()
    } else {
        app.metadata
            .title
            .clone()
            .unwrap_or_else(|| app.name.clone())
    }
}

fn property_string(properties: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    properties
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}
