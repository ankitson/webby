use minijinja::{context, Environment};

use crate::app::AppEntry;
use crate::preview::preview_slug;

fn make_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_template("style.css", include_str!("../templates/style.css"))
        .expect("style.css template");
    env.add_template("index.html", include_str!("../templates/index.html"))
        .expect("index.html template");
    env.add_template("browse.html", include_str!("../templates/browse.html"))
        .expect("browse.html template");
    env
}

fn card(app: &AppEntry) -> String {
    let hue = hue_for(&app.name);
    let hue_shift = (hue + 72) % 360;
    let preview = format!("./.webby-previews/{}.jpg", preview_slug(&app.name));
    format!(
        "<article class=\"site\" style=\"--tile-hue: {}; --tile-shift: {}; --preview-image: url('{}')\"><a class=\"preview-link\" href=\"{}\" aria-label=\"Open {}\"><div class=\"preview\" aria-hidden=\"true\"></div></a><a class=\"site-title\" href=\"{}\">{}</a></article>\n    ",
        hue,
        hue_shift,
        esc(&preview),
        esc(&app.href),
        esc(&app.name),
        esc(&app.href),
        esc(&app.name),
    )
}

fn hue_for(value: &str) -> u32 {
    value.bytes().fold(0u32, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(byte as u32)
    }) % 360
}

fn esc(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn render_index(apps: &[AppEntry], title: &str) -> String {
    let tools: Vec<&AppEntry> = apps.iter().filter(|a| !a.tmp).collect();
    let temp: Vec<&AppEntry> = apps.iter().filter(|a| a.tmp).collect();
    let items: String = tools.iter().map(|a| card(a)).collect();
    let scratch_items: String = temp.iter().map(|a| card(a)).collect();

    make_env()
        .get_template("index.html")
        .expect("index.html template")
        .render(context! { title, items, scratch_items })
        .expect("render index.html")
}

pub fn render_caddy_browse_template() -> String {
    make_env()
        .get_template("browse.html")
        .expect("browse.html template")
        .render(context! {})
        .expect("render browse.html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_uses_preview_tiles_without_old_labels() {
        let apps = vec![
            AppEntry {
                name: "alpha".to_string(),
                is_dir: true,
                href: "./alpha/".to_string(),
                tmp: false,
            },
            AppEntry {
                name: "tmp-beta".to_string(),
                is_dir: false,
                href: "./tmp-beta.html".to_string(),
                tmp: true,
            },
        ];

        let html = render_index(&apps, "webby");

        assert!(html.contains("<h1 class=\"sr-only\">webby</h1>"));
        assert!(html.contains("<article class=\"site\" style=\"--tile-hue:"));
        assert!(html.contains("style=\"--tile-hue:"));
        assert!(html.contains("--preview-image: url('./.webby-previews/alpha.jpg')"));
        assert!(html.contains("<div class=\"preview\" aria-hidden=\"true\"></div>"));
        assert!(html.contains("class=\"preview-link\""));
        assert!(html.contains("<a class=\"site-title\" href=\"./alpha/\">alpha</a>"));
        assert!(html.contains("<a class=\"site-title\" href=\"./tmp-beta.html\">tmp-beta</a>"));
        assert!(!html.contains("<iframe"));
        assert!(!html.contains(">alpha</span>"));
        assert!(!html.contains(">beta</span>"));
        assert!(!html.contains("Index"));
        assert!(!html.contains("entries"));
        assert!(!html.contains(">tool<"));
        assert!(!html.contains(">page<"));
    }

    #[test]
    fn browse_template_contains_caddy_markers() {
        let html = render_caddy_browse_template();
        assert!(html.contains("{{range .Items}}"));
        assert!(html.contains("{{end}}"));
        assert!(html.contains("data-name="));
        assert!(html.contains("data-isdir="));
        assert!(html.contains("data-url="));
        assert!(html.contains("class=\"site\"") || html.contains("id=\"tools-grid\""));
        assert!(html.contains("hueFor"));
        assert!(html.contains("previewSlug"));
        assert!(html.contains("if (/^\\./.test(raw)) continue;"));
        assert!(!html.contains("createElement('iframe')"));
    }
}
