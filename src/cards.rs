use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::app::AppEntry;
use crate::preview::{PREVIEW_DIR, preview_slug};

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

pub fn from_app_entry(app: &AppEntry) -> CardItem {
    from_app_entry_with_base(app, ".")
}

pub fn from_app_entry_with_base(app: &AppEntry, resource_base: &str) -> CardItem {
    CardItem {
        id: app.name.clone(),
        title: display_title(app),
        href: prefix_resource(resource_base, &app.href),
        description: app.metadata.description.clone(),
        category: property_string(&app.metadata.properties, "category"),
        properties: app.metadata.properties.clone(),
        tmp: app.tmp,
        preview_url: Some(prefix_resource(
            resource_base,
            &format!("./{}/{}.jpg", PREVIEW_DIR, preview_slug(&app.name)),
        )),
        icon: None,
    }
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

fn prefix_resource(resource_base: &str, value: &str) -> String {
    if value.starts_with("http://") || value.starts_with("https://") || value.starts_with('/') {
        return value.to_string();
    }
    let relative = value.strip_prefix("./").unwrap_or(value);
    if resource_base == "." || resource_base.is_empty() {
        format!("./{relative}")
    } else {
        format!("{}/{relative}", resource_base.trim_end_matches('/'))
    }
}
