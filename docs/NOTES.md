## 2026-06-21 — Public Preview Asset Path

Goal: make generated preview images load on the public Cloudflare Pages bag.

Discovery:
- Public pages referenced `./.webby-previews/*.jpg`, but Cloudflare Pages returned the generated homepage HTML for those hidden-directory asset requests.
- The public bag also did not have a deployed preview directory, so cards fell through to the HTML fallback instead of an image response.

Decision:
- Move Webby's generated preview assets to the non-hidden `webby-previews/` directory.
- Update generated card manifests and the web component fallback preview base to use that non-hidden path.

Verification:
- Regenerated the public bag previews into `/projects/webby/bags/public/webby-previews/`.
- Served the public bag locally and confirmed `/webby-previews/kickoff-worldcup.jpg` returns `200`.
- `just check`

## 2026-06-19 — Remove Browse Mode

Goal: remove Webby's old Caddy-specific browse mode now that the homeserver homepage consumes `webby-cards.json` directly.

Decision:
- Keep generated Webby indexes as the default.
- Use `noIndex` for host pages that consume Webby card data and render their own homepage sections.
- Remove the `browse` index mode, the `gen-browse` command, and the Caddy browse template.
- Do not special-case or clean up `browse.html`; with browse mode gone it is just another standalone HTML app name.

Verification:
- `cargo fmt --check`
- `cargo test`

## 2026-06-19 — Shared Card Grid Web Component

Goal: make Webby's card UI reusable by `tools.home.ankitson.com`, generated bag indexes, and other static pages without duplicating DOM/CSS card logic.

Decision:
- Add a plain ES module web component, `<webby-card-grid>`, with Shadow DOM and bundled Webby card styles.
- Keep Rust as the data producer: it scans bag entries, writes `webby-card-grid.js`, and embeds normalized card JSON into generated index pages.
- Use CSS custom properties and Shadow DOM parts for page-level overrides instead of carrying separate page card styles.

Verification:
- `cargo fmt`
- `cargo test`
- Browser smoke checks confirmed the custom element upgraded and rendered cards on `home.ankitson.com` and `tools.home.ankitson.com`.

## 2026-06-19 — Stable Card Width Layout Option

Goal: keep homepage cards from slowly growing and then snapping smaller when a new responsive grid column fits.

Decision:
- Add an opt-in `stable-card-width` attribute to `<webby-card-grid>`.
- In stable mode, dense grids keep cards at `--webby-card-stable-width` and compute the column gap needed to span the available row width.
- Sparse grids still grow cards up to `--webby-card-max-width` so two-card rows do not turn into tiny cards separated by a giant gap.

Verification:
- `cargo test`
- Browser measurements on `home.ankitson.com` at 1280, 1440, 1520, 1600, and 1920px showed dense cards staying at 292px while columns changed.

## 2026-06-19 — Card Manifest Output Mode

Goal: let a host homepage reuse Webby app cards without serving a separate Webby index page.

Decision:
- Every deploy writes `webby-cards.json` with the same normalized card data used by the Webby component.
- Added bag-level `noIndex` plus CLI `--no-index` for cases where a host page consumes Webby card data.
- `noIndex` removes generated `index.html` and keeps only reusable card data plus `webby-card-grid.js`.
- Directory staging now skips local metadata/cache folders such as `.git`, `.wrangler`, `node_modules`, and `logs`.

Verification:
- `cargo test`
- Added coverage for `noIndex` output and metadata directory skipping.

## 2026-06-18 — Caddy Browse Tile Collapse

Goal: fix internal Caddy browse tiles collapsing after regenerating the internal homepage.

Discovery:
- The shared CSS sizes `.preview-link` with `aspect-ratio`, then absolutely positions `.preview` inside it.
- The Caddy browse JavaScript still generated `.preview` directly inside `.site` and used `.site-link`, so the tile had no sized preview box.

Change:
- Updated `templates/browse.html` to generate `.preview-link > .preview` and `.site-title`.
- Moved temp apps into the main grid for both static and Caddy indexes.
- Added a compact `temp` label in the tile caption instead of fading temp previews or rendering them in a second row.
- Extended the render test to assert those browse-template classes.

Verification:
- `just check`

## 2026-06-18 — GitHub Releases

Goal: add a maintainable release path for prebuilt webby binaries.

Decision:
- Use a tag-driven GitHub Actions workflow on `v*` tags.
- Build native binaries on Ubuntu and macOS runners.
- Publish packaged artifacts to a GitHub Release using the built-in `GITHUB_TOKEN`.

Verification:
- `just check`
- YAML parsed with Python's YAML parser.

## 2026-06-18 — Shot-Scraper Preview Backend

Goal: make page preview capture reliable for static documentation sites.

Discovery:
- Raw `google-chrome --headless=new --screenshot` repeatedly timed out on `internal/jobsearch-docs/index.html`, including with a 60 second Chrome timeout.
- The same page captured correctly through DevTools and through `uvx shot-scraper` in about one second.

Change:
- Replaced the direct Chrome process backend with `uvx shot-scraper`.
- Added a 2000ms pre-capture wait to let app fonts and first render settle.
- Added an optional app argument so one preview can be regenerated without recapturing the whole bag.
- Added a preview integration test using a fake `uvx` command.

