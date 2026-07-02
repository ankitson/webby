<h1 align="center">webby</h1>

<p align="center">
  Publish a static site in one command.
</p>

<p align="center">
  <a href="https://crates.io/crates/webby-deploy"><img alt="Crates.io" src="https://img.shields.io/crates/v/webby-deploy"></a>
  <a href="https://github.com/ankitson/webby"><img alt="Rust" src="https://img.shields.io/badge/rust-2024-orange"></a>
  <a href="Cargo.toml"><img alt="License" src="https://img.shields.io/badge/license-MIT-blue"></a>
</p>

<p align="center">
  <img alt="A Webby launcher page with cards grouped into Documents and Applications rows, each with a real screenshot preview." src="docs/assets/webby-category-grouped-homepage.png" width="820">
</p>

Webby makes publishing static sites dead simple for humans and agents. Point it at an HTML file, a folder with an `index.html`, or a directory of Markdown files, and Webby hosts it on the provider you choose — local, Tailscale, Cloudflare Pages, Caddy, or a custom command — with a generated launcher page, screenshot cards, and card data other sites can embed.

## Install

```sh
cargo install webby-deploy
```

## Quickstart

Preview locally, with no config:

```sh
webby add ./site.html -b local          # a single HTML file
webby add ./dist -b local --name app    # or a built static folder
webby serve -b local
```

Open `http://localhost:8765/site.html`. Manage what's staged with `webby ls`,
`webby open <name>`, `webby rm <name>`, and `webby where`; `webby ls` without
`-b` lists every configured bag.

## Why Webby

I wanted an easy way to host static pages that works the same across multiple providers, gives agents a place to show you rich HTML, and integrates into more complex setups. Use Webby when the page is done and the remaining problem is distribution:

- **Agent output:** prototypes and reports you want to inspect, keep, or share
  after a session, without a deployment project per page.
- **Homeserver launcher:** dashboards, one-off tools, and personal apps in one
  bag with screenshots and a generated index instead of a folder listing.
- **Repo docs:** `webby docs ./docs -b internal` from any repo gives a navigable
  static docs app next to the tools it explains.
- **Temporary public demo:** push a folder through Tailscale Funnel for a
  short-lived public URL from the current machine.
- **Durable public mini-site:** publish the same artifact to Cloudflare Pages
  with `webby pub`.
- **Embedded catalog:** another homepage renders Webby's card data and
  screenshots without giving Webby the homepage.

Everything Webby writes is plain static files, so the result is easy to host,
cache, embed, or delete:

## The Mental Model

**Apps** are static artifacts: either one `.html` file, a folder with
`index.html`, or a generated docs app from Markdown.

