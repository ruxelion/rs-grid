# Architecture invariant guard.
#
# rs-grid-core must stay free of WASM/web crates so it remains testable with a
# plain native `cargo test`. This fails (exit 1) if wasm-bindgen, web-sys, or
# js-sys appears anywhere in rs-grid-core's normal dependency tree.
#
# Mirrored by the bash step in .github/workflows/ci.yml. Invoked by
# `just check-arch`.

$forbidden = 'wasm-bindgen', 'web-sys', 'js-sys'

$tree = cargo tree -p rs-grid-core -e normal
if ($LASTEXITCODE -ne 0) {
    Write-Error 'cargo tree failed for rs-grid-core.'
    exit 1
}

$hits = $tree | Select-String -SimpleMatch -Pattern $forbidden
if ($hits) {
    Write-Host 'rs-grid-core must not depend on WASM/web crates. Found:'
    $hits | ForEach-Object { Write-Host "  $_" }
    exit 1
}

Write-Host 'OK: rs-grid-core has no WASM/web dependency.'
exit 0
