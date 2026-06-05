// Static browse-page generator. Mirrors the html-bag Caddy `browse` template,
// but the card list is rendered here (Pages has no server-side file listing).

export interface AppEntry {
  name: string; // canonical name (no extension, no trailing slash)
  isDir: boolean;
  href: string; // relative link from the index, e.g. ./foo/ or ./foo.html
  tmp: boolean; // name starts with "tmp"
}

const esc = (s: string) =>
  s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

const FOLDER_ICON = `<svg viewBox="0 0 24 24"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>`;

function card(app: AppEntry): string {
  const display = app.tmp ? app.name.replace(/^tmp[-_]?/, "") || app.name : app.name;
  const desc = app.tmp ? (app.isDir ? "scratch" : "scratch html") : app.isDir ? "HTML tool" : "HTML file";
  return `        <a class="card" href="${esc(app.href)}">
          <div class="card-icon">${FOLDER_ICON}</div>
          <div class="card-text">
            <div class="card-name">${esc(display)}</div>
            <div class="card-desc">${esc(desc)}</div>
          </div>
        </a>`;
}

export function renderIndex(opts: {
  apps: AppEntry[];
  title: string;
  subtitle: string;
  homeUrl?: string;
}): string {
  const tools = opts.apps.filter((a) => !a.tmp);
  const temp = opts.apps.filter((a) => a.tmp);

  const toolsHtml = tools.length
    ? tools.map(card).join("\n")
    : `        <p class="empty">Nothing here yet.</p>`;
  const tempHtml = temp.map(card).join("\n");

  const back = opts.homeUrl
    ? `    <a class="back" href="${esc(opts.homeUrl)}">&larr; Home</a>\n`
    : "";

  const tempSection = temp.length
    ? `
    <section class="temp-section" style="animation: slideUp 0.7s ease both; animation-delay: 0.25s;">
      <div class="category-label"><span class="dot temp"></span>Temp &middot; Scratch</div>
      <div class="grid">
${tempHtml}
      </div>
    </section>`
    : "";

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${esc(opts.title)}</title>
  <link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23d4a574' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'><path d='M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z'/></svg>">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Fraunces:ital,opsz,wght@0,9..144,300;0,9..144,400;1,9..144,300&family=Outfit:wght@300;400;500;600&display=swap" rel="stylesheet">
  <style>
    :root {
      --bg: #0a0a0c;
      --card: rgba(255, 255, 255, 0.025);
      --card-hover: rgba(255, 255, 255, 0.055);
      --card-border: rgba(255, 255, 255, 0.06);
      --card-border-hover: rgba(255, 255, 255, 0.13);
      --accent: #d4a574;
      --accent-dim: rgba(180, 130, 80, 0.5);
      --text: #c8c2b8;
      --text-muted: #6b6560;
      --heading: #e8e2d8;
    }
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { font-family: 'Outfit', system-ui, sans-serif; background: var(--bg); color: var(--text); min-height: 100vh; overflow-x: hidden; }
    .atmosphere { position: fixed; inset: 0; z-index: 0;
      background:
        radial-gradient(ellipse 70% 50% at 15% 15%, rgba(180, 120, 60, 0.07) 0%, transparent 60%),
        radial-gradient(ellipse 50% 70% at 85% 85%, rgba(100, 80, 150, 0.045) 0%, transparent 55%),
        radial-gradient(ellipse 60% 40% at 50% 40%, rgba(180, 140, 80, 0.025) 0%, transparent 70%); }
    .container { position: relative; z-index: 2; max-width: 860px; margin: 0 auto; padding: 5rem 2rem 6rem; }
    .header { margin-bottom: 4rem; animation: fadeIn 1s ease; }
    h1 { font-family: 'Fraunces', Georgia, serif; font-size: 3rem; font-weight: 300; font-style: italic; color: var(--heading); line-height: 1.1; letter-spacing: -0.02em; }
    .subtitle { font-size: 0.75rem; color: var(--text-muted); margin-top: 0.75rem; font-weight: 400; letter-spacing: 0.14em; text-transform: uppercase; }
    .rule { width: 36px; height: 1px; background: var(--accent-dim); margin-top: 2rem; }
    .back { display: inline-flex; align-items: center; gap: 0.4rem; font-size: 0.75rem; color: var(--text-muted); text-decoration: none; letter-spacing: 0.1em; text-transform: uppercase; margin-bottom: 1.5rem; transition: color 0.2s; }
    .back:hover { color: var(--accent); }
    .category-label { font-size: 0.65rem; font-weight: 500; letter-spacing: 0.16em; text-transform: uppercase; color: var(--text-muted); margin-bottom: 0.75rem; display: flex; align-items: center; gap: 0.6rem; }
    .dot { width: 5px; height: 5px; border-radius: 50%; background: var(--accent); }
    .dot.temp { background: var(--text-muted); }
    .temp-section { margin-top: 2.75rem; }
    .grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.5rem; }
    @media (max-width: 720px) { .grid { grid-template-columns: repeat(2, 1fr); } h1 { font-size: 2.2rem; } .container { padding: 3.5rem 1.25rem 4rem; } }
    @media (max-width: 440px) { .grid { grid-template-columns: 1fr; } }
    a.card { display: flex; align-items: center; gap: 0.75rem; padding: 0.8rem 0.9rem; background: var(--card); border: 1px solid var(--card-border); border-radius: 10px; text-decoration: none; color: var(--text); transition: background 0.2s, border-color 0.25s, transform 0.2s; backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); }
    a.card:hover { background: var(--card-hover); border-color: var(--card-border-hover); transform: translateY(-2px); }
    .card-icon { width: 34px; height: 34px; flex-shrink: 0; display: flex; align-items: center; justify-content: center; border-radius: 8px; background: rgba(255, 255, 255, 0.035); transition: background 0.2s; }
    a.card:hover .card-icon { background: rgba(255, 255, 255, 0.06); }
    .card-icon svg { width: 16px; height: 16px; stroke: var(--text-muted); stroke-width: 1.5; fill: none; stroke-linecap: round; stroke-linejoin: round; transition: stroke 0.2s; }
    a.card:hover .card-icon svg { stroke: var(--accent); }
    .card-text { min-width: 0; }
    .card-name { font-size: 0.85rem; font-weight: 500; color: var(--heading); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; transition: color 0.2s; }
    a.card:hover .card-name { color: #fff; }
    .card-desc { font-size: 0.7rem; color: var(--text-muted); font-weight: 300; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .empty { color: var(--text-muted); font-size: 0.9rem; font-style: italic; padding: 2rem 0; }
    @keyframes fadeIn { from { opacity: 0; transform: translateY(-8px); } to { opacity: 1; transform: translateY(0); } }
    @keyframes slideUp { from { opacity: 0; transform: translateY(20px); } to { opacity: 1; transform: translateY(0); } }
  </style>
</head>
<body>
  <div class="atmosphere"></div>
  <div class="container">
${back}    <header class="header">
      <h1>${esc(opts.title)}</h1>
      <p class="subtitle">${esc(opts.subtitle)}</p>
      <div class="rule"></div>
    </header>

    <section style="animation: slideUp 0.7s ease both; animation-delay: 0.15s;">
      <div class="category-label"><span class="dot"></span>Apps</div>
      <div class="grid">
${toolsHtml}
      </div>
    </section>${tempSection}
  </div>
</body>
</html>
`;
}
