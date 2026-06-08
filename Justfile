# rs-grid — recettes just
# Usage: just <recipe>

set shell := ["cmd.exe", "/C"]
set dotenv-load

# Liste des recettes disponibles
default:
    @just --list

# ── Cargo ────────────────────────────────────────────────

# Vérification rapide (tout le workspace)
check:
    cargo check --workspace

# Build natif (rs-grid-core)
build:
    cargo build -p rs-grid-core

# Tests unitaires (tout le workspace — crates WASM exclues)
test:
    cargo nextest run --workspace --exclude rs-grid-web --exclude rs-grid-leptos --exclude rs-grid-dioxus --exclude rs-grid-yew --exclude rs-grid-render-canvas --exclude fixture-leptos --exclude example-common

# Tests unitaires rs-grid-core uniquement
test-core:
    cargo nextest run -p rs-grid-core

# Coverage HTML (rapport dans target/llvm-cov/html/, ouvre le navigateur)
# generate_theme.rs est un binaire, exclu du coverage
coverage:
    cargo llvm-cov nextest -p rs-grid-core -p rs-grid-scene -p rs-grid-icons --ignore-filename-regex "generate_theme" --html --open

# Coverage lcov (format CI → target/llvm-cov/lcov.info)
coverage-lcov:
    if not exist "target\llvm-cov" mkdir "target\llvm-cov"
    cargo llvm-cov nextest -p rs-grid-core -p rs-grid-scene -p rs-grid-icons --ignore-filename-regex "generate_theme" --lcov --output-path target/llvm-cov/lcov.info

# Formatage (rustfmt.toml utilise des options nightly — nightly obligatoire,
# sinon le formatage local diverge de la CI)
fmt:
    cargo +nightly fmt --all

# Linting (--all-targets couvre aussi tests, benches et examples)
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Régénérer class_map_data.rs depuis les sources DaisyUI (node_modules)
# Le générateur vit dans tools/class-map (maintainer codegen, hors démos).
gen-class-map:
    cd tools\class-map && cmd /c npm install --prefer-offline --no-audit --no-fund
    cd tools\class-map && cmd /c npm run gen

# fmt + lint + test
ci: fmt lint test

# ── TLS ──────────────────────────────────────────────────

# Générer les certificats locaux (mkcert requis)
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

# Installer les dépendances Playwright (une seule fois)
e2e-install:
    cd e2e && npm install && npx playwright install chromium

# Lancer les tests e2e (build fixture + Playwright)
e2e:
    just _build-fixture
    cd e2e && npm test

# Regénérer les screenshots de référence
e2e-update-snapshots:
    just _build-fixture
    cd e2e && npm run update-snapshots

# ── Benchmarks ───────────────────────────────────────────

# Tous les benchmarks (core + scene), rapports HTML dans target/criterion/
bench:
    cargo bench -p rs-grid-core -p rs-grid-scene

# Benchmarks rs-grid-core uniquement (hit-test + tri + filtre)
bench-core:
    cargo bench -p rs-grid-core

# Benchmarks hit-testing uniquement
bench-hit:
    cargo bench -p rs-grid-core --bench hit_test

# Benchmarks tri et filtre uniquement
bench-sort:
    cargo bench -p rs-grid-core --bench sort

# Benchmarks scene builder uniquement
bench-scene:
    cargo bench -p rs-grid-scene --bench scene_builder

# Benchmarks initialisation (O(n_cols), pas O(n_rows))
bench-init:
    cargo bench -p rs-grid-core --bench init

# Benchmarks pipeline complet par frame (scroll + rendu scène)
bench-scroll:
    cargo bench -p rs-grid-scene --bench scroll_frame

# Mesure l'empreinte mémoire par ligne (allocateur custom, --release)
mem:
    cargo run -p rs-grid-core --example mem_per_row --release

# Mesure la taille du bundle WASM (release, wasm-opt inclus via Trunk)
wasm-size:
    cd e2e\fixture-leptos && trunk build --release
    powershell -NoProfile -Command "Get-ChildItem e2e\fixture-leptos\dist\*.wasm | ForEach-Object { $kb = [math]::Round($_.Length/1KB,1); $est_gz = [math]::Round($_.Length*0.35/1KB,1); Write-Host ('{0,-50} {1,8} KB  (~{2} KB gzip)' -f $_.Name, $kb, $est_gz) }"

# ── MCP (Model Context Protocol) ────────────────────────

# Build le serveur MCP (TypeScript → dist/ + copie des docs)
mcp-build:
    cd mcp && npm install
    cd mcp && npm run build

# Lancer le serveur MCP en mode développement (tsx, sans build)
mcp-dev:
    cd mcp && npm run dev

# Publier le serveur MCP sur npm (NPM_TOKEN requis)
# Usage: NPM_TOKEN=xxx just mcp-publish
mcp-publish: mcp-build
    cd mcp && npm publish --//registry.npmjs.org/:_authToken={{env("NPM_TOKEN")}}
