# rs-grid — recettes just
# Usage: just <recipe>

set shell := ["powershell.exe", "-NoLogo", "-Command"]

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
    Push-Location examples/basic-leptos; trunk build; Pop-Location

# Serveur de développement (port 9080, configuré dans Trunk.toml)
serve:
    Push-Location examples/basic-leptos; trunk serve --address 0.0.0.0; Pop-Location

# Installer les dépendances Playwright (une seule fois)
e2e-install:
    Push-Location e2e; npm install; npx playwright install chromium; Pop-Location

# Lancer les tests e2e (build WASM + tests Playwright)
e2e:
    Push-Location examples/basic-leptos; trunk build; Pop-Location
    Push-Location e2e; npm test; Pop-Location

# Regénérer les screenshots de référence
e2e-update-snapshots:
    Push-Location examples/basic-leptos; trunk build; Pop-Location
    Push-Location e2e; npm run update-snapshots; Pop-Location
