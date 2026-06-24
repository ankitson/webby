# webby

Drop a static HTML app into a bag and get a URL.

webby is for tiny tools, one-off dashboards, agent-made prototypes, and
standalone HTML files. It works with no config on localhost, and can also
activate tailnet, temporary public, durable public, or custom hosting.

## Install

```sh
cargo install webby-deploy
```

## Start Fast

No config required:

```sh
webby add ./clock.html
webby serve
```

That stages `clock.html` in the `local` bag and serves the bag at
`http://localhost:8765`.

## Commands

A bag is a named directory plus a hosting provider. Apps are copied into a bag,
then the provider decides how that directory gets a URL.

```sh
webby add <path> [--name N] [--tmp] [--title T] [--description D] [--property K=V] [-b BAG]
webby docs <dir> [--name N] [--tmp] [--title T] [--description D] [--property K=V] [-b BAG]
webby pub <path> [--name N] [--tmp] [--title T] [--description D] [--property K=V]
webby deploy -b BAG
webby preview [APP] -b BAG [--force]
webby serve [-b BAG] [--port N]
webby ls [-b BAG]
webby rm <name> [-b BAG]
webby open <name> [-b BAG]
webby domain <hostname> -b BAG
webby where
webby init
```

`webby ls` lists all bags by default. Use `-b` / `--bag` to select one bag.

An app is a folder with `index.html` or a standalone `.html` file. Names that
start with `tmp` are shown under the Temp section in generated indexes.

`webby preview -b BAG` captures static card previews into the bag's
`webby-previews/` directory as optimized WebP images. It shells out to
`uvx shot-scraper` for capture and `uvx --with pillow python` for resizing and
conversion, skips existing previews unless `--force` is passed, and keeps
generated indexes fast by serving images instead of live iframes. Pass an app
name, for example `webby preview jobsearch-docs -b internal --force`, to
refresh one preview.

Every deploy writes `webby-cards.json` next to the generated assets. That JSON
contains the same card data used by Webby's index component, so another page can
embed a Webby bag without scraping a directory listing.

## App Metadata

Apps can carry their own card metadata. For a standalone app, put it in the
`.html` file. For a folder app, put it in `index.html`.

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

`title` and `description` customize the generated card. If they are omitted,
Webby falls back to the page's `<title>` and `<meta name="description">` when
available.

`properties` is an app-defined key/value object. Webby copies it into
`webby-cards.json` without assigning meaning to the keys. Host pages can use
those properties however they want, for example grouping by
`properties.category`.

For compatibility with older card consumers, a string `properties.category`
also appears as the generated card's top-level `category` field.

You can also set metadata while staging an app. Webby writes the metadata into
the staged HTML app and then regenerates card data from that self-contained
artifact:

```sh
webby add ./network-audit.html -b internal \
  --title "Network Audit" \
  --description "Internal network and DNS audit notes." \
  --property category=Documents \
  --property kind=report
```

## Markdown Docs

Point Webby at a directory of Markdown files to generate a static docs app and
stage it into any bag:

```sh
webby docs ./docs -b internal \
  --name project-docs \
  --title "Project Docs" \
  --property category=Documents
```

Webby scans Markdown files within `--depth` directories below the source root
(default `3`), renders them with a sidebar, copies linked in-root assets up to
`--max-asset-size-mib` MiB each (default `25`), rewrites in-root `.md` links to
generated `.html` pages, and then regenerates the bag index/card manifest.

Markdown frontmatter is optional and permissive. `title` and `description`
control page display; OKF-style fields such as `type`, `resource`, `tags`, and
`timestamp` are shown when present. Unknown fields are tolerated.

If the directory has an `index.md`, that becomes the docs app homepage. If not,
Webby writes a generated homepage that links to the discovered pages. Raw HTML
inside Markdown is escaped by default. Links outside the selected root are left
unchanged and are not copied into the generated app.

## Provider Examples

Built-in bags:

| bag | provider | reach |
| --- | --- | --- |
| `local` | `local` | localhost preview |
| `tailnet` | `tailscale-serve` | private Tailscale HTTPS |
| `funnel` | `tailscale-funnel` | temporary public HTTPS |
| `public` | `cloudflare-pages` | durable public HTTPS |

If `INTERNAL_URL` or `INTERNAL_DIR` is set, webby also adds an `internal` Caddy
compatibility bag.

Tailnet:

```sh
webby add ./dashboard -b tailnet
webby deploy -b tailnet
```

Temporary public Funnel:

```sh
webby add ./demo -b funnel
webby deploy -b funnel
```

Durable public Cloudflare Pages:

```sh
export CLOUDFLARE_ACCOUNT_ID=...
export CLOUDFLARE_API_TOKEN=...
webby pub ./lissajous --name lissajous
```

## Configuration

Run:

```sh
webby init
```

This writes `~/.config/webby/config.json`. Override the config path with
`WEBBY_CONFIG`, and the default bag storage root with `WEBBY_DATA_DIR`.

Example:

```json
{
  "defaultBag": "local",
  "bags": {
    "local": {
      "dir": "~/.local/share/webby/local",
      "host": { "type": "local", "port": 8765 }
    },
    "tailnet": {
      "dir": "~/.local/share/webby/tailnet",
      "host": { "type": "tailscale-serve", "path": "/", "background": true }
    },
    "funnel": {
      "dir": "~/.local/share/webby/funnel",
      "host": { "type": "tailscale-funnel", "path": "/", "background": true }
    },
    "public": {
      "dir": "~/.local/share/webby/public",
      "host": {
        "type": "cloudflare-pages",
        "project": "webby",
        "tokenEnv": "CLOUDFLARE_API_TOKEN"
      }
    },
    "homepage-tools": {
      "dir": "~/.local/share/webby/tools",
      "noIndex": true,
      "host": { "type": "caddy", "url": "https://example.test/webby/" }
    }
  }
}
```

webby can also load a KEY=VALUE env file from `WEBBY_ENV`; when running from a
checkout, a local `.env.secret` file is loaded if present.

## Provider Notes

`local` generates an index and can be served with `webby serve`.

Webby always writes `webby-cards.json` and `webby-card-grid.js`. By default it
also writes a standalone `index.html` for the bag root.

Set `noIndex: true` on a bag, or pass `--no-index` to `add`, `pub`, `deploy`,
or `serve`, when a larger site consumes the card data and the bag should not
publish its own root index page.

Set `indexChromeDir` on a bag to inline optional `head.html` and `body.html`
fragments into the generated root index. This is intended for host-site chrome
such as a shared header while keeping webby itself generic:

```json
{
  "bags": {
    "public": {
      "dir": "~/.local/share/webby/public",
      "indexChromeDir": "~/site-chrome/dist",
      "host": { "type": "cloudflare-pages", "project": "webby" }
    }
  }
}
```

Use `WEBBY_INDEX_CHROME_DIR` to apply the same chrome directory to every bag.

Caddy-hosted bags do not get a special listing mode. They can either expose the
normal generated Webby index, or set `noIndex` when another homepage owns the
root experience.

`tailscale-serve` and `tailscale-funnel` call the `tailscale` CLI with the bag
directory as the target.

`cloudflare-pages` calls:

```sh
wrangler pages deploy <dir> --project-name <project> --branch <branch> --commit-dirty=true
```

`command` providers run a shell command template. `{dir}`, `{label}`, and
`{url}` are expanded before execution.
