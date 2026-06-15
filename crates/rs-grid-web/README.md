# rs-grid-web

[![crates.io](https://img.shields.io/crates/v/rs-grid-web.svg)](https://crates.io/crates/rs-grid-web)

Browser integration layer for [rs-grid](https://rs-grid.com). Manages the full lifecycle of a grid instance in the DOM: mouse/keyboard events, canvas sizing, `requestAnimationFrame` loop, DPR handling, CSS theme, and localisation.

## Usage

```rust
use rs_grid_web::canvas::GridCanvas;
use rs_grid_core::model::GridModel;

let canvas: web_sys::HtmlCanvasElement = /* ... */;
let model = GridModel::new(columns, data_source);
let grid = GridCanvas::mount(canvas, model)?;
```

## Callbacks

| Callback | Triggers |
|---|---|
| `set_on_change` | Cell data mutations (paste, edit) |
| `set_on_columns_changed` | Column resize, reorder, pin |
| `set_on_validation_error` | Validator rejected an edit |
| `set_on_cell_button_click` | Cell button clicked |

[Documentation](https://rs-grid.com/getting-started.html) · [Repository](https://github.com/ruxelion/rs-grid) · [crates.io](https://crates.io/crates/rs-grid-web)

## License

MIT
