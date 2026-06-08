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

# Compile a standalone webby binary for each target into dist/.
binaries:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p dist && rm -f dist/webby-*
    targets="bun-linux-x64:linux-x64 bun-linux-arm64:linux-arm64 bun-darwin-arm64:darwin-arm64 bun-darwin-x64:darwin-x64"
    for t in $targets; do
      bunt="${t%%:*}"; name="${t##*:}"
      echo "compiling webby-${name}…"
      bun build src/cli.ts --compile --minify --target="${bunt}" --outfile "dist/webby-${name}"
    done
    ls -lh dist/

# Cut a GitHub release (tag vX.Y.Z) and upload the dist/ binaries.
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    just binaries
    gh release create "{{version}}" dist/webby-* --title "webby {{version}}" \
      --notes "Standalone webby CLI. Download your platform's binary, \`chmod +x\`, and put it on PATH. Needs a .env.secret next to the binary (or \$WEBBY_ENV) — see .env.secret.example."
