# rs-grid — recettes just
# Usage: just <recipe>

set shell := ["powershell.exe", "-NoLogo", "-Command"]

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
    cd examples/basic-leptos && trunk build

# Serveur de développement
serve:
    cd examples/basic-leptos && trunk serve

# Installer les dépendances Playwright (une seule fois)
e2e-install:
    cd e2e && npm install && npx playwright install chromium

# Lancer les tests e2e (build WASM + tests Playwright)
e2e:
    cd examples/basic-leptos && trunk build
    cd e2e && npm test

# Regénérer les screenshots de référence
e2e-update-snapshots:
    cd examples/basic-leptos && trunk build
    cd e2e && npm run update-snapshots
