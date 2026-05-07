# rs-grid-icons

Embedded SVG icon library: country flags (ISO 3166-1 alpha-2) and
gender symbols. All icons are pre-encoded as base64 data URIs at build
time by `build.rs` from the `flags/` and `genders/` source SVGs.

**Zero runtime dependencies, zero network requests, no WASM/web
dependency** — usable from native and WASM targets alike.

## Public API

```rust
// Country flags (ISO 3166-1 alpha-2, uppercase code, e.g. "FR")
pub fn flag_data_uri(code: &str) -> Option<&'static str>;
pub fn flag_count() -> usize;
pub fn all_flags() -> impl Iterator<Item = (&'static str, &'static str)>;

// Gender icons (uppercase key, e.g. "MALE", "FEMALE")
pub fn gender_icon_uri(key: &str) -> Option<&'static str>;
pub fn gender_icon_count() -> usize;
pub fn all_gender_icons()
    -> impl Iterator<Item = (&'static str, &'static str)>;
```

## Critical invariants

- **No WASM / web dependency.** This crate must remain usable from any
  Rust target. Do not add `wasm-bindgen` or `web-sys` here.
- Lookups are **O(log n)** via `binary_search_by_key` — the generated
  `FLAGS` / `GENDERS` slices are sorted at build time.
- Lookup keys are **case-sensitive uppercase** (`"FR"`, `"MALE"`).
  Callers must normalise.
- All data URIs start with `data:image/svg+xml;base64,…` — safe to
  drop directly into `<img src>` or a Canvas2D `drawImage` source.

## Adding or updating icons

1. Drop the SVG into `flags/` (named `XX.svg`, where `XX` is the ISO
   code) or `genders/` (named `KEY.svg`).
2. Rebuild — `build.rs` regenerates `OUT_DIR/icons_data.rs` and
   re-sorts the slice automatically.
3. Add a unit test in `src/lib.rs` covering the new key.
