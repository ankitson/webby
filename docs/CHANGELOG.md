## 2026-06-22

### Markdown Docs App Generation

Added:
- `webby docs <dir>` for generating a static docs app from a directory of Markdown files and staging it into a bag.
- Native Markdown rendering with tables, task lists, strikethrough, sidebar navigation, generated docs CSS, and one static HTML page per Markdown file.
- Permissive YAML frontmatter support for display metadata, including OKF-style `type`, `title`, `description`, `resource`, `tags`, and `timestamp` fields.
- Local in-root link rewriting from `.md` links to generated `.html` pages.
- Linked in-root asset copying with `--max-asset-size-mib`.
- Depth-limited discovery with heavy/generated directory skipping.

Modified:
- The generated docs app embeds Webby metadata in its `index.html`, so it appears in `webby-cards.json` like any other app.

Why:
- Markdown docs should be publishable through Webby without moving them out of their owning repo or requiring a separate MkDocs/Python workflow.
- The implementation follows the OKF spirit of Markdown plus frontmatter without making Webby an OKF validator or knowledge platform.

### Mini Page Load Flash

Modified:
- Added generated `<link rel="modulepreload">` for the card-grid module.
- Added generated high-priority image preloads for the first preview tiles.
- Added initial reserved height for `webby-card-grid:not(:defined)`.

Why:
- The mini page should paint with its intended background and have preview images available before the custom element renders its cards.

### App-Owned Card Metadata

Added:
- Embedded app metadata support via `<script type="application/webby+json">` in standalone app HTML files and folder app `index.html` files.
- Generic `properties` output in generated `webby-cards.json`.
- Standard HTML metadata fallbacks for card `title` and `description` from `<title>` and `<meta name="description">`.
- `<webby-card-grid group-by-property="...">` support for consumers that want to group by arbitrary properties.
- Staging-time metadata flags on `webby add` and `webby pub`: `--title`, `--description`, and repeatable `--property KEY=VALUE`.

Modified:
- Generated cards now mirror string `properties.category` to the legacy top-level `category` field for compatibility.
- The reusable card component preserves `item.properties` during normalization.
- Metadata flags write into the staged app HTML before manifest generation, keeping the staged app self-contained.

Why:
- Card metadata should travel with the app artifact instead of living in a bag-level registry or manual generated-manifest edit.
- Category grouping is now a consumer convention over generic app properties, not a dedicated Webby storage concept.

## 2026-06-21

### Public Preview Asset Path

Modified:
- Preview captures now write to `webby-previews/` instead of `.webby-previews/`.
- Generated card manifests and the card-grid default preview base now reference `webby-previews/*.jpg`.
- The generated public card manifest and preview directory are ignored with the other public deploy artifacts.
- Updated preview-path tests and README text.

Why:
- Cloudflare Pages does not reliably serve the hidden `.webby-previews/` asset path; missing image requests were falling back to the public homepage HTML.

## 2026-06-19

### Remove Browse Mode

Removed:
- `browse` as a supported generated-index behavior.
- The `webby gen-browse` CLI command.
- The Caddy browse template and its render tests.
- The Justfile recipe for regenerating `browse.html`.

Modified:
- Caddy-hosted bags now default to normal index output unless configured with `noIndex`.
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
- Bag `noIndex` config and CLI `--no-index` override for hosts that do not want a root `index.html`.
- Integration tests for `noIndex` and skipped metadata folders.

Modified:
- All bags write `index.html` by default unless configured with `noIndex` or invoked with `--no-index`.
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

### Firefox Tab-Switch Flash

Modified:
- Added a top-of-head critical background/color-scheme block to generated webby indexes.
- Moved configured index chrome before module/image preloads so host theme scripts can set `html[data-theme]` earlier.
- Regenerated the public bag index with the updated head order.

Why:
- Firefox could expose its blue/grey default canvas while crossing from the blog origin to the projects origin before the generated index CSS applied.

### Lightweight Preview Tiles

Modified:
- Replaced iframe-backed index previews with CSS-only visual previews keyed from app names.
- Updated the Caddy browse template to build the same lightweight tiles without loading each app in an iframe.
- Reduced hover work to small transform and opacity changes, with reduced-motion support.

Why:
- Live iframe previews made the index laggy by loading every hosted app inside the launcher. Static CSS previews keep visual variety while making the UI much snappier.

## 2026-06-21

### Shared Site Header

Added:
- Added generated `site-header.html` and `site-header.css` template partials.

Modified:
- The generated index now includes the shared personal-site header before the card grid.
- Removed the old public/internal bag nav from the index template.
- Corrected Justfile argument comments after confirming `just deploy -b public` is the working deploy invocation.

Why:
- The projects page should share navigation and visual chrome with the blog without fetching runtime header assets from another site.

### Shared Header Theme Parity

Modified:
- Updated the generated header partials to include the shared dark-mode toggle and script.
- Added explicit light/dark page variables for the mini index.
- Mirrored the document theme onto `webby-card-grid` and replaced its internal media-query theme switch with a host `data-theme` switch.
- Extended the render test to assert the theme toggle and grid theme sync script are present.

Why:
- The mini page should not render a light header beside the dark projects page, and the shared header should align with the blog across different root font sizes.

### Generic Index Chrome

Added:
- Added optional `indexChromeDir` bag config and `WEBBY_INDEX_CHROME_DIR` env support for inlining `head.html` and `body.html` fragments into generated indexes.
- Documented the generic chrome hook in the README.
- Added render and integration coverage for configured custom chrome fragments.

Removed:
- Removed personal-site generated header partials from webby core.

Modified:
- The default webby index no longer includes personal site chrome, but still mirrors `html[data-theme]` onto `webby-card-grid` when an external chrome fragment provides a theme toggle.
- Restored system dark-mode behavior for the generic index when no custom theme is set.

Why:
- Webby should provide an extension point for host-site chrome without knowing or committing a specific person's header.

### Host Theme Tokens

Modified:
- Mapped generated index colors to optional `--site-page-*` host theme tokens with webby palette fallbacks.
- Changed `webby-card-grid` to consume inherited `--webby-theme-*` variables so card colors can follow host-provided theme tokens across the shadow DOM boundary.

Why:
- A host site can share its color scheme with webby's index while webby remains generic by default.

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

## 2026-06-23

### Optimized Preview Images

Modified:
- `webby preview` now captures through a temporary JPEG and writes optimized `.webp` preview files by default.
- Generated card manifests, index preloads, and the card-grid fallback now point at `webby-previews/*.webp`.
- Preview capture tests now cover both the screenshot and optimization tool invocations.

Why:
- The blog had a separate post-processing step for smaller WebP preview assets; Webby should generate those optimized card images directly.

## 2026-06-24

### Static First Generated Indexes

Modified:
- Generated bag indexes now emit static card/link/image markup directly in `index.html`.
- Removed the default index dependency on `<webby-card-grid>` upgrade, embedded card JSON, and module import before cards appear.
- Added image dimensions, decoding/loading/fetch-priority attributes, and integration coverage for static card markup.
- Kept `webby-card-grid.js` as a reusable embed asset while trimming unused gradient color hashing from the component.

Why:
- The primary cards should be discoverable and laid out during the initial HTML parse so generated pages feel snappy and avoid JS-driven layout swaps.
