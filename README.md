# webby

Drop a simple HTML app and serve it — **internally** via the home Caddy
(html-bag) or **publicly** via Cloudflare Pages. One tool, two backends.

A *bag* is a directory of standalone HTML apps plus a way to serve it:

| bag        | backend | served at                       | reach                    |
| ---------- | ------- | ------------------------------- | ------------------------ |
| `internal` | Caddy   | `tools.home.example.com`       | LAN / Tailscale, instant |
| `public`   | Pages   | `mini.example.com`             | public internet, always-on |

An "app" is either a **folder with `index.html`** (plus assets) or a
**standalone `.html` file**. Anything whose name starts with `tmp` is treated
as scratch: shown under a separate *Temp* heading and gitignored.

## Install

webby is a [Bun](https://bun.sh) CLI (it uses Bun APIs, so it needs `bun` — not
`node`/`npx`). There's no binary to download; install straight from the repo:

```sh
bun install -g github:ankitson/webby     # puts `webby` on your PATH
# one-off, no install (the bunx / npx equivalent):
bunx github:ankitson/webby where
```

Configure it from the environment (see [Configuration](#configuration)), then:

## Usage

```sh
webby add <path> [--name N] [--tmp] [--public]   # stage an app into a bag
webby pub <path> [--name N] [--tmp]              # add to public bag + deploy
webby deploy [--public]                          # regenerate index + deploy (pages bags)
webby ls   [--public | --bag <name>]             # list apps in a bag
webby rm   <name> [--public]                     # remove an app
webby open <name> [--public]                     # print/open an app URL
webby domain <hostname>                          # attach a custom domain to the public bag
```

- **Internal** is a plain file copy into the `internal/` dir — live immediately
  via Caddy's live mount, no deploy step.
- **Public** copies into `public/`, regenerates a static browse `index.html`,
  and runs `wrangler pages deploy public/` — deploy is just "push the directory".
- The internal listing shows **both** bags: every public app is mirrored into
  `internal/` as a relative symlink (`../public/<app>`), so the tools host lists
  internal + public apps in one flat page. `public/` stays the single source of
  truth; the symlinks are maintained automatically on `pub` / `deploy`.

### Examples

```sh
webby add ./clock.html                 # → tools.home.example.com/clock.html (instant)
webby add ./dashboard --tmp            # scratch folder app, internal
webby pub ./lissajous --name lissajous # publish a folder app to mini.example.com
webby deploy --public                  # re-push the whole public bag
```

## Configuration

webby reads everything from the **environment** — nothing is baked into the
code. Export the keys below, or point `$WEBBY_ENV` at a `KEY=VALUE` file (handy
with `op inject`/`op run`). When running from a clone, an in-repo `.env.secret`
(gitignored) is loaded automatically; see `.env.secret.example` for the keys.

- `CF_ACCOUNT_ID` — Cloudflare account that owns the Pages project
- `CF_TOKEN_REF` — 1Password reference for the API token (needs **Pages: Edit**);
  read via `op read` at deploy time, never written to disk
- `INTERNAL_URL`, `PUBLIC_URL` — the domains each bag is served at
- `INTERNAL_DIR`, `PUBLIC_DIR`, `PUBLIC_PROJECT` — bag paths / Pages project name

```sh
export CF_ACCOUNT_ID=… CF_TOKEN_REF='op://…' INTERNAL_URL=… PUBLIC_URL=…
export INTERNAL_DIR=… PUBLIC_DIR=… PUBLIC_PROJECT=webby
webby where      # prints the resolved bags
```

## How public hosting works

`mini.example.com` is a **custom domain on a Cloudflare Pages project**, so the
apps live on Cloudflare's edge — not on the home server (which has no public
ingress and suspends nightly). `webby deploy` pushes the `public/` directory to
Pages; Cloudflare serves it globally with automatic TLS.

The webby token is Pages-scoped only. Attaching a custom domain also needs a
DNS record (`CNAME <host> → <project>.pages.dev`, proxied); creating that
requires a DNS-edit token on the zone, done once per domain.

## Notes

- Built with Bun + TypeScript. `wrangler` is invoked via `bunx`.
- The old `html-bag` repo has been merged in: its apps now live in `internal/`
  and the home Caddy mounts `internal/` (+ `public/`) directly. `/projects/html-bag`
  is vestigial.
