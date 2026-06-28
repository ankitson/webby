#!/usr/bin/env python3
"""Generate README preview screenshots from a temporary Webby bag."""

from __future__ import annotations

import html
import json
import os
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ASSET_DIR = ROOT / "docs" / "assets"
WEBBY = ROOT / "target" / "debug" / "webby"


SAMPLES = [
    {
        "name": "network-audit",
        "title": "Network Audit",
        "description": "Firewall, DNS, and tailnet notes.",
        "category": "Docs",
        "kind": "docs",
        "accent": "#4f8cff",
        "lines": ["pfSense uplinks", "Tailscale routes", "DNS overrides"],
    },
    {
        "name": "jobsearch-docs",
        "title": "Job Search Docs",
        "description": "Tracker architecture and automation runbooks.",
        "category": "Docs",
        "kind": "docs",
        "accent": "#17a673",
        "lines": ["Ingestion", "Enrichment agents", "Browser autofill"],
    },
    {
        "name": "media-console",
        "title": "Media Console",
        "description": "Playback, queue, and storage health.",
        "category": "Apps",
        "kind": "dashboard",
        "accent": "#e05b42",
        "lines": ["Plex", "Readest", "Archive"],
    },
    {
        "name": "budget-snapshot",
        "title": "Budget Snapshot",
        "description": "A tiny spending and cashflow dashboard.",
        "category": "Apps",
        "kind": "dashboard",
        "accent": "#c17c1f",
        "lines": ["Cashflow", "Subscriptions", "Forecast"],
    },
    {
        "name": "home-wiki",
        "title": "Home Wiki",
        "description": "Household docs, procedures, and checklists.",
        "category": "Docs",
        "kind": "docs",
        "accent": "#8b6be8",
        "lines": ["Appliance notes", "Backup checklist", "Travel setup"],
    },
    {
        "name": "recipe-search",
        "title": "Recipe Search",
        "description": "Fast local recipe lookup.",
        "category": "Apps",
        "kind": "app",
        "accent": "#2c9ab7",
        "lines": ["lentil", "quick dinner", "freezer"],
    },
    {
        "name": "ops-runbook",
        "title": "Ops Runbook",
        "description": "Deploy and recovery steps for home services.",
        "category": "Docs",
        "kind": "docs",
        "accent": "#65707d",
        "lines": ["Caddy reload", "Container restore", "Secret rotation"],
    },
    {
        "name": "agent-prototype",
        "title": "Agent Prototype",
        "description": "A throwaway UI made during an agent session.",
        "category": "Scratch",
        "kind": "app",
        "accent": "#d05786",
        "tmp": True,
        "lines": ["summarize", "triage", "publish"],
    },
]


def run(args: list[str], env: dict[str, str]) -> None:
    subprocess.run(args, cwd=ROOT, env=env, check=True)


def metadata(sample: dict[str, object]) -> str:
    return json.dumps(
        {
            "title": sample["title"],
            "description": sample["description"],
            "properties": {
                "category": sample["category"],
                "kind": "markdown-docs" if sample["kind"] == "docs" else "app",
            },
        },
        indent=2,
    )


