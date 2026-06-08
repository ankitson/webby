## 2026-06-08 — Refresh Public Mirrors on List

Goal: fix `webby ls` on `main` when the internal bag is missing public symlinks.

Discovery:
- The internal listing only included public apps if `syncPublicLinks()` had already run during `pub` or `deploy`.
- A checkout or host with missing/stale public symlinks could make `webby ls` show internal-only results.

Change:
- `webby ls` now refreshes public mirrors before listing the internal bag.
- The sync step also removes stale symlinks that point into the public bag while leaving real internal apps alone.
