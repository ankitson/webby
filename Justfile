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
