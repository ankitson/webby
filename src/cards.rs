use serde::Serialize;

use crate::app::AppEntry;
use crate::preview::preview_slug;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardItem {
    pub id: String,
    pub title: String,
    pub href: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tmp: bool,
    pub preview_url: Option<String>,
    pub icon: Option<String>,
}

pub fn from_app_entry(app: &AppEntry) -> CardItem {
    CardItem {
        id: app.name.clone(),
        title: display_title(app),
        href: app.href.clone(),
        description: None,
        category: None,
        tmp: app.tmp,
        preview_url: Some(format!("./.webby-previews/{}.jpg", preview_slug(&app.name))),
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
        app.name.clone()
    }
}
