---
name: webby
description: Serve or publish a static HTML app to localhost, Tailscale Serve, Tailscale Funnel, Cloudflare Pages, Caddy, or a custom command provider. Use when asked to put an HTML page/tool/visualization online, publish to a tailnet URL, or publish to a public mini site.
---

# webby

Drop a static app into a bag and get a URL. An app is a folder with
`index.html` or a standalone `.html` file. Name it `tmp*` for scratch.

## Run From This Repo

```sh
cargo run -- where
cargo run -- add ./clock.html
cargo run -- serve
```

Installed CLI:

```sh
webby where
```

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
webby add <path> [-b bag] [--name name] [--tmp]
webby rm <name> [-b bag]
webby open <name> [-b bag]
webby domain <host> -b public
webby preview [app] -b <bag> [--force]
webby init
```

## Notes

- Rust CLI; use `cargo install --path .` locally.
- `-b` / `--bag` is the only bag selector. There is no `--public` flag.
- `webby preview` captures static JPEG card previews into `.webby-previews/` via `uvx shot-scraper`; pass an app name to refresh a single preview.
- `command` providers can use `{dir}`, `{label}`, and `{url}` template values.