Verification:
- `just check`
- `cargo run -- preview -b internal --force --timeout-secs 15`

## 2026-06-18 — Commit Prep

Goal: verify the current preview-template work and make the Cloudflare fallback commit-ready.

Change:
- Hardened the Cloudflare Pages deploy fallback from `npx wrangler` to `npx --yes wrangler`.
- Added an integration test that removes `wrangler` from `PATH` and verifies the `npx` invocation.

Verification:
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

## 2026-06-11 — Lightweight Preview Tiles

Goal: remove lag from the generated index without going back to a plain text directory.

Decision:
- Stop using live iframes as tile backgrounds. They load and execute every app on the index page, which is too expensive for a launcher.
- Use deterministic CSS preview art from each app name instead. The tile still has visual identity, but it does not fetch or run the target app until clicked.
- Keep hover/focus effects transform-only and add reduced-motion handling.

Verification:
- `cargo fmt --check`
- `cargo test`

## 2026-06-11 — Hover Performance Trim

Goal: explain and reduce the remaining mouseover lag on large generated index tiles.

Discovery:
- The Firefox profile `Firefox 2026-06-11 10.49 profile.json.gz` showed the current root page spending time in `Coalesced input move flusher [Event]` markers, with long stalls around 171-275ms.
- The same root page had 129 CSS transition markers, including non-compositor `border-*-color` transitions on `.site` and label spans.
- The profile also still contained an earlier root page with iframe child windows, confirming that hard refresh matters after regenerating `browse.html`, but the remaining hover jank was from CSS transitions on the lightweight page.

Change:
- Stopped animating large tile surfaces and preview pseudo-elements.
- Removed animated shadow, opacity, background-color, and border-color changes from tile hover.
- Kept only the small label transform and added tile paint/layout containment.

Verification:
- `cargo fmt --check`
- `cargo test`
- Reinstalled `webby` and regenerated `internal/browse.html` plus the local index.

## 2026-06-11 — Static Screenshot Previews

Goal: restore real visual previews without iframe runtime cost.

Decision:
- Add `webby preview -b <bag>` to capture static JPEG previews into `.webby-previews/`.
- Cards reference `.webby-previews/<app>.jpg` as their first background layer and retain the gradient as a fallback.
- Capture app files directly via `file://` instead of the internal HTTPS URL. Chrome headless repeatedly timed out or captured blank pages against `tools.home.ankitson.com`, while file captures produced real screenshots.
- Use Chrome's native `--screenshot` path with a bounded per-page timeout. CDP `Page.captureScreenshot` repeatedly hung in this environment.

Results:
- Captured 8 of 9 internal app previews. `sleep-tracker` still times out under headless Chrome and falls back to the gradient tile.
- Deployed regenerated `internal/browse.html`; Caddy serves the preview JPEGs from `/.webby-previews/`.

Verification:
- `cargo fmt --check`
- `cargo test`
- `webby deploy -b internal`
- `webby deploy -b local`

## 2026-06-10 — Preview Tile Index

Goal: make the generated webby index feel like a dense app launcher instead of a text-heavy directory.

Change:
- Removed the visible `webby` hero, index count, item numbering, and `tool` / `page` labels.
- Replaced list cards with responsive preview tiles that use each app URL in a lazy iframe background.
- Added hover and keyboard-focus animation while keeping the tile label compact and readable on mobile.

Verification:
- `cargo fmt --check`
- `cargo test`

## 2026-06-08 — Refresh Public Mirrors on List

Goal: fix `webby ls` on `main` when the internal bag is missing public symlinks.

Discovery:
- The internal listing only included public apps if `syncPublicLinks()` had already run during `pub` or `deploy`.
- A checkout or host with missing/stale public symlinks could make `webby ls` show internal-only results.

Change:
- `webby ls` now refreshes public mirrors before listing the internal bag.
- The sync step also removes stale symlinks that point into the public bag while leaving real internal apps alone.

## 2026-06-08 — Bag Flag CLI Convention

Goal: make bag selection consistent across commands and stop treating `--public` as a special CLI flag.

Change:
- `webby ls` with no flags now lists every configured bag.
- `webby ls --bag <name>` and `webby ls -b <name>` list one bag.
- `add`, `deploy`, `rm`, `open`, and `domain` use the same `--bag` / `-b` selector where bag selection applies.
- Help text, README, skill docs, and Justfile recipes now use the bag selector convention.

## 2026-06-08 — Rust OSS Provider Rewrite

Goal: turn the OSS provider experiment into a Rust CLI that is easier to install, starts with no config, and has tested provider deploy paths.

Key decisions:
- Replace the Bun/TypeScript CLI with a Cargo binary named `webby`.
- Make `local` the default bag for new users; `webby add ./app.html && webby serve` works with no config.
- Keep `-b` / `--bag` as the only bag selector across bag-aware commands.
- Keep built-in `local`, `tailnet`, `funnel`, and `public` bags, with optional `internal` Caddy compatibility from env.
- Test deploy command construction with fake `tailscale` and `wrangler` executables instead of live external services.

Next steps:
- Decide on release packaging and whether to publish binaries in addition to `cargo install`.
- Revisit Cloudflare domain attachment once the preferred Wrangler/API path is confirmed for OSS users.
