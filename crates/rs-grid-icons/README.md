# rs-grid-icons

Embedded SVG icon library for [rs-grid](https://rs-grid.com). Provides country flags (ISO 3166-1 alpha-2) and gender symbols, pre-encoded as base64 data URIs at build time.

Zero runtime dependencies — no network requests, no WASM or web dependency. Usable from native and WASM targets alike.

## Usage

```rust
use rs_grid_icons::{flag_data_uri, gender_icon_uri};

// Country flag (uppercase ISO code)
if let Some(uri) = flag_data_uri("FR") {
    // uri = "data:image/svg+xml;base64,..."
    // safe to use directly in <img src> or Canvas2D drawImage
}

// Gender icon
if let Some(uri) = gender_icon_uri("MALE") {
    // uri = "data:image/svg+xml;base64,..."
}
```

## Asset provenance

- **Flags**: [flag-icons](https://github.com/lipis/flag-icons) v7.5.0, MIT © Panayiotis Lipiridis
- **Gender symbols**: original minimal glyphs, CC0

[Repository](https://github.com/ruxelion/rs-grid) · [Documentation](https://rs-grid.com/getting-started.html)

## License

MIT
