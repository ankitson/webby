use minijinja::{Environment, context};

use crate::app::AppEntry;
use crate::cards::from_app_entry;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IndexChromeContent {
    pub head: String,
    pub body: String,
}

fn make_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_template("style.css", include_str!("../templates/style.css"))
        .expect("style.css template");
    env.add_template("index.html", include_str!("../templates/index.html"))
        .expect("index.html template");
    env
}

pub fn render_index(apps: &[AppEntry], title: &str, chrome: &IndexChromeContent) -> String {
    let items = apps.iter().map(from_app_entry).collect::<Vec<_>>();
    let items_json = serde_json::to_string(&items).expect("serialize card items");

    make_env()
        .get_template("index.html")
        .expect("index.html template")
        .render(
            context! { title, items_json, chrome_head => chrome.head, chrome_body => chrome.body },
        )
        .expect("render index.html")
}

pub fn render_card_manifest(apps: &[AppEntry]) -> String {
    let items = apps.iter().map(from_app_entry).collect::<Vec<_>>();
    serde_json::to_string_pretty(&items).expect("serialize card items")
}

pub fn web_component_js() -> &'static str {
    include_str!("../templates/webby-card-grid.js")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::AppMetadata;

    #[test]
    fn index_uses_preview_tiles_without_old_labels() {
        let apps = vec![
            AppEntry {
                name: "alpha".to_string(),
                is_dir: true,
                href: "./alpha/".to_string(),
                tmp: false,
                metadata: AppMetadata::default(),
            },
            AppEntry {
                name: "tmp-beta".to_string(),
                is_dir: false,
                href: "./tmp-beta.html".to_string(),
                tmp: true,
                metadata: AppMetadata::default(),
            },
        ];

        let html = render_index(&apps, "webby", &IndexChromeContent::default());

        assert!(html.contains("<h1 class=\"sr-only\">webby</h1>"));
        assert!(html.contains("<webby-card-grid id=\"webby-grid\""));
        assert!(html.contains("type=\"application/json\" id=\"webby-card-data\""));
        assert!(html.contains("grid.setAttribute(\"data-theme\", theme);"));
        assert!(html.contains("\"id\":\"alpha\""));
        assert!(html.contains("\"title\":\"alpha\""));
        assert!(html.contains("\"href\":\"./alpha/\""));
        assert!(html.contains("\"previewUrl\":\"./webby-previews/alpha.webp\""));
        assert!(html.contains("\"id\":\"tmp-beta\""));
        assert!(html.contains("\"title\":\"beta\""));
        assert!(html.contains("\"tmp\":true"));
        assert!(html.contains("import \"./webby-card-grid.js\";"));
        assert!(!html.contains("<iframe"));
        assert!(!html.contains("Index"));
        assert!(!html.contains("entries"));
        assert!(!html.contains("bag-nav"));
        assert!(!html.contains("site-header"));
        assert!(!html.contains(">tool<"));
        assert!(!html.contains(">page<"));
    }

    #[test]
    fn index_includes_optional_chrome_fragments() {
        let html = render_index(
            &[],
            "webby",
            &IndexChromeContent {
                head: "<style>.custom-chrome{color:red}</style>".to_string(),
                body: "<header class=\"custom-chrome\">Custom</header>".to_string(),
            },
        );

        assert!(html.contains("<style>.custom-chrome{color:red}</style>"));
        assert!(html.contains("<header class=\"custom-chrome\">Custom</header>"));
    }

    #[test]
    fn card_manifest_serializes_reusable_card_data() {
        let apps = vec![AppEntry {
            name: "alpha".to_string(),
            is_dir: true,
            href: "./alpha/".to_string(),
            tmp: false,
            metadata: AppMetadata::default(),
        }];

        let json = render_card_manifest(&apps);

        assert!(json.contains("\"id\": \"alpha\""));
        assert!(json.contains("\"href\": \"./alpha/\""));
        assert!(json.contains("\"previewUrl\": \"./webby-previews/alpha.webp\""));
    }

    #[test]
    fn component_asset_defines_custom_element() {
        let js = web_component_js();
        assert!(js.contains("class WebbyCardGrid extends HTMLElement"));
        assert!(js.contains("customElements.define(\"webby-card-grid\""));
        assert!(js.contains("attachShadow({ mode: \"open\" })"));
        assert!(js.contains("group-by-property"));
        assert!(js.contains("--webby-accent"));
        assert!(js.contains("--webby-card-max-width: 800px"));
        assert!(js.contains("--webby-card-max-height: 800px"));
        assert!(!js.contains("linear-gradient"));
    }
}