**Bags** are named staging directories with a hosting provider attached. The
built-in bags are listed under [Private Or Public
Publishing](#private-or-public-publishing).

**Deploying** a bag regenerates its static index and card manifest, refreshes
missing or stale preview images, then asks the provider to make the bag
available at its URL.

Most usage is just:

```sh
webby add <file-or-folder> -b <bag>
webby deploy -b <bag>
```

For the `local` bag, use `webby serve` instead of `deploy`.

## Markdown Docs

Publish a Markdown directory as a static docs app:

```sh
webby docs ./docs -b local \
  --name project-docs \
  --title "Project Docs" \
  --property category=Documents

webby serve -b local
```

Webby scans Markdown files within `--depth` directories below the source root
(default `3`), renders them with a sidebar, rewrites in-root `.md` links to the
generated `.html` pages, and copies linked in-root assets up to
`--max-asset-size-mib` MiB each (default `25`).

If the directory has `index.md`, it becomes the docs homepage. Otherwise Webby
writes a generated homepage that links to the discovered pages. Optional YAML
frontmatter can set page `title`, `description`, `tags`, `type`, `resource`,
and `timestamp`.

## Card Metadata And Previews

Every generated bag includes:

- `index.html`: a static launcher page, unless disabled.
- `webby-cards.json`: normalized card data for every staged app.
- `webby-card-grid.js`: a reusable card-grid custom element for host pages.
- `webby-previews/*.webp`: optimized screenshots when preview capture succeeds.

Set card metadata while staging:

```sh
webby add ./network-audit.html -b internal \
  --title "Network Audit" \
  --description "Internal network and DNS audit notes." \
  --property category=Documents \
  --property kind=report
```

Or put metadata in the app itself:

```html
<script type="application/webby+json">
{
  "title": "Network Audit",
  "description": "Internal network and DNS audit notes.",
  "properties": {
    "category": "Documents",
    "kind": "report"
  }
}
</script>
```

For standalone apps, put that block in the `.html` file. For folder apps, put
it in `index.html`. If Webby metadata is absent, Webby falls back to the page
`<title>` and `<meta name="description">`.

Preview capture runs automatically on `add`, `docs`, `pub`, and `deploy`.
Pass `--no-preview` when you only want to refresh metadata/index files. Run an
explicit refresh with:

```sh
webby preview -b internal
webby preview project-docs -b internal --force
```

Preview capture uses `uvx shot-scraper` plus Pillow. Generated preview URLs get
a content hash query string when the image exists, so hosts can cache preview
images aggressively while still picking up changed thumbnails.

## Private Or Public Publishing

Built-in bags:

| Bag | Provider | Use it for | Command |
| --- | --- | --- | --- |
| `local` | Local HTTP server | Safe preview on your machine | `webby serve -b local` |
| `tailnet` | Tailscale Serve | Private HTTPS on your tailnet | `webby deploy -b tailnet` |
| `funnel` | Tailscale Funnel | Temporary public HTTPS from this machine | `webby deploy -b funnel` |
| `cf-pages` | Cloudflare Pages | Durable public HTTPS | `webby pub <path>` or `webby deploy -b cf-pages` |
| `internal` | Caddy compatibility | Existing internal static host | Added when `INTERNAL_URL` or `INTERNAL_DIR` is set |

Private Tailscale:

```sh
webby add ./dashboard -b tailnet
webby deploy -b tailnet
```

Temporary public Funnel:

```sh
webby add ./demo -b funnel
webby deploy -b funnel
```

Cloudflare Pages:

```sh
export CLOUDFLARE_ACCOUNT_ID=...
export CLOUDFLARE_API_TOKEN=...

webby pub ./landing-page --name landing-page
```

`webby pub` is shorthand for staging into the `cf-pages` bag and deploying it.
For legacy configs, an explicit `public` bag still wins; otherwise the old
`public` name is accepted as a compatibility alias for `cf-pages`.

## Embedding Webby In A Larger Site

Sometimes Webby should manage apps and card data, while another site owns the
homepage. Use `--no-index` for that:

```sh
webby add ./tool.html -b internal --no-index
webby deploy -b internal --no-index
```

That keeps `webby-cards.json`, `webby-card-grid.js`, and previews, but removes
the generated root `index.html`.

Host pages can consume `webby-cards.json` directly or use the emitted custom
element:

```html
<script type="module" src="/webby/webby-card-grid.js"></script>
<webby-card-grid src="/webby/webby-cards.json" group-by-property="category"></webby-card-grid>
```

For a generated Webby index that still needs shared site chrome, set
`indexChromeDir` in the bag config. Webby will inline optional `head.html` and
`body.html` fragments from that directory.

## Configuration

Start with:

```sh
webby init
```

This writes `~/.config/webby/config.json`. Override paths with:

- `WEBBY_CONFIG`: config file path.
- `WEBBY_DATA_DIR`: default storage root for built-in bags.
- `WEBBY_DEFAULT_BAG`: default bag label.
- `WEBBY_ENV`: optional `KEY=VALUE` env file to load before config.

Minimal custom bag example:

```json
{
  "defaultBag": "local",
  "bags": {
    "tools": {
      "dir": "~/Sites/tools",
      "host": {
        "type": "command",
        "url": "https://tools.example.com",
        "deploy": "rsync -a {dir}/ deploy@example.com:/var/www/tools/"
      }
    }
  }
}
```

Command providers can use `{dir}`, `{label}`, and `{url}` in `deploy`,
`afterAdd`, and `open` templates.

## Command Reference

```sh
webby add <path> [-b bag] [--name name] [--tmp] [--title T] [--description D] [--property K=V] [--no-index] [--no-preview]
webby docs <dir> [-b bag] [--name name] [--title T] [--description D] [--property K=V] [--depth N] [--max-asset-size-mib N] [--no-index] [--no-preview]
webby pub <path> [--name name] [--tmp] [--title T] [--description D] [--property K=V] [--no-index] [--no-preview]
webby deploy -b bag [--port N] [--no-index] [--no-preview]
webby serve [-b bag] [--port N] [--no-index]
webby preview [app] -b bag [--force] [--width PX] [--height PX] [--timeout-secs N]
webby preview-url <url-or-file> <output.webp> [--force] [--width PX] [--height PX] [--timeout-secs N]
webby ls [-b bag]
webby open <name> [-b bag]
webby rm <name> [-b bag]
webby domain <hostname> -b cf-pages
webby where
webby init [--force]
```

Run `webby <command> --help` for exact option descriptions.

## Development

```sh
just check
just install
```

Maintainers can publish this repository's docs through the shared docme/Webby
workflow:

```sh
just docs-deploy
```
