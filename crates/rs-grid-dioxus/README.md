# rs-grid-dioxus

[![crates.io](https://img.shields.io/crates/v/rs-grid-dioxus.svg)](https://crates.io/crates/rs-grid-dioxus)

[Dioxus](https://dioxuslabs.com) CSR component wrapper for [rs-grid](https://rs-grid.com) — a high-performance data grid compiled to WebAssembly.

## Installation

```toml
[dependencies]
rs-grid-dioxus = "0.1"
rs-grid-core = "0.1"
```

## Usage

```rust
use rs_grid_dioxus::GridCanvas;
use rs_grid_core::model::GridModel;

fn App() -> Element {
    let model = GridModel::new(columns, data_source);
    rsx! {
        GridCanvas {
            model: model,
            width: "100%",
            height: "500px",
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `model` | `GridModel` | required | Data model (consumed on mount) |
| `width` | `String` | `"100%"` | CSS width of the canvas container |
| `height` | `String` | `"600px"` | CSS height of the canvas container |
| `theme` | `Option<Signal<Theme>>` | CSS vars | Custom theme signal |
| `locale` | `Option<Signal<Locale>>` | auto | UI language |
| `on_mount` | `Option<Box<dyn FnOnce(WebGridCanvas)>>` | — | Called after mount with the grid handle |
| `on_validation_error` | `Option<ValidationErrorCb>` | — | `fn(row, col_key, message)` |

[Live demo](https://rs-grid.com/index.html#demo) · [Documentation](https://rs-grid.com/getting-started.html) · [Repository](https://github.com/ruxelion/rs-grid) · [crates.io](https://crates.io/crates/rs-grid-dioxus)

## License

MIT
