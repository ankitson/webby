use crate::app::AppEntry;

pub fn render_index(apps: &[AppEntry], title: &str) -> String {
    let tools: Vec<&AppEntry> = apps.iter().filter(|a| !a.tmp).collect();
    let temp: Vec<&AppEntry> = apps.iter().filter(|a| a.tmp).collect();
    let mut body = String::new();

    body.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    body.push_str("  <meta charset=\"utf-8\">\n");
    body.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    body.push_str(&format!("  <title>{}</title>\n", esc(title)));
    body.push_str("  <style>\n");
    body.push_str(STYLE);
    body.push_str("  </style>\n</head>\n<body>\n<main>\n");
    body.push_str(&format!(
        "  <header><p>Index · {} entries</p><h1>{}</h1></header>\n",
        tools.len(),
        esc(title)
    ));
    body.push_str("  <section class=\"grid\">\n");
    if tools.is_empty() {
        body.push_str("    <p class=\"empty\">Nothing here yet.</p>\n");
    } else {
        for (i, app) in tools.iter().enumerate() {
            body.push_str(&card(app, i));
        }
    }
    body.push_str("  </section>\n");

    if !temp.is_empty() {
        body.push_str("  <h2>Temp</h2>\n  <section class=\"grid\">\n");
        for (i, app) in temp.iter().enumerate() {
            body.push_str(&card(app, i));
        }
        body.push_str("  </section>\n");
    }

    body.push_str("</main>\n</body>\n</html>\n");
    body
}

fn card(app: &AppEntry, index: usize) -> String {
    let display = if app.tmp {
        app.name
            .strip_prefix("tmp-")
            .or_else(|| app.name.strip_prefix("tmp_"))
            .unwrap_or(&app.name)
    } else {
        &app.name
    };
    let meta = if app.tmp {
        "scratch"
    } else if app.is_dir {
        "tool"
    } else {
        "page"
    };
    format!(
        "    <a class=\"card\" href=\"{}\"><span>{:02}</span><strong>{}</strong><small>{}</small></a>\n",
        esc(&app.href),
        index + 1,
        esc(display),
        meta
    )
}

fn esc(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const STYLE: &str = r#"    :root {
      color-scheme: light dark;
      --paper: #f6f4ef;
      --ink: #201f1b;
      --muted: #6d675e;
      --line: rgba(32,31,27,.14);
      --panel: rgba(255,255,255,.58);
      --accent: #b75f32;
    }
    @media (prefers-color-scheme: dark) {
      :root {
        --paper: #11110f;
        --ink: #eee7db;
        --muted: #aaa296;
        --line: rgba(255,255,255,.12);
        --panel: rgba(255,255,255,.04);
        --accent: #e08a52;
      }
    }
    * { box-sizing: border-box; }
    body { margin: 0; background: var(--paper); color: var(--ink); font: 16px/1.45 system-ui, sans-serif; }
    main { width: min(1120px, calc(100vw - 32px)); margin: 0 auto; padding: 64px 0; }
    header { border-bottom: 1px solid var(--line); margin-bottom: 28px; }
    header p, small { color: var(--muted); text-transform: uppercase; letter-spacing: .12em; font-size: 12px; }
    h1 { margin: 0 0 24px; font-size: clamp(42px, 8vw, 78px); line-height: .95; font-weight: 650; }
    h2 { margin: 42px 0 16px; font-size: 14px; letter-spacing: .16em; text-transform: uppercase; color: var(--muted); }
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, 260px), 1fr)); gap: 12px; }
    .card { display: grid; grid-template-columns: auto 1fr; gap: 4px 14px; align-items: baseline; padding: 18px; color: inherit; text-decoration: none; border: 1px solid var(--line); background: var(--panel); border-radius: 6px; }
    .card:hover { border-color: var(--accent); }
    .card span { color: var(--accent); font-variant-numeric: tabular-nums; }
    .card strong { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
    .card small { grid-column: 2; }
    .empty { color: var(--muted); }
"#;
