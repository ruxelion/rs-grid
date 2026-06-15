# rs-grid-leptos

[Leptos](https://leptos.dev) component wrapper for [rs-grid](https://rs-grid.com) — a high-performance data grid compiled to WebAssembly.

## Installation

```toml
[dependencies]
rs-grid-leptos = "0.1"
rs-grid-core = "0.1"
```

Enable a Leptos rendering mode:

```toml
rs-grid-leptos = { version = "0.1", features = ["csr"] }
# or features = ["hydrate"] for SSR/hydrate apps
```

## Usage

```rust
use rs_grid_leptos::GridCanvas;
use rs_grid_core::model::GridModel;

#[component]
fn App() -> impl IntoView {
    let model = GridModel::new(columns, data_source);
    view! {
        <GridCanvas
            model=model
            width="100%"
            height="500px"
        />
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
| `on_cell_button_click` | `Option<CellButtonClickCb>` | — | `fn(row, col_key, button_id)` |

[Live demo](https://rs-grid.com/index.html#demo) · [Documentation](https://rs-grid.com/getting-started.html) · [Repository](https://github.com/ruxelion/rs-grid)

## License

MIT
