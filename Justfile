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

# Serveur de développement Leptos (port 9081)
serve:
    Push-Location examples/basic-leptos; trunk serve --address 0.0.0.0 --port 9081; Pop-Location

# Installer les dépendances Playwright (une seule fois)
e2e-install:
    Push-Location e2e; npm install; npx playwright install chromium; Pop-Location

# Lancer les tests e2e (build WASM + tests Playwright)
e2e:
    Push-Location examples/basic-leptos; trunk build; Pop-Location
    Push-Location e2e; npm test; Pop-Location

# Build WASM pour l'exemple vanilla JS (wasm-pack)
build-js:
    wasm-pack build examples/basic-js --target web --out-dir pkg

# Serveur de développement vanilla JS (port 9080)
serve-js: build-js
    Push-Location examples/basic-js; python -m http.server 9080; Pop-Location

# Scaffolder un nouvel exemple wasm-bindgen
new-example name:
    $dest = "examples/{{name}}"; \
    if (Test-Path $dest) { Write-Error "$dest already exists"; exit 1 }; \
    Copy-Item -Recurse examples/_template-wasm $dest; \
    Get-ChildItem $dest -Recurse -Filter *.tmpl | ForEach-Object { \
        $newName = $_.FullName -replace '\.tmpl$',''; \
        $content = (Get-Content $_.FullName -Raw) -replace '\{\{NAME\}\}','{{name}}' -replace '\{\{TITLE\}\}','{{name}}'; \
        Set-Content -Path $newName -Value $content -NoNewline; \
        Remove-Item $_.FullName \
    }; \
    Write-Host "`nCreated $dest"; \
    Write-Host "Next steps:"; \
    Write-Host "  1. Add `"$dest`" to [workspace] members in Cargo.toml"; \
    Write-Host "  2. just build-example {{name}}"; \
    Write-Host "  3. just serve-example {{name}}"

# Build WASM pour un exemple donné (wasm-pack)
build-example name:
    wasm-pack build examples/{{name}} --target web --out-dir pkg

# Servir un exemple donné (port 9080)
serve-example name:
    just build-example {{name}}
    Push-Location examples/{{name}}; python -m http.server 9080; Pop-Location

# Regénérer les screenshots de référence
e2e-update-snapshots:
    Push-Location examples/basic-leptos; trunk build; Pop-Location
    Push-Location e2e; npm run update-snapshots; Pop-Location
