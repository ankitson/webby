use minijinja::{Environment, context};

use crate::app::AppEntry;
use crate::cards::{CardItem, from_app_entry};

const PREVIEW_PRELOAD_LIMIT: usize = 6;

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
    render_index_from_items(&items, title, chrome)
}

fn render_index_from_items(items: &[CardItem], title: &str, chrome: &IndexChromeContent) -> String {
    let cards_html = render_static_cards(items);
    let preview_preloads = render_preview_preloads(items);

    make_env()
        .get_template("index.html")
        .expect("index.html template")
        .render(context! {
            title,
            cards_html,
            preview_preloads,
            chrome_head => chrome.head,
            chrome_body => chrome.body,
        })
        .expect("render index.html")
}

fn render_static_cards(items: &[CardItem]) -> String {
    if items.is_empty() {
        return "  <p class=\"empty\">Nothing here yet.</p>".to_string();
    }

    let mut html = String::from("  <div class=\"webby-grid\" aria-label=\"Sites\">\n");
    for (index, item) in items.iter().enumerate() {
        html.push_str(&render_static_card(item, index));
    }
    html.push_str("  </div>");
    html
}

fn render_static_card(item: &CardItem, index: usize) -> String {
    let title_text = escape_html_text(&item.title);
    let title_attr = escape_html_attr(&item.title);
    let href = escape_html_attr(&item.href);
    let loading = if index < PREVIEW_PRELOAD_LIMIT {
        "eager"
    } else {
        "lazy"
    };
    let fetch_priority = if index < PREVIEW_PRELOAD_LIMIT {
        "high"
    } else {
        "low"
    };
    let preview = item
        .preview_url
        .as_deref()
        .map(|url| {
            format!(
                "      <img class=\"webby-preview-image\" src=\"{}\" alt=\"\" width=\"960\" height=\"600\" loading=\"{}\" decoding=\"async\" fetchpriority=\"{}\">\n",
                escape_html_attr(url),
                loading,
                fetch_priority
            )
        })
        .unwrap_or_default();
    let temp_label = if item.tmp {
        "      <span class=\"webby-temp-label\">temp</span>\n"
    } else {
        ""
    };

    format!(
        "    <article class=\"webby-site\">\n      <a class=\"webby-preview-link\" href=\"{href}\" aria-label=\"Open {title_attr}\">\n{preview}      </a>\n      <div class=\"webby-site-caption\">\n        <a class=\"webby-site-title\" href=\"{href}\">{title_text}</a>\n{temp_label}      </div>\n    </article>\n"
    )
}

fn render_preview_preloads(items: &[CardItem]) -> String {
    items
        .iter()
        .filter_map(|item| item.preview_url.as_deref())
        .take(PREVIEW_PRELOAD_LIMIT)
        .map(|href| {
            format!(
                "  <link rel=\"preload\" as=\"image\" href=\"{}\" fetchpriority=\"high\">",
                escape_html_attr(href)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_html_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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
        assert!(html.contains("<div class=\"webby-grid\" aria-label=\"Sites\">"));
        assert!(html.contains("<article class=\"webby-site\">"));
        assert!(html.contains(
            "<a class=\"webby-preview-link\" href=\"./alpha/\" aria-label=\"Open alpha\">"
        ));
        assert!(html.contains("<img class=\"webby-preview-image\" src=\"./webby-previews/alpha.webp\" alt=\"\" width=\"960\" height=\"600\" loading=\"eager\" decoding=\"async\" fetchpriority=\"high\">"));
        assert!(html.contains("<span class=\"webby-temp-label\">temp</span>"));
        assert!(html.contains(
            "<link rel=\"preload\" as=\"image\" href=\"./webby-previews/alpha.webp\" fetchpriority=\"high\">"
        ));
        assert!(!html.contains("<webby-card-grid"));
        assert!(!html.contains("id=\"webby-card-data\""));
        assert!(!html.contains("<script type=\"module\">"));
        assert!(!html.contains("modulepreload"));
        assert!(!html.contains("import \"./webby-card-grid.js\";"));
        assert!(!html.contains("<iframe"));
        assert!(!html.contains("Index"));
        assert!(!html.contains("entries"));
        assert!(!html.contains("bag-nav"));
        assert!(!html.contains("site-header"));
        assert!(!html.contains(">tool<"));
        assert!(!html.contains(">page<"));
    }

    #[test]
    fn index_escapes_static_card_markup() {
        let apps = vec![AppEntry {
            name: "alpha".to_string(),
            is_dir: true,
            href: "./alpha/".to_string(),
            tmp: false,
            metadata: AppMetadata {
                title: Some("A & <B> \"Q\"".to_string()),
                ..AppMetadata::default()
            },
        }];

        let html = render_index(&apps, "webby", &IndexChromeContent::default());

        assert!(html.contains("aria-label=\"Open A &amp; &lt;B&gt; &quot;Q&quot;\""));
        assert!(html.contains(">A &amp; &lt;B&gt; \"Q\"</a>"));
        assert!(!html.contains(">A & <B>"));
    }

    #[test]
    fn index_renders_empty_state_without_js() {
        let html = render_index(&[], "webby", &IndexChromeContent::default());

        assert!(html.contains("<p class=\"empty\">Nothing here yet.</p>"));
        assert!(!html.contains("<webby-card-grid"));
        assert!(!html.contains("<script type=\"module\">"));
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
