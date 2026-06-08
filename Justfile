set shell := ["bash", "-cu"]

# List recipes
default:
    @just --list

# Stage an app into a bag (internal by default). Pass extra flags after `--`.
add path *flags:
    bun run src/cli.ts add {{path}} {{flags}}

# Add to the public bag and deploy to Cloudflare Pages.
pub path *flags:
    bun run src/cli.ts pub {{path}} {{flags}}

# Regenerate the public index and deploy.
deploy:
    bun run src/cli.ts deploy --public

# List apps in a bag (default internal; pass `-- --public`).
ls *flags:
    bun run src/cli.ts ls {{flags}}

# Attach a custom domain to the public Pages bag.
domain host:
    bun run src/cli.ts domain {{host}}

# Build a React/JSX internal app into a self-contained bundle.js.
build name:
    bun build internal/{{name}}/app.jsx --bundle --outfile internal/{{name}}/bundle.js

# Build all internal JSX apps that have an app.jsx.
build-all:
    #!/usr/bin/env bash
    for dir in internal/*/; do
      if [ -f "${dir}app.jsx" ]; then
        name="$(basename "${dir}")"
        echo "Building ${name}..."
        bun build "${dir}app.jsx" --bundle --outfile "${dir}bundle.js"
      fi
    done

# Install webby on PATH from this checkout (tracks local edits).
install:
    bun link

# Print the install one-liners for others.
distribute:
    @echo "bun install -g github:ankitson/webby   # persistent"
    @echo "bunx github:ankitson/webby <cmd>       # one-off (npx-style)"
