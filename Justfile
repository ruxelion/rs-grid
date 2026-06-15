# rs-grid — just recipes
# Usage: just <recipe>

set shell := ["cmd.exe", "/C"]
set dotenv-load

# List available recipes
default:
    @just --list

# ── Cargo ────────────────────────────────────────────────

# Quick check (entire workspace)
check:
    cargo check --workspace

# Native build (rs-grid-core)
build:
    cargo build -p rs-grid-core

# Unit tests (entire workspace — WASM crates excluded)
test:
    cargo nextest run --workspace --exclude rs-grid-web --exclude rs-grid-leptos --exclude rs-grid-dioxus --exclude rs-grid-yew --exclude rs-grid-render-canvas --exclude fixture-leptos --exclude example-common

# Unit tests — rs-grid-core only
test-core:
    cargo nextest run -p rs-grid-core

# HTML coverage report (output: target/llvm-cov/html/, opens browser)
# generate_theme.rs is a binary — excluded from coverage
coverage:
    cargo llvm-cov nextest -p rs-grid-core -p rs-grid-scene -p rs-grid-icons --ignore-filename-regex "generate_theme" --html --open

# lcov coverage (CI format → target/llvm-cov/lcov.info)
coverage-lcov:
    if not exist "target\llvm-cov" mkdir "target\llvm-cov"
    cargo llvm-cov nextest -p rs-grid-core -p rs-grid-scene -p rs-grid-icons --ignore-filename-regex "generate_theme" --lcov --output-path target/llvm-cov/lcov.info

# Format (rustfmt.toml uses nightly-only options — nightly required,
# otherwise local formatting diverges from CI)
fmt:
    cargo +nightly fmt --all

# Lint (--all-targets also covers tests, benches and examples)
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Regenerate class_map_data.rs from DaisyUI sources (node_modules)
# Generator lives in tools/class-map (maintainer codegen, not part of demos)
gen-class-map:
    cd tools\class-map && cmd /c npm install --prefer-offline --no-audit --no-fund
    cd tools\class-map && cmd /c npm run gen

# fmt + lint + test
ci: fmt lint test

# ── TLS ──────────────────────────────────────────────────

# Generate local TLS certificates (requires mkcert)
tls-setup:
    mkdir .certs 2>nul || exit 0
    cd .certs && mkcert localhost 127.0.0.1 ::1

# ── Examples ─────────────────────────────────────────────
#
# The framework demos now live in standalone repos:
#   github.com/ruxelion/rs-grid-example-{leptos,dioxus,yew,js}
# Clone one and run `trunk serve` (or `wasm-pack build` for js).

# Build the internal e2e fixture (minimal Leptos app, no Tailwind)
[private]
_build-fixture:
    cd e2e\fixture-leptos && trunk build

# ── E2E (Playwright) ─────────────────────────────────────

# Install Playwright dependencies (run once)
e2e-install:
    cd e2e && npm install && npx playwright install chromium

# Run e2e tests (build fixture + Playwright)
e2e:
    just _build-fixture
    cd e2e && npm test

# Regenerate reference screenshots
e2e-update-snapshots:
    just _build-fixture
    cd e2e && npm run update-snapshots

# ── Benchmarks ───────────────────────────────────────────

# All benchmarks (core + scene), HTML reports in target/criterion/
bench:
    cargo bench -p rs-grid-core -p rs-grid-scene

# rs-grid-core benchmarks only (hit-test + sort + filter)
bench-core:
    cargo bench -p rs-grid-core

# Hit-testing benchmarks only
bench-hit:
    cargo bench -p rs-grid-core --bench hit_test

# Sort and filter benchmarks only
bench-sort:
    cargo bench -p rs-grid-core --bench sort

# Scene builder benchmarks only
bench-scene:
    cargo bench -p rs-grid-scene --bench scene_builder

# Initialization benchmarks (O(n_cols), not O(n_rows))
bench-init:
    cargo bench -p rs-grid-core --bench init

# Full per-frame pipeline benchmarks (scroll + scene render)
bench-scroll:
    cargo bench -p rs-grid-scene --bench scroll_frame

# Memory footprint per row (custom allocator, --release)
mem:
    cargo run -p rs-grid-core --example mem_per_row --release

# WASM bundle size (release build with wasm-opt via Trunk)
wasm-size:
    cd e2e\fixture-leptos && trunk build --release
    powershell -NoProfile -Command "Get-ChildItem e2e\fixture-leptos\dist\*.wasm | ForEach-Object { $kb = [math]::Round($_.Length/1KB,1); $est_gz = [math]::Round($_.Length*0.35/1KB,1); Write-Host ('{0,-50} {1,8} KB  (~{2} KB gzip)' -f $_.Name, $kb, $est_gz) }"

# ── MCP (Model Context Protocol) ────────────────────────

# Build the rs-grid-site documentation (sibling repo → doc_build/).
# Requires rs-grid-site cloned next to this repo (../rs-grid-site).
build-site:
    cd ..\rs-grid-site && npm install --prefer-offline --no-audit --no-fund
    cd ..\rs-grid-site && npm run build

# Build the MCP server (TypeScript → dist/ + copy docs)
mcp-build: build-site
    cd mcp && npm install
    cd mcp && npm run build

# Run the MCP server in development mode (tsx, no build step)
mcp-dev:
    cd mcp && npm run dev

# Publish the MCP server to npm (requires NPM_TOKEN)
# Usage: NPM_TOKEN=xxx just mcp-publish
mcp-publish: mcp-build
    cd mcp && npm publish --//registry.npmjs.org/:_authToken={{env("NPM_TOKEN")}}

# ── Release (release-plz) ────────────────────────────────

# Local release preview: applies version bumps + per-crate CHANGELOG.md to the
# working tree. Inspect with `git diff`, then `git checkout .` to discard.
# One-time prerequisite: cargo install release-plz
release-preview:
    release-plz update
    @echo Inspect with 'git diff', then 'git checkout .' to discard.

# Publish all 8 crates to crates.io in dependency order (30 s between each).
# Requires: cargo login + owner rights on each crate.
publish:
    cargo publish -p rs-grid-core
    @echo Waiting for crates.io index to propagate...
    powershell -NoProfile -Command "Start-Sleep 30"
    cargo publish -p rs-grid-icons
    powershell -NoProfile -Command "Start-Sleep 30"
    cargo publish -p rs-grid-scene
    powershell -NoProfile -Command "Start-Sleep 30"
    cargo publish -p rs-grid-render-canvas
    powershell -NoProfile -Command "Start-Sleep 30"
    cargo publish -p rs-grid-web
    powershell -NoProfile -Command "Start-Sleep 30"
    cargo publish -p rs-grid-leptos
    powershell -NoProfile -Command "Start-Sleep 30"
    cargo publish -p rs-grid-dioxus
    powershell -NoProfile -Command "Start-Sleep 30"
    cargo publish -p rs-grid-yew
    @echo All crates published. Remember to create per-crate tags and run git push origin --tags.
