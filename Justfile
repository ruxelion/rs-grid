# rs-grid — recettes just
# Usage: just <recipe>

set shell := ["cmd.exe", "/C"]
set dotenv-load

tls_cert := ".certs\\localhost+2.pem"
tls_key  := ".certs\\localhost+2-key.pem"

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
    cargo nextest run --workspace --exclude rs-grid-web --exclude rs-grid-leptos --exclude rs-grid-dioxus --exclude rs-grid-yew --exclude rs-grid-render-canvas --exclude basic-leptos --exclude basic-dioxus --exclude basic-yew --exclude example-common

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

# Formatage
fmt:
    cargo fmt --all

# Linting
lint:
    cargo clippy --workspace -- -D warnings

# Régénérer class_map_data.rs depuis les sources DaisyUI (node_modules)
gen-class-map:
    cd examples\basic-leptos && cmd /c npm install --prefer-offline --no-audit --no-fund
    cd examples\basic-leptos && cmd /c npm run gen

# fmt + lint + test
ci: fmt lint test

# ── TLS ──────────────────────────────────────────────────

# Générer les certificats locaux (mkcert requis)
tls-setup:
    mkdir .certs 2>nul || exit 0
    cd .certs && mkcert localhost 127.0.0.1 ::1

# ── Examples ─────────────────────────────────────────────

# Build WASM d'un exemple (leptos|dioxus|yew|js|react)
build-wasm name:
    @just _build-{{name}}

# Serveur de dev d'un exemple (leptos|dioxus|yew|js|react)
serve name:
    @just _build-{{name}}
    @just _serve-{{name}}

# Scaffolder un nouvel exemple wasm-bindgen
new-example name:
    if exist "examples\{{name}}" (echo examples\{{name}} already exists & exit /b 1)
    xcopy /E /I examples\_template-wasm "examples\{{name}}"
    @echo.
    @echo Created examples\{{name}}
    @echo Next steps:
    @echo   1. Rename .tmpl files and replace placeholders
    @echo   2. Add "examples\{{name}}" to [workspace] members in Cargo.toml
    @echo   3. just build-wasm {{name}}
    @echo   4. just serve {{name}}

[private]
_build-leptos:
    cd examples\basic-leptos && cmd /c npm install --prefer-offline --no-audit --no-fund
    cd examples\basic-leptos && cmd /c npm run css
    cd examples\basic-leptos && trunk build

[private]
_serve-leptos:
    cd examples\basic-leptos && cmd /c npm install --prefer-offline --no-audit --no-fund
    cd examples\basic-leptos && cmd /c npm run css
    cd examples\basic-leptos && trunk serve --address 0.0.0.0 --port 9081 --tls-key-path ..\..\{{tls_key}} --tls-cert-path ..\..\{{tls_cert}}

[private]
_build-dioxus:
    cd examples\basic-dioxus && trunk build

[private]
_serve-dioxus:
    cd examples\basic-dioxus && trunk serve --address 0.0.0.0 --port 9082 --tls-key-path ..\..\{{tls_key}} --tls-cert-path ..\..\{{tls_cert}}

[private]
_build-yew:
    cd examples\basic-yew && trunk build

[private]
_serve-yew:
    cd examples\basic-yew && trunk serve --address 0.0.0.0 --port 9083 --tls-key-path ..\..\{{tls_key}} --tls-cert-path ..\..\{{tls_cert}}

[private]
_build-js:
    wasm-pack build examples\basic-js --target web --out-dir pkg

[private]
_serve-js:
    cd examples\basic-js && python -m http.server 9080

[private]
_build-react:
    just _build-js
    cd examples\basic-react && npm install
    cd examples\basic-react && npm run build

[private]
_serve-react:
    cd examples\basic-react && npm run dev

# ── E2E (Playwright) ─────────────────────────────────────

# Installer les dépendances Playwright (une seule fois)
e2e-install:
    cd e2e && npm install && npx playwright install chromium

# Lancer les tests e2e (build Leptos + Playwright)
e2e:
    just _build-leptos
    cd e2e && npm test

# Regénérer les screenshots de référence
e2e-update-snapshots:
    just _build-leptos
    cd e2e && npm run update-snapshots

# ── MCP (Model Context Protocol) ────────────────────────

# Build le serveur MCP (TypeScript → dist/ + copie des docs)
mcp-build: build-site
    cd mcp && npm install
    cd mcp && npm run build

# Lancer le serveur MCP en mode développement (tsx, sans build)
mcp-dev:
    cd mcp && npm run dev

# Publier le serveur MCP sur npm (NPM_TOKEN requis)
# Usage: NPM_TOKEN=xxx just mcp-publish
mcp-publish: mcp-build
    cd mcp && npm publish --//registry.npmjs.org/:_authToken={{env("NPM_TOKEN")}}

# ── Site (RSPress) ───────────────────────────────────────

# Build WASM demo et copie dans le site
build-site-wasm:
    just _build-js
    if not exist "site\docs\public\wasm" mkdir "site\docs\public\wasm"
    robocopy examples\basic-js\pkg site\docs\public\wasm basic_js.js basic_js_bg.wasm /IS /IT || exit 0

# Serveur de développement RSPress (port 5173)
site: build-site-wasm
    cd site && npm install
    cd site && npx rspress dev --host 0.0.0.0

# Build du site RSPress (avec démo WASM)
build-site: build-site-wasm
    cd site && npm install
    cd site && npm run build
