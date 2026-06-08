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
