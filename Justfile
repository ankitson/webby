set shell := ["bash", "-cu"]

# List recipes.
default:
    @just --list

# Build the Rust CLI.
build:
    cargo build

# Run formatter, compiler, and tests.
check:
    cargo fmt --check
    cargo check
    cargo test

# Run webby from this checkout. Pass CLI args after the recipe name.
run *args:
    cargo run -- {{args}}

# Stage an app into a bag. Pass extra flags after the recipe name.
add path *flags:
    cargo run -- add {{path}} {{flags}}

# Add to the public bag and deploy to Cloudflare Pages.
pub path *flags:
    cargo run -- pub {{path}} {{flags}}

# Activate/deploy a bag. Pass flags after the recipe name, e.g. `-b public`.
deploy *flags:
    cargo run -- deploy {{flags}}

# List all bags, or pass `-b <name>`.
ls *flags:
    cargo run -- ls {{flags}}

# Serve a bag locally.
serve *flags:
    cargo run -- serve {{flags}}

# Attach a custom domain to a Pages bag. Pass `-- -b public`.
domain host *flags:
    cargo run -- domain {{host}} {{flags}}

# Install webby on PATH from this checkout.
install:
    cargo install --path .

# Install webby from the Git repository.
install-git:
    cargo install --git https://github.com/ankitson/webby

# Push a v-prefixed release tag, which triggers the GitHub release workflow.
release version:
    git tag -a v{{version}} -m "webby v{{version}}"
    git push origin v{{version}}

# Build a React/JSX app asset with Bun when an app needs it.
build-jsx app:
    bun build {{app}}/app.jsx --bundle --outfile {{app}}/bundle.js

# Capture static screenshots for the internal Caddy bag.
preview-internal:
    cargo run -- preview -b internal
