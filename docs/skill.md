---
name: rs-grid
description: >
  High-performance, renderer-agnostic data grid engine built with Rust and
  compiled to WebAssembly. Use when embedding a virtualized data grid in a
  Leptos CSR application, adding Canvas2D-based grid rendering, or implementing
  large-dataset table views in the browser.
license: MIT
compatibility: >
  Requires Rust 2021 edition, wasm32-unknown-unknown target, and Trunk for
  WASM builds. Leptos CSR only (no SSR). Browser Canvas2D API at runtime.
metadata:
  author: ruxelion
  version: "0.1"
  repository: https://github.com/ruxelion/rs-grid
---

# rs-grid

High-performance data grid engine for the browser, written in Rust and compiled
to WebAssembly. Handles millions of rows with smooth 60 fps rendering via a
virtualized Canvas2D renderer.

## Capabilities

- Render large datasets (up to ~9×10¹⁴ rows) with viewport virtualization
- O(log n) hit-testing via precomputed column offset arrays
- Cell, row, and column selection with anchor/focus model
- Renderer-agnostic scene graph (ScenePrimitive)
- Leptos CSR component out of the box
- CSS variable theming

## Skills

### Add rs-grid to a Leptos project

1. Add the git-tag dependencies to `Cargo.toml` (rs-grid is consumed by git
   tag, not from crates.io):

   ```toml
   rs-grid-core   = { git = "https://github.com/ruxelion/rs-grid", tag = "v0.1.0" }
   rs-grid-leptos = { git = "https://github.com/ruxelion/rs-grid", tag = "v0.1.0" }
   ```

2. Build a `GridModel` and mount `GridCanvas` with the `model=` prop:

   ```rust
   use rs_grid_core::{column::ColumnDef, model::GridModel};
   use rs_grid_leptos::GridCanvas;

   let columns = vec![ColumnDef::new("name", "Name", 200.0)];
   let data = vec![vec!["Alice".into()]];
   let model = GridModel::new(columns, data, 32.0, 40.0);

   view! { <GridCanvas model=model width="100%" height="600px" /> }
   ```

3. Define `--rs-grid-*` CSS custom properties on `:root` for theming
   (see `examples/example-common/themes/light.css`).

### Mutate grid state

All mutations use `GridState::apply(GridCommand)`:

- `ScrollTo { x, y }` — update scroll position
- `SelectCell { row, col }` — select a cell
- `ExtendSelection { row, col }` — shift-extend
- `Resize { width, height }` — update canvas size
- `ClearSelection` — deselect all

### Add a new renderer backend

1. Create a crate depending on `rs-grid-scene`
2. Call `SceneBuilder::build(&grid_state)` to get a `SceneFrame`
3. Iterate over primitives (Rect, Text, Line) and issue draw calls
4. Do not modify `rs-grid-core` or `rs-grid-scene`

## Constraints

- `rs-grid-core` has no WASM dependency — keep it that way
- Row indices must use `u64`, not `usize`
- Hit-testing must remain O(log n)
- Max line width: 80 characters
- No `unwrap()` in production code

## Workflows

### Run tests

```
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

### Run the e2e fixture locally

```
cd e2e/fixture-leptos && trunk serve
```
