# rs-grid-core

Headless grid engine core for [rs-grid](https://rs-grid.com) — zero WASM dependency, fully testable on native targets.

Provides the central `GridState` struct combining model, viewport, selection, and hit-testing. All mutations go through `GridState::apply(GridCommand)`.

## Features

- Viewport virtualisation — only visible rows are computed
- O(log n) hit-testing on cells, row headers, and column headers
- Selection: cell, row, column, range
- Sorting and filtering
- Clipboard (TSV copy/paste)
- Undo/redo
- Row indices are `u64` — supports datasets larger than 4 billion rows on WASM32

## Usage

```rust
use rs_grid_core::{model::GridModel, state::GridState, commands::GridCommand};

let model = GridModel::new(columns, data_source);
let mut state = GridState::new(model);
state.apply(GridCommand::ScrollTo { x: 0.0, y: 0.0 });
```

## Part of the rs-grid family

| Crate | Role |
|---|---|
| **rs-grid-core** | Headless engine (this crate) |
| rs-grid-scene | Scene graph / rendering primitives |
| rs-grid-render-canvas | Canvas2D renderer |
| rs-grid-web | Browser event loop |
| rs-grid-leptos | Leptos component |
| rs-grid-dioxus | Dioxus component |
| rs-grid-yew | Yew component |
| rs-grid-icons | Embedded SVG flags & icons |

[Documentation](https://rs-grid.com/getting-started.html) · [Repository](https://github.com/ruxelion/rs-grid) · [Live demo](https://rs-grid.com/index.html#demo)

## License

MIT
