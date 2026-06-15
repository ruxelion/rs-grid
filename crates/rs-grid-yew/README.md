# rs-grid-yew

[Yew](https://yew.rs) CSR component wrapper for [rs-grid](https://rs-grid.com) — a high-performance data grid compiled to WebAssembly.

## Installation

```toml
[dependencies]
rs-grid-yew = "0.1"
rs-grid-core = "0.1"
```

## Usage

```rust
use rs_grid_yew::{GridCanvas, ModelSlot};
use rs_grid_core::model::GridModel;

#[function_component]
fn App() -> Html {
    let model = GridModel::new(columns, data_source);
    let slot = ModelSlot::new(model);
    html! {
        <GridCanvas model={slot} width="100%" height="500px" />
    }
}
```

`GridModel` is not `Clone`, so it must be wrapped in `ModelSlot` before passing as a Yew prop.

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `model` | `ModelSlot` | required | Wrap with `ModelSlot::new(model)` |
| `width` | `AttrValue` | `"100%"` | CSS width of the canvas container |
| `height` | `AttrValue` | `"600px"` | CSS height of the canvas container |
| `theme` | `Option<Theme>` | CSS vars | Custom theme |
| `locale` | `Option<Locale>` | auto | UI language |
| `on_mount` | `Option<Callback<WebGridCanvas>>` | — | Called after mount with the grid handle |
| `on_validation_error` | `Option<ValidationErrorCb>` | — | `fn(row, col_key, message)` |

[Live demo](https://rs-grid.com/index.html#demo) · [Documentation](https://rs-grid.com/getting-started.html) · [Repository](https://github.com/ruxelion/rs-grid)

## License

MIT
