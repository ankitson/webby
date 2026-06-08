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
