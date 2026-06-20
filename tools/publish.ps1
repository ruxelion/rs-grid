# Publish rs-grid crates to crates.io - version-aware.
#
# Walks the 8 crates in dependency order and publishes ONLY those whose current
# version is not yet on crates.io (checked via the registry API), with an
# index-propagation wait after each publish. Unchanged crates are skipped, so
# the recipe is safe to run after any release (it no longer fails trying to
# re-publish already-published versions). Idempotent: re-running after a partial
# failure resumes where it left off.
#
# Requires: cargo login (crates.io token) + owner rights on each crate.
# Invoked by `just publish`. ASCII-only on purpose (PowerShell 5.1 parses
# BOM-less .ps1 files as the ANSI code page).

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot/..").Path
$ua = @{ 'User-Agent' = 'rs-grid-publish (github.com/ruxelion/rs-grid)' }

# Dependency order: a crate must be published before anything that depends on it.
$crates = @(
  'rs-grid-core',
  'rs-grid-icons',
  'rs-grid-scene',
  'rs-grid-render-canvas',
  'rs-grid-web',
  'rs-grid-leptos',
  'rs-grid-dioxus',
  'rs-grid-yew'
)

function Get-CrateVersion($name) {
  $toml = Join-Path $root "crates/$name/Cargo.toml"
  $line = (Select-String -Path $toml -Pattern '^version\s*=' | Select-Object -First 1).Line
  return ($line -replace '.*"(.*)".*', '$1')
}

function Test-Published($name, $ver) {
  try {
    Invoke-RestMethod -Headers $ua "https://crates.io/api/v1/crates/$name/$ver" -ErrorAction Stop | Out-Null
    return $true
  } catch {
    return $false
  }
}

$published = @()
foreach ($name in $crates) {
  $ver = Get-CrateVersion $name
  if (Test-Published $name $ver) {
    Write-Host "skip     $name v$ver (already on crates.io)"
    continue
  }
  Write-Host "publish  $name v$ver ..."
  cargo publish -p $name --manifest-path (Join-Path $root 'Cargo.toml')
  if ($LASTEXITCODE -ne 0) { throw "cargo publish failed for $name v$ver" }
  $published += "$name v$ver"
  Write-Host "  waiting 30s for the crates.io index to propagate..."
  Start-Sleep 30
}

if ($published.Count -eq 0) {
  Write-Host "Nothing to publish: all current versions are already on crates.io."
} else {
  Write-Host "Published: $($published -join ', ')"
}
Write-Host "Reminder: per-crate git tags (rs-grid-<crate>-vX.Y.Z) and 'git push origin --tags'."