def app_html(sample: dict[str, object]) -> str:
    title = html.escape(str(sample["title"]))
    description = html.escape(str(sample["description"]))
    accent = str(sample["accent"])
    lines = [html.escape(str(line)) for line in sample["lines"]]
    kind = str(sample["kind"])
    if kind == "docs":
        body = f"""
        <div class="docs-layout">
          <aside>
            <span class="eyebrow">docs</span>
            <b>{title}</b>
            <a>{lines[0]}</a>
            <a>{lines[1]}</a>
            <a>{lines[2]}</a>
          </aside>
          <article>
            <h1>{title}</h1>
            <p>{description}</p>
            <div class="rule"></div>
            <h2>{lines[0]}</h2>
            <p class="textline wide"></p>
            <p class="textline"></p>
            <pre>webby docs ./docs -b internal</pre>
          </article>
        </div>
        """
    elif kind == "dashboard":
        body = f"""
        <div class="dash">
          <header><span class="eyebrow">dashboard</span><h1>{title}</h1></header>
          <div class="metrics">
            <section><b>98%</b><span>{lines[0]}</span></section>
            <section><b>24</b><span>{lines[1]}</span></section>
            <section><b>7d</b><span>{lines[2]}</span></section>
          </div>
          <div class="chart">
            <i style="height:44%"></i><i style="height:72%"></i><i style="height:56%"></i>
            <i style="height:84%"></i><i style="height:62%"></i><i style="height:91%"></i>
          </div>
        </div>
        """
    else:
        body = f"""
        <div class="app">
          <span class="eyebrow">tool</span>
          <h1>{title}</h1>
          <div class="search">{lines[0]}</div>
          <ul>
            <li><b>{lines[1]}</b><span>{description}</span></li>
            <li><b>{lines[2]}</b><span>Saved locally, published by Webby.</span></li>
          </ul>
        </div>
        """

    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <meta name="description" content="{description}">
  <script type="application/webby+json">
{metadata(sample)}
  </script>
  <style>
    :root {{
      color-scheme: dark;
      --accent: {accent};
      --paper: #111613;
      --ink: #f5f0e8;
      --muted: #aaa397;
      --panel: #1b211d;
      --line: rgba(255,255,255,.14);
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      min-height: 100vh;
      background:
        radial-gradient(circle at 20% 15%, color-mix(in srgb, var(--accent) 30%, transparent), transparent 30%),
        linear-gradient(145deg, #101512, #20251f);
      color: var(--ink);
      font: 16px/1.45 ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }}
    .eyebrow {{
      display: inline-block;
      color: var(--accent);
      font-size: 12px;
      font-weight: 800;
      letter-spacing: .12em;
      text-transform: uppercase;
    }}
    h1, h2, p {{ margin: 0; }}
    .docs-layout {{ display: grid; grid-template-columns: 230px 1fr; min-height: 100vh; }}
    aside {{ padding: 34px 22px; border-right: 1px solid var(--line); background: rgba(0,0,0,.16); }}
    aside b {{ display: block; margin: 12px 0 24px; font-size: 24px; line-height: 1.05; }}
    aside a {{ display: block; margin: 10px 0; color: var(--muted); }}
    article {{ padding: 44px; }}
    article h1, .dash h1, .app h1 {{ margin-top: 10px; font-size: clamp(34px, 7vw, 72px); line-height: .92; letter-spacing: 0; }}
    article p {{ max-width: 560px; margin-top: 18px; color: var(--muted); font-size: 20px; }}
    .rule {{ width: 100%; height: 1px; margin: 36px 0; background: var(--line); }}
    article h2 {{ color: var(--accent); font-size: 24px; }}
    .textline {{ height: 13px; width: 62%; margin-top: 18px; border-radius: 20px; background: rgba(255,255,255,.14); }}
    .textline.wide {{ width: 86%; }}
    pre {{ margin-top: 34px; padding: 18px; border: 1px solid var(--line); border-radius: 8px; background: #0b0f0d; color: #d9f7e5; overflow: hidden; }}
    .dash, .app {{ min-height: 100vh; padding: 44px; display: flex; flex-direction: column; justify-content: space-between; }}
    .metrics {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; margin-top: 38px; }}
    .metrics section, .app li {{ border: 1px solid var(--line); border-radius: 8px; background: rgba(255,255,255,.06); padding: 18px; }}
    .metrics b {{ display: block; font-size: 42px; color: var(--accent); }}
    .metrics span, .app span {{ color: var(--muted); }}
    .chart {{ height: 230px; display: flex; align-items: end; gap: 18px; padding: 28px; border: 1px solid var(--line); border-radius: 8px; background: rgba(0,0,0,.18); }}
    .chart i {{ flex: 1; border-radius: 10px 10px 3px 3px; background: linear-gradient(180deg, var(--accent), rgba(255,255,255,.2)); }}
    .search {{ margin: 34px 0; padding: 18px 20px; border: 1px solid var(--line); border-radius: 8px; background: rgba(255,255,255,.08); color: var(--muted); }}
    .app ul {{ display: grid; gap: 14px; padding: 0; margin: 0; list-style: none; }}
    .app li b {{ display: block; margin-bottom: 6px; color: var(--accent); }}
  </style>
</head>
<body>
{body}
</body>
</html>
"""


def chrome_body(path: Path) -> None:
    path.write_text(
        """<header class="readme-preview-header">
  <div>
    <strong>home.ankitson.com</strong>
    <span>Webby bag: internal</span>
  </div>
  <nav>
    <a>Apps</a>
    <a>Docs</a>
    <a>Scratch</a>
  </nav>
</header>
<style>
  .readme-preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 18px 28px 0;
    color: var(--ink);
    font: 15px/1.4 system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  }
  .readme-preview-header strong { display: block; font-size: 18px; }
  .readme-preview-header span { color: var(--muted); }
  .readme-preview-header nav { display: flex; gap: 10px; }
  .readme-preview-header a {
    border: 1px solid var(--line);
    border-radius: 999px;
    padding: 6px 11px;
    color: var(--muted);
  }
  html, body {
    min-height: 100vh;
    background: #f7f7f2 !important;
  }
  @media (max-width: 640px) {
    .readme-preview-header { align-items: flex-start; flex-direction: column; padding: 14px 14px 0; }
    .readme-preview-header nav { flex-wrap: wrap; }
  }
</style>
""",
        encoding="utf-8",
    )


def main() -> None:
    ASSET_DIR.mkdir(parents=True, exist_ok=True)
    run(["cargo", "build"], os.environ.copy())

    with tempfile.TemporaryDirectory(prefix="webby-readme-preview-") as tmp_name:
        tmp = Path(tmp_name)
        sources = tmp / "sources"
        bag = tmp / "bag"
        chrome = tmp / "chrome-body.html"
        sources.mkdir()
        chrome_body(chrome)

        config = tmp / "config.json"
        config.write_text(
            json.dumps(
                {
                    "defaultBag": "local",
                    "bags": {
                        "local": {
                            "dir": str(bag),
                            "indexChrome": {"body": str(chrome)},
                            "host": {"type": "local", "port": 8765},
                        }
                    },
                },
                indent=2,
            ),
            encoding="utf-8",
        )
        env = os.environ.copy()
        env.update(
            {
                "WEBBY_CONFIG": str(config),
                "WEBBY_DATA_DIR": str(tmp / "data"),
                "WEBBY_ENV": str(tmp / "missing.env"),
            }
        )

        for sample in SAMPLES:
            app_dir = sources / str(sample["name"])
            app_dir.mkdir()
            (app_dir / "index.html").write_text(app_html(sample), encoding="utf-8")
            args = [str(WEBBY), "add", str(app_dir), "-b", "local", "--name", str(sample["name"])]
            if sample.get("tmp"):
                args.append("--tmp")
            run(args, env)

        index = bag / "index.html"
        run(
            [
                str(WEBBY),
                "preview-url",
                str(index),
                str(ASSET_DIR / "webby-homeserver-cards.webp"),
                "--force",
                "--width",
                "1440",
                "--height",
                "620",
                "--timeout-secs",
                "12",
            ],
            env,
        )
        run(
            [
                str(WEBBY),
                "preview-url",
                str(index),
                str(ASSET_DIR / "webby-docs-and-apps-grid.webp"),
                "--force",
                "--width",
                "920",
                "--height",
                "900",
                "--timeout-secs",
                "12",
            ],
            env,
        )


if __name__ == "__main__":
    main()
