## 2026-06-19

### Remove Browse Mode

Removed:
- `browse` as a supported `indexMode`.
- The `webby gen-browse` CLI command.
- The Caddy browse template and its render tests.
- The Justfile recipe for regenerating `browse.html`.

Modified:
- Caddy-hosted bags now default to normal `page` output unless configured as `manifest-only`.
- `browse.html` is no longer reserved or cleaned up; it behaves like any other standalone HTML app.

Why:
- The current homeserver homepage consumes `webby-cards.json`; Webby should not carry a Caddy directory-listing integration as a first-class mode.

### Shared Card Grid Web Component

Added:
- `templates/webby-card-grid.js`, a self-contained `<webby-card-grid>` custom element with bundled Webby card styles.
- `src/cards.rs`, which normalizes Webby `AppEntry` values into reusable card data.

Modified:
- Generated Webby indexes now embed card JSON and let `<webby-card-grid>` render the cards.
- Bag generation writes `webby-card-grid.js` next to generated assets.

Why:
- Card presentation should be reusable across generated Webby pages and homeserver pages without copying card markup and CSS.

### Stable Card Width Layout

Added:
- Optional `stable-card-width` mode for `<webby-card-grid>`.
- `--webby-card-stable-width` custom property for pages that want dense rows to avoid card-width snapping at responsive breakpoints.

Modified:
- The component now runs a resize-aware layout pass in stable mode: dense categories keep stable card widths and distribute extra width into gaps; sparse categories fill available width until the configured max card width.

Why:
- CSS Grid `repeat(auto-fit, minmax(..., 1fr))` makes cards grow continuously and then snap smaller when an additional column fits. The homepage should keep row coverage without that card-size snap.

### Card Manifest Output

Added:
- `webby-cards.json` generation for every bag deploy.
- Bag `indexMode` config with `page` and `manifest-only`.
- Integration tests for `manifest-only` and skipped metadata folders.

Modified:
- All bags default to `page` unless configured as `manifest-only`.
- Directory staging skips `.git`, `.wrangler`, `.DS_Store`, `node_modules`, and `logs`.

Why:
- A host homepage can now consume Webby cards as data and render them in its own UI without keeping a separate Webby listing page.

## 2026-06-18

### Crates.io Package Metadata

Added:
- Crates.io package metadata in `Cargo.toml`, including license, repository, homepage, documentation, readme, keywords, and categories.
- Package excludes for repo-local release workflows, bags, logs, and skills so `cargo publish` ships only the crate-relevant files.

Modified:
- Renamed the Cargo package to `webby-deploy` because the `webby` crate name is already taken on crates.io.
- README install instructions now use `cargo install webby-deploy` while keeping the installed binary named `webby`.

Fixed:
- Rust 2024 compatibility for `.env.secret` loading by isolating the now-unsafe process environment mutation during startup.

Why:
- `cargo publish` requires complete package metadata and should not include local generated site artifacts.

### Caddy Browse Tile Markup

Fixed:
- Caddy browse template now emits `.preview-link` and `.site-title` markup to match the shared tile CSS.

Modified:
- Temp apps now render alongside regular apps in the same grid with a compact `temp` label near the title.
- Temp app previews are no longer faded or moved into a separate row.

Why:
- The old Caddy-only markup put `.preview` directly inside `.site`, so the absolute-positioned preview had no sized parent and tiles collapsed to tiny label-height rows.

### GitHub Release Workflow

Added:
- GitHub Actions release workflow for `v*` tags.
- Native Linux and macOS binary packaging.
- `just release <version>` helper for pushing release tags.

Modified:
- README now documents prebuilt GitHub release binaries and tag-driven publishing.

Why:
- Users should have a binary download path in addition to `cargo install --git`.

### Shot-Scraper Preview Capture

Modified:
- `webby preview` now delegates page screenshots to `uvx shot-scraper`.
- Preview capture waits 2000ms before taking each screenshot.
- `webby preview [APP] -b <bag>` can refresh one app preview instead of the whole bag.
- Removed the direct Rust-managed Chrome screenshot process and file URL encoding path.

