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
webby add <path> [--name N] [--tmp] [-b BAG]
webby pub <path> [--name N] [--tmp]
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

`webby preview -b BAG` captures static JPEG card previews into the bag's
`webby-previews/` directory. It shells out to `uvx shot-scraper`, skips
existing previews unless `--force` is passed, and keeps generated indexes fast
by serving images instead of live iframes. Pass an app name, for example
`webby preview jobsearch-docs -b internal --force`, to refresh one preview.

Every deploy writes `webby-cards.json` next to the generated assets. That JSON
contains the same card data used by Webby's index component, so another page can
embed a Webby bag without scraping a directory listing.

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
