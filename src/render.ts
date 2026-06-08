// Static browse-page generator. The internal bag uses Caddy's live `browse`
// with internal/browse.html; this renders the same look for the public bag
// (Pages has no server-side file listing). Keep the two visually in sync —
// the STYLE block below is byte-identical with internal/browse.html.

export interface AppEntry {
  name: string; // canonical name (no extension, no trailing slash)
  isDir: boolean;
  href: string; // relative link from the index, e.g. ./foo/ or ./foo.html
  tmp: boolean; // name starts with "tmp"
}

const esc = (s: string) =>
  s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

function card(app: AppEntry, i: number): string {
  const display = app.tmp ? app.name.replace(/^tmp[-_]?/, "") || app.name : app.name;
  const meta = app.tmp ? "scratch" : app.isDir ? "tool" : "page";
  const num = String(i + 1).padStart(2, "0");
  return `        <a class="card" href="${esc(app.href)}">
          <span class="card-num">${num}</span>
          <span class="card-body">
            <span class="card-name">${esc(display)}</span>
            <span class="card-meta">${esc(meta)}</span>
          </span>
          <span class="card-go" aria-hidden="true">&#8599;</span>
        </a>`;
}

export function renderIndex(opts: {
  apps: AppEntry[];
  title: string;
  homeUrl?: string;
}): string {
  const tools = opts.apps.filter((a) => !a.tmp);
  const temp = opts.apps.filter((a) => a.tmp);

  const toolsHtml = tools.length
    ? tools.map(card).join("\n")
    : `        <p class="empty">Nothing here yet.</p>`;

  const back = opts.homeUrl
    ? `      <a class="back" href="${esc(opts.homeUrl)}">&#8592; Home</a>\n`
    : "";

  const kicker = `Index &middot; ${tools.length} ${tools.length === 1 ? "entry" : "entries"}`;

  const tempSection = temp.length
    ? `

    <section class="temp-section">
      <p class="rubric"><span class="tick"></span>Temp &middot; Scratch</p>
      <div class="grid">
${temp.map(card).join("\n")}
      </div>
    </section>`
    : "";

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${esc(opts.title)}</title>
  <link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23bb5a2c' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'><path d='M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z'/></svg>">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Fraunces:ital,opsz,wght@0,9..144,300;0,9..144,400;0,9..144,500;1,9..144,300;1,9..144,400;1,9..144,500&family=Hanken+Grotesk:wght@400;500;600&family=Spline+Sans+Mono:wght@400;500&display=swap" rel="stylesheet">
  <style>
${STYLE}
  </style>
</head>
<body>
  <div class="atmosphere"></div>
  <div class="grain"></div>
  <main class="container">
    <header class="masthead">
${back}      <p class="kicker">${kicker}</p>
      <h1>${esc(opts.title)}</h1>
      <div class="rule"></div>
    </header>

    <section>
      <div class="grid">
${toolsHtml}
      </div>
    </section>${tempSection}
  </main>
</body>
</html>
`;
}

// Shared stylesheet — kept byte-identical with internal/browse.html so the
// internal (Caddy) and public (Pages) listings look the same.
// Editorial contents-page: warm paper, Fraunces italic display, oversized
// serif index numerals, monospace micro-labels. Light by default, auto dark.
const STYLE = `    :root {
      --paper: #f4f1e9;
      --panel: #fbf9f3;
      --ink: #211e19;
      --ink-soft: #5b554b;
      --faint: #a8a093;
      --line: rgba(33, 30, 25, 0.13);
      --line-strong: rgba(33, 30, 25, 0.28);
      --accent: #bb5a2c;
      --accent-wash: rgba(187, 90, 44, 0.07);
      --shadow: 0 18px 40px -24px rgba(54, 38, 24, 0.4);
      --grain: 0.5;
    }
    @media (prefers-color-scheme: dark) {
      :root {
        --paper: #110f0d;
        --panel: rgba(255, 255, 255, 0.028);
        --ink: #ece5d8;
        --ink-soft: #b3ac9f;
        --faint: #6c655a;
        --line: rgba(255, 255, 255, 0.1);
        --line-strong: rgba(255, 255, 255, 0.22);
        --accent: #e1894d;
        --accent-wash: rgba(225, 137, 77, 0.1);
        --shadow: 0 18px 44px -22px rgba(0, 0, 0, 0.8);
        --grain: 0.16;
      }
    }
    * { margin: 0; padding: 0; box-sizing: border-box; }
    html { -webkit-text-size-adjust: 100%; }
    body { font-family: 'Hanken Grotesk', system-ui, sans-serif; background: var(--paper); color: var(--ink-soft); min-height: 100vh; -webkit-font-smoothing: antialiased; text-rendering: optimizeLegibility; }
    .atmosphere { position: fixed; inset: 0; z-index: 0; pointer-events: none;
      background:
        radial-gradient(ellipse 50% 40% at 8% -5%, var(--accent-wash) 0%, transparent 60%),
        radial-gradient(ellipse 45% 45% at 105% 105%, var(--accent-wash) 0%, transparent 55%); }
    .grain { position: fixed; inset: 0; z-index: 0; pointer-events: none; opacity: var(--grain); mix-blend-mode: multiply;
      background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='160' height='160'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E"); }
    @media (prefers-color-scheme: dark) { .grain { mix-blend-mode: screen; } }
    .container { position: relative; z-index: 1; max-width: 1240px; margin: 0 auto; padding: clamp(3rem, 7vw, 6rem) clamp(1.25rem, 4vw, 3.5rem) 6rem; }
    .masthead { margin-bottom: clamp(2.5rem, 5vw, 4rem); animation: rise 0.7s cubic-bezier(0.2, 0.7, 0.2, 1) both; }
    .back { display: inline-flex; align-items: center; gap: 0.4rem; font-family: 'Spline Sans Mono', monospace; font-size: 0.7rem; color: var(--faint); text-decoration: none; letter-spacing: 0.14em; text-transform: uppercase; margin-bottom: 1.8rem; transition: color 0.2s; }
    .back:hover { color: var(--accent); }
    .kicker { font-family: 'Spline Sans Mono', monospace; font-size: 0.72rem; letter-spacing: 0.32em; text-transform: uppercase; color: var(--faint); margin-bottom: 1.1rem; }
    h1 { font-family: 'Fraunces', Georgia, serif; font-optical-sizing: auto; font-size: clamp(3.2rem, 9vw, 6rem); font-weight: 300; font-style: italic; color: var(--ink); line-height: 0.92; letter-spacing: -0.025em; }
    .rule { height: 1px; background: var(--line-strong); margin-top: clamp(1.6rem, 3vw, 2.4rem); }
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, 300px), 1fr)); gap: clamp(0.6rem, 1.2vw, 1rem); }
    a.card { position: relative; display: flex; align-items: center; gap: 1.15rem; padding: 1.5rem 1.6rem; background: var(--panel); border: 1px solid var(--line); border-radius: 4px; text-decoration: none; color: var(--ink-soft); overflow: hidden; transition: border-color 0.25s, transform 0.25s, box-shadow 0.3s; animation: rise 0.6s cubic-bezier(0.2, 0.7, 0.2, 1) both; }
    a.card::before { content: ""; position: absolute; left: 0; top: 0; bottom: 0; width: 2px; background: var(--accent); transform: scaleY(0); transform-origin: top; transition: transform 0.28s cubic-bezier(0.2, 0.7, 0.2, 1); }
    a.card:hover { border-color: var(--line-strong); transform: translateY(-3px); box-shadow: var(--shadow); }
    a.card:hover::before { transform: scaleY(1); }
    .card-num { font-family: 'Fraunces', Georgia, serif; font-style: italic; font-weight: 400; font-size: 1.9rem; line-height: 1; color: var(--faint); flex-shrink: 0; min-width: 2.2rem; transition: color 0.25s; font-variant-numeric: tabular-nums; }
    a.card:hover .card-num { color: var(--accent); }
    .card-body { display: flex; flex-direction: column; gap: 0.3rem; min-width: 0; }
    .card-name { font-size: 1.08rem; font-weight: 500; color: var(--ink); letter-spacing: -0.01em; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .card-meta { font-family: 'Spline Sans Mono', monospace; font-size: 0.66rem; font-weight: 400; letter-spacing: 0.16em; text-transform: uppercase; color: var(--faint); }
    .card-go { margin-left: auto; flex-shrink: 0; font-size: 1.1rem; color: var(--faint); transform: translate(-2px, 2px); opacity: 0; transition: opacity 0.25s, transform 0.25s, color 0.25s; }
    a.card:hover .card-go { opacity: 1; transform: translate(0, 0); color: var(--accent); }
    .temp-section { margin-top: clamp(2.5rem, 5vw, 3.5rem); }
    .rubric { display: flex; align-items: center; gap: 0.6rem; font-family: 'Spline Sans Mono', monospace; font-size: 0.68rem; font-weight: 400; letter-spacing: 0.2em; text-transform: uppercase; color: var(--faint); margin-bottom: 1.1rem; }
    .tick { width: 14px; height: 1px; background: var(--line-strong); }
    .empty { font-family: 'Fraunces', Georgia, serif; font-style: italic; font-size: 1.1rem; color: var(--faint); padding: 1.5rem 0; }
    @keyframes rise { from { opacity: 0; transform: translateY(14px); } to { opacity: 1; transform: translateY(0); } }
    @media (prefers-reduced-motion: reduce) { * { animation: none !important; } }
    @media (max-width: 560px) { a.card { padding: 1.25rem 1.3rem; } .card-num { font-size: 1.6rem; min-width: 1.9rem; } }`;