Added:
- Integration coverage that verifies `webby preview` invokes `uvx shot-scraper` with the requested size and timeout.

Why:
- Chrome's native `--headless --screenshot` path can hang indefinitely on fast-rendering MkDocs pages such as `jobsearch-docs`, while `shot-scraper` captures them reliably through Playwright.

### Cloudflare Wrangler Fallback

Modified:
- Cloudflare Pages deploys now fall back to `npx --yes wrangler` when a `wrangler` binary is not already on `PATH`.

Added:
- Integration coverage for the `npx` fallback path.

Why:
- The fallback should be non-interactive and test-covered so deploys do not hang waiting for an install prompt.

## 2026-06-11

### Static Screenshot Previews

Added:
- `webby preview -b <bag>` captures static JPEG card previews into `.webby-previews/`.
- `just preview-internal` runs the preview capture for the internal Caddy bag.

Modified:
- Generated cards use `.webby-previews/<app>.jpg` as a background image over the existing gradient fallback.
- Caddy browse rendering skips dot-directories like `.webby-previews/`.
- Preview capture loads app files directly with Chrome's native screenshot mode and a bounded timeout.

Why:
- Real screenshots give the visual signal the iframe version had, without loading every app inside the index at runtime.

### Hover Performance Trim

Modified:
- Removed large-surface hover transitions from generated tiles: card transform, preview pseudo-element transforms, overlay opacity animation, background-color animation, shadow animation, and border-color animation.
- Added paint/layout containment to each tile.
- Kept only the small label lift as an animated hover effect.

Why:
- A Firefox profile showed hover jank dominated by `Coalesced input move flusher` stalls and many CSS transition markers, including non-compositor border-color transitions on large cards.

### Lightweight Preview Tiles

Modified:
- Replaced iframe-backed index previews with CSS-only visual previews keyed from app names.
- Updated the Caddy browse template to build the same lightweight tiles without loading each app in an iframe.
- Reduced hover work to small transform and opacity changes, with reduced-motion support.

Why:
- Live iframe previews made the index laggy by loading every hosted app inside the launcher. Static CSS previews keep visual variety while making the UI much snappier.

## 2026-06-10

### Preview Tile Index

Modified:
- Reworked generated bag indexes into a full-width responsive grid of preview-backed site tiles.
- Removed visible index header text, entry counts, numbering, and app type labels from the generated listing.
- Added hover and focus animation for preview tiles.

Why:
- The index should act as a compact launcher for hosted apps, with the apps themselves as the primary visual signal.

## 2026-06-08

### Internal Listing Public Mirrors

Modified:
- `webby ls` refreshes public-to-internal symlinks before listing the internal bag.
- Public mirror sync prunes stale public symlinks without clobbering real internal apps.

Why:
- The default internal listing should include public apps even when a host or checkout starts without the symlinks already in place.

### Bag Flag CLI Convention

Modified:
- `webby ls` lists all bags by default.
- `--bag <name>` and `-b <name>` select a specific bag for `ls` and other bag-aware commands.
- Removed `--public` from CLI parsing, help text, README, skill docs, and Justfile examples.

Why:
- Public should be a normal bag name, not a dedicated CLI flag.

### Rust Provider Rewrite

Added:
- Cargo project and Rust binary entrypoint.
- Rust config, app staging, index rendering, provider execution, and local server modules.
- Integration tests for deploy paths across local, Caddy, Tailscale Serve, Tailscale Funnel, Cloudflare Pages, and command providers.

Removed:
- Bun/TypeScript CLI implementation, TypeScript config, and Bun lockfile from the core tool.

Modified:
- README, skill docs, env example, Justfile, and gitignore for the Rust workflow.

Why:
- The OSS version should be simple to install with Cargo, useful with zero config, and backed by provider deploy tests.
