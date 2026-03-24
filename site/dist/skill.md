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
  WASM builds. Leptos CSR only (no SSR support). Browser Canvas2D API required
  at runtime.
metadata:
  author: bpodwinski
  version: "0.1"
  repository: https://github.com/bpodwinski/rs-grid
  stack: Rust, WebAssembly, Leptos, Canvas2D
---

# rs-grid

High-performance data grid engine for the browser, written in Rust and compiled
to WebAssembly. Handles millions of rows with smooth 60 fps rendering via a
virtualized Canvas2D renderer.

## Capabilities

- Render large datasets (up to ~9×10¹⁴ rows) with viewport virtualization
- O(log n) hit-testing via precomputed column offset arrays
- Cell, row, and column selection with anchor/focus model
- Renderer-agnostic scene graph — swap out the rendering backend without touching core
- CSS variable theming
- Leptos CSR component integration

## Architecture context

Data flows unidirectionally through a fixed pipeline:

```
GridState → SceneBuilder → SceneFrame → CanvasRenderer → <canvas>
```

All mutations go through `GridState::apply(GridCommand)`. There are no
callbacks or two-way bindings. The pipeline is deterministic and easy to test.

### Workspace crates

| Crate | Role | WASM |
|-------|------|------|
| `rs-grid-core` | Headless logic: model, viewport, selection, hit-testing | No |
| `rs-grid-scene` | Converts `GridState` to `ScenePrimitive` values | No |
| `rs-grid-render-canvas` | Canvas2D renderer via wasm-bindgen | Yes |
| `rs-grid-web` | Browser glue: events, DPR, requestAnimationFrame | Yes |
| `rs-grid-leptos` | Leptos CSR component `<GridCanvas>` | Yes |

Dependency direction: `leptos → web → render-canvas → scene → core`.
Never introduce a reverse dependency.

## Skills

### Add rs-grid to a Leptos project

**Inputs required:**
- Existing Leptos CSR project with Trunk configured
- `wasm32-unknown-unknown` target installed

**Steps:**
1. Add `rs-grid-leptos = { path = "../rs-grid-leptos" }` to `Cargo.toml`
2. Import and mount `<GridCanvas rows=N cols=M />` in a Leptos component
3. Include the theme CSS file (`rs-grid-theme.css`) via Trunk or a stylesheet

**Constraints:**
- Leptos CSR only; SSR is not supported
- Row indices are `u64`; column indices are `usize`
- Props `rows` and `cols` are required; `row_height` and `header_height` are optional

### Mutate grid state

All mutations use the command pattern:

```rust
grid_state.apply(GridCommand::ScrollTo { x, y });
grid_state.apply(GridCommand::SelectCell { row, col });
grid_state.apply(GridCommand::Resize { width, height });
grid_state.apply(GridCommand::ClearSelection);
```

**Constraints:**
- Do not mutate fields directly — always use `apply(GridCommand)`
- Commands are applied synchronously; the renderer picks up changes on the next
  animation frame

### Add a new renderer backend

**Inputs required:**
- A new Rust crate depending on `rs-grid-scene`

**Steps:**
1. Create a crate with `rs-grid-scene` as a dependency
2. Call `SceneBuilder::build(&grid_state)` to obtain a `SceneFrame`
3. Iterate over `SceneFrame::primitives` and issue draw calls for each
   `ScenePrimitive` (Rect, Text, Line)

**Constraints:**
- Do not modify `rs-grid-core` or `rs-grid-scene`
- The new crate may introduce WASM dependencies; `core` and `scene` must remain WASM-free

### Theme the grid

Override CSS custom properties before the component mounts:

```css
:root {
  --rs-grid-bg:              #0d1117;
  --rs-grid-header-bg:       #161b22;
  --rs-grid-border:          #30363d;
  --rs-grid-text:            #c9d1d9;
  --rs-grid-selection-bg:    rgba(56, 139, 253, 0.15);
  --rs-grid-selection-border:#388bfd;
}
```

The web integration layer reads these via `rs-grid-web::theme_from_css_vars`
at mount time.

## Workflows

### Run tests

```bash
# Unit tests (native, no WASM needed)
cargo test --workspace

# Quick type-check
cargo check --workspace

# Lint
cargo clippy --workspace -- -D warnings
```

### Run the demo locally

```bash
cd examples/basic-leptos
trunk serve
# Open http://localhost:8080
```

### Build the documentation site with Docker

```bash
# From repo root
docker compose up --build
# Open http://localhost:8080
```

## Constraints

- `rs-grid-core` has no WASM dependency — keep it that way
- Row indices must use `u64`, not `usize` (wasm32 has 32-bit `usize`)
- Hit-testing must remain O(log n) — never introduce O(n) in that path
- Maximum line width: 80 characters (enforced by `rustfmt.toml`)
- No `unwrap()` in production code — use `expect("reason")` or `?`
