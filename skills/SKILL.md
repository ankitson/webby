---
name: webby
description: Serve or publish a static HTML app to localhost, Tailscale Serve, Tailscale Funnel, Cloudflare Pages, Caddy, or a custom command provider. Use when asked to put an HTML page/tool/visualization online, publish to a tailnet URL, or publish to a public mini site.
---

# webby

Drop a static app into a bag and get a URL. An app is a folder with
`index.html` or a standalone `.html` file. Name it `tmp*` for scratch.

Install: `cargo install webby-deploy && webby ls`

## Bags

```sh
webby ls             # all bags
webby ls -b local    # one bag
webby where          # paths and provider URLs
```

Built-ins:

- `local`: localhost preview, no config.
- `tailnet`: `tailscale serve`.
- `funnel`: `tailscale funnel`.
- `public`: Cloudflare Pages.
- `internal`: optional Caddy compatibility when configured by env.

## Local Preview

```sh
webby add ./clock.html
webby serve
```

This is safe and local-only.

## Tailnet

```sh
webby add ./dashboard -b tailnet
webby deploy -b tailnet
```

Requires authenticated `tailscale`.

## Temporary Public

```sh
webby add ./demo -b funnel
webby deploy -b funnel
```

Always confirm with the user before Funnel. It exposes the app publicly from
the current machine.

## Durable Public

```sh
webby pub ./vancouver-tides
webby deploy -b public
```

Always confirm with the user before `pub` or `deploy -b public`. Cloudflare
Pages publishes to the live internet and expects `CLOUDFLARE_ACCOUNT_ID` plus
`CLOUDFLARE_API_TOKEN`, or a configured token command/reference.

## Common Commands

```sh
webby add <path> [-b bag] [--name name] [--tmp] [--title T] [--description D] [--property K=V]
webby docs <dir> [-b bag] [--name name] [--tmp] [--title T] [--description D] [--property K=V]
webby rm <name> [-b bag]
webby open <name> [-b bag]
webby domain <host> -b public
webby preview [app] -b <bag> [--force]
webby init
```

## App Metadata

Apps can carry card metadata inside their own HTML. For standalone apps, put it
in the `.html` file; for folder apps, put it in `index.html`.

```html
<script type="application/webby+json">
{
  "title": "Network Audit",
  "description": "Internal network and DNS audit notes.",
  "properties": { "category": "Documents" }
}
</script>
```

`webby add` and `webby pub` can write that block into the staged app with
`--title`, `--description`, and repeatable `--property key=value`.

Use `webby docs ./docs --name project-docs` to generate a static docs app from
a Markdown directory. It renders Markdown natively, reads optional YAML
frontmatter, rewrites in-root `.md` links, copies linked in-root assets, and
stages the result like any other folder app.

## Notes

- Rust CLI; use `cargo install --path .` locally.
- `-b` / `--bag` is the only bag selector. There is no `--public` flag.
- `webby preview` captures static optimized WebP card previews into `webby-previews/` via `uvx shot-scraper` and Pillow through `uvx`; pass an app name to refresh a single preview.
- Generated bag indexes render static card HTML; `webby-card-grid.js` is emitted for reusable embeds, not required for the default index to show cards.
- `command` providers can use `{dir}`, `{label}`, and `{url}` template values.
