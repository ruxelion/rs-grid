# rs-grid — recettes just
# Usage: just <recipe>

set shell := ["cmd.exe", "/C"]

# Liste des recettes disponibles
default:
    @just --list

# Vérification rapide (tout le workspace)
check:
    cargo check --workspace

# Build natif (rs-grid-core)
build:
    cargo build -p rs-grid-core

# Tests unitaires
test:
    cargo test --workspace

# Tests unitaires rs-grid-core uniquement
test-core:
    cargo test -p rs-grid-core

# Formatage
fmt:
    cargo fmt --all

# Linting
lint:
    cargo clippy --workspace -- -D warnings

# fmt + lint + test
ci: fmt lint test

# Build WASM (exemple Leptos)
build-wasm:
    cd examples\basic-leptos && trunk build

# Serveur de développement Leptos (port 9081)
serve:
    cd examples\basic-leptos && trunk serve --address 0.0.0.0 --port 9081

# Installer les dépendances Playwright (une seule fois)
e2e-install:
    cd e2e && npm install && npx playwright install chromium

# Lancer les tests e2e (build WASM + tests Playwright)
e2e:
    cd examples\basic-leptos && trunk build
    cd e2e && npm test

# Build WASM pour l'exemple vanilla JS (wasm-pack)
build-js:
    wasm-pack build examples\basic-js --target web --out-dir pkg

# Serveur de développement vanilla JS (port 9080)
serve-js: build-js
    cd examples\basic-js && python -m http.server 9080

# Scaffolder un nouvel exemple wasm-bindgen
new-example name:
    if exist "examples\{{name}}" (echo examples\{{name}} already exists & exit /b 1)
    xcopy /E /I examples\_template-wasm "examples\{{name}}"
    @echo.
    @echo Created examples\{{name}}
    @echo Next steps:
    @echo   1. Rename .tmpl files and replace placeholders
    @echo   2. Add "examples\{{name}}" to [workspace] members in Cargo.toml
    @echo   3. just build-example {{name}}
    @echo   4. just serve-example {{name}}

# Build WASM pour un exemple donné (wasm-pack)
build-example name:
    wasm-pack build examples\{{name}} --target web --out-dir pkg

# Servir un exemple donné (port 9080)
serve-example name:
    just build-example {{name}}
    cd examples\{{name}} && python -m http.server 9080

# Regénérer les screenshots de référence
e2e-update-snapshots:
    cd examples\basic-leptos && trunk build
    cd e2e && npm run update-snapshots

# Serveur de développement RSPress (port 5173)
site:
    cd site && npx rspress dev --host 0.0.0.0

# Build du site RSPress
build-site:
    cd site && npm run build
