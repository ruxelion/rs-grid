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

1. Add `rs-grid-leptos = { path = "../rs-grid-leptos" }` to Cargo.toml
2. Import and mount `<GridCanvas rows=N cols=M />` in a Leptos view
3. Include `rs-grid-theme.css` for theming

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

### Run demo locally

```
cd examples/basic-leptos && trunk serve
```

### Build docs site with Docker

```
docker compose up --build
```
