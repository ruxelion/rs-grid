# rs-grid-icons

Embedded SVG icon library: country flags (ISO 3166-1 alpha-2) and
gender symbols, plus the matching country names in English and 14
other languages. Icons are pre-encoded as base64 data URIs at build
time by `build.rs` from the `flags/` and `genders/` source SVGs.
English country names have no such source file and are
hand-maintained in `src/countries.rs`. Translated names come from
`country_names/*.toml` (one file per language, CLDR-sourced), parsed
by `build.rs` into `COUNTRY_TRANSLATIONS`.

**Zero runtime dependencies, zero network requests, no WASM/web
dependency** — usable from native and WASM targets alike.

## Asset provenance

- **Flags** (`flags/*.svg`): [flag-icons](https://github.com/lipis/flag-icons)
  v7.5.0 (4x3 set), MIT, © 2013 Panayiotis Lipiridis.
- **Gender symbols** (`genders/*.svg`): original minimal Mars/Venus glyphs,
  CC0.
- **Country name translations** (`country_names/*.toml`, 14 languages):
  [CLDR](https://cldr.unicode.org/) 48.2.0 (Unicode, Inc.), Unicode License
  V3 (`SPDX: Unicode-3.0`). The 4 flag-icons UK-subdivision codes
  (`GB-ENG`/`GB-NIR`/`GB-SCT`/`GB-WLS`) aren't real CLDR territories and are
  hand-translated per language instead.
- Attribution lives in `THIRD-PARTY-LICENSES.md` at the workspace root. When
  updating flags or country name translations, keep that file in sync.

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

// Country names (same codes as flag_data_uri, e.g. "FR" -> "France")
pub fn country_name(code: &str) -> Option<&'static str>;
pub fn country_count() -> usize;
pub fn all_countries() -> impl Iterator<Item = (&'static str, &'static str)>;

// Country names, translated (BCP-47 primary subtag, e.g. "fr", "de")
pub fn country_name_in(code: &str, lang: &str) -> Option<&'static str>;
pub fn supported_country_langs() -> Vec<&'static str>; // 14 codes, "en" is the default/fallback
```

## Critical invariants

- **No WASM / web dependency.** This crate must remain usable from any
  Rust target. Do not add `wasm-bindgen` or `web-sys` here.
- Lookups are **O(log n)** via `binary_search_by_key` — the generated
  `FLAGS` / `GENDERS` slices are sorted at build time. `country_name`
  is the one exception: `COUNTRY_NAMES` is hand-maintained (no
  build.rs sort step), so it's a plain O(n) linear scan — fine at
  254 entries and not a render-hot path.
- Lookup keys are **case-sensitive uppercase** (`"FR"`, `"MALE"`).
  Callers must normalise.
- `COUNTRY_NAMES` must stay in sync with `FLAGS` — every flag code
  needs a name and vice versa. Enforced by the
  `every_flag_has_a_country_name` / `every_country_name_has_a_flag`
  tests in `src/lib.rs`.
- `country_name_in` always succeeds wherever `country_name` does — it
  falls back to the English name for `"en"`, an unrecognized language
  tag, or a code missing from that language's table. It can only
  return `None` where `country_name` itself would.
- All data URIs start with `data:image/svg+xml;base64,…` — safe to
  drop directly into `<img src>` or a Canvas2D `drawImage` source.

## Adding or updating icons

1. Drop the SVG into `flags/` (named `XX.svg`, where `XX` is the ISO
   code) or `genders/` (named `KEY.svg`).
2. Rebuild — `build.rs` regenerates `OUT_DIR/icons_data.rs` and
   re-sorts the slice automatically.
3. If it's a new flag, add the matching `("XX", "Name")` entry to
   `COUNTRY_NAMES` in `src/countries.rs` — nothing generates this
   automatically, and `every_flag_has_a_country_name` will fail the
   build until you do.
4. Also add an `XX = "..."` line to each of the 14
   `country_names/*.toml` files — the
   `every_declared_lang_covers_every_flag_code` test fails otherwise.
   `country_name_in` itself would gracefully fall back to English for
   a gap at runtime, but the test deliberately bypasses that fallback
   to keep coverage complete rather than silently degrading.
5. Add a unit test in `src/lib.rs` covering the new key.
