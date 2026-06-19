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
