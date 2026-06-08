---
name: webby
description: Serve a simple HTML app — internally via the home Caddy (instant, LAN/Tailscale) or publicly via Cloudflare Pages. Use when asked to "put this online", "drop an HTML page", "make a quick tool/visualization reachable", or publish to the public mini site.
---

# webby

drop an html app, get a url. internal = instant Caddy. public = Cloudflare Pages.

an app is a **folder with `index.html`** or a **standalone `.html` file**.
name it `tmp*` for throwaway (gitignored, shown under a Temp heading).

## run it

```sh
bunx github:ankitson/webby <cmd>    # e.g. bunx github:ankitson/webby where
```

## first: find the bags

paths and urls aren't hardcoded here — ask webby:

```sh
webby where
```

run everything from the webby dir (`bun run src/cli.ts …`), or via the wrapper if installed.

## internal (instant, no deploy)

```sh
webby add ./clock.html              # → live now on the tools host
webby add ./dashboard               # folder app
webby add ./scratch.html --tmp      # throwaway
```

caddy serves it the moment it lands. no build, no deploy.

## public (Cloudflare Pages)

```sh
webby pub ./vancouver-tides         # stage in public/ + deploy
webby deploy --public               # re-push the whole public dir
```

deploy is just "push the `public/` directory" — nothing more.
public apps also appear on the internal listing automatically (symlinked).

> **always confirm with the user before a public deploy** (`pub` / `deploy --public`).
> it publishes to the live internet. internal `add` is safe — deploy is not.

## the rest

```sh
webby ls                # list a bag (add --public for the public one)
webby open <name>       # print/open the url
webby rm <name>         # remove (add --public for public)
webby domain <host>     # attach a custom domain to the public bag
```

## notes

- secrets/domains live in `.env.secret`, never in code.
- bun + typescript; `wrangler` via `bunx`.
