# rs-grid

[![CI](https://github.com/bpodwinski/rs-grid/actions/workflows/ci.yml/badge.svg)](https://github.com/bpodwinski/rs-grid/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/bpodwinski/83f89a42144442293ea8d8111edd7c98/raw/coverage.json)](https://github.com/bpodwinski/rs-grid/actions/workflows/coverage.yml)
[![Security Audit](https://github.com/bpodwinski/rs-grid/actions/workflows/audit.yml/badge.svg)](https://github.com/bpodwinski/rs-grid/actions/workflows/audit.yml)
[![Docs](https://github.com/bpodwinski/rs-grid/actions/workflows/docs.yml/badge.svg)](https://github.com/bpodwinski/rs-grid/actions/workflows/docs.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A high-performance data grid engine built in Rust and compiled to WebAssembly.
Renders on Canvas2D with viewport virtualisation, supporting datasets from
thousands to **quadrillions** of rows at 60 fps.

> **Status:** early development (v0.1.0) — API may change.

<video src="https://rs-grid.com/rsgrid4k.mp4" autoplay loop muted playsinline width="100%"></video>

## Features

- **Viewport virtualisation** — only visible rows are computed and drawn
- **Selection** — cell, row, column and range selection with Shift-click extend
- **Inline editing** — double-click to edit, Enter to commit, Escape to cancel
- **Clipboard** — copy, cut and paste (TSV format)
- **Undo / Redo** — cell edits, paste, column resize, column move
- **Sorting** — click column header to cycle None → Asc → Desc
- **Filtering** — per-column text filter (case-insensitive contains)
- **Search** — Ctrl+F full-grid search with match navigation
- **Column operations** — resize (drag), auto-fit (double-click separator),
  drag & drop reorder, pinned (frozen) columns
- **Scrollbars** — custom-drawn vertical and horizontal scrollbars with
  drag, click-track and arrow buttons
- **Cell formatting** — number, currency, percent, boolean, image, image+text
- **Theming** — CSS custom properties (`--rs-grid-*`), light and dark mode
- **HiDPI** — device-pixel-ratio aware for crisp rendering on Retina displays
- **Context menu** — right-click with built-in and custom actions
- **O(log n) hit-testing** — precomputed column offsets
- **Renderer-agnostic core** — swap Canvas2D for WebGPU without touching
  grid logic

## Architecture

```
GridState  ──►  SceneBuilder  ──►  SceneFrame  ──►  CanvasRenderer  ──►  <canvas>
```

The dependency graph flows in one direction:

```
rs-grid-leptos  →  rs-grid-web  →  rs-grid-render-canvas  →  rs-grid-scene  →  rs-grid-core
```

| Crate | Role |
|---|---|
| `rs-grid-core` | Headless grid logic: model, viewport, selection, hit-testing. **No WASM dependency.** |
| `rs-grid-scene` | Converts `GridState` into renderer-agnostic primitives (`ScenePrimitive`) |
| `rs-grid-render-canvas` | Canvas2D rendering backend via `wasm-bindgen` |
| `rs-grid-web` | Browser integration: DOM events, DPR, rAF loop, CSS theme, clipboard |
| `rs-grid-leptos` | Leptos CSR component wrapper (`<GridCanvas>`) |
| `rs-grid-icons` | Embedded SVG icons (country flags, gender symbols) |

## Quick start

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2021)
- [Trunk](https://trunkrs.dev/) — `cargo install trunk`

### Run the demo

```sh
cd examples/basic-leptos
trunk serve
```

Open <http://localhost:8080>. The demo generates fake data on-the-fly via a
deterministic hash — no server or database needed.

### Use in a Leptos app

```rust
use rs_grid_core::{column::ColumnDef, model::GridModel};
use rs_grid_leptos::GridCanvas;

let columns = vec![
    ColumnDef::new("name", "Name", 200.0),
    ColumnDef::new("email", "Email", 260.0),
];
let data = vec![
    vec!["Alice".into(), "alice@example.com".into()],
    vec!["Bob".into(),   "bob@example.com".into()],
];
let model = GridModel::new(columns, data, 32.0, 40.0);

view! {
    <GridCanvas model=model width="100%" height="600px" />
}
```

## Data sources

rs-grid supports two data source modes:

| Mode | Type | Use case |
|---|---|---|
| In-memory | `VecDataSource` | Small datasets loaded upfront |
| Virtual | `FnDataSource` | Large/infinite datasets generated on demand |

```rust
// Virtual data source — generates cells on the fly
let source = FnDataSource::new(1_000_000, |row, col_key| {
    Some(format!("Row {} / {}", row, col_key))
});
let model = GridModel::with_data_source(columns, Box::new(source), 32.0, 40.0);
```

## Commands

All mutations go through `GridState::apply(GridCommand)`. The full command list:

| Command | Description |
|---|---|
| `SelectCell` | Set single-cell selection |
| `ExtendSelection` | Shift-click extend |
| `ScrollTo` / `ScrollBy` | Absolute or delta scroll |
| `Resize` | Update canvas dimensions |
| `ClearSelection` | Deselect |
| `CopySelection` / `CutSelection` | Clipboard operations |
| `MoveSelection` | Arrow-key navigation |
| `PasteAt` | Paste TSV at anchor |
| `SelectRow` / `SelectCol` | Row or column selection |
| `ResizeColumn` / `AutoFitColumn` | Column width |
| `MoveColumn` | Drag & drop reorder |
| `ToggleSort` | Cycle sort direction |
| `SetColumnFilter` / `ClearAllFilters` | Text filtering |
| `StartEdit` / `CommitEdit` / `CancelEdit` | Inline editing |
| `Undo` / `Redo` | History stack (max 100) |
| `Search` / `SearchNext` / `SearchPrev` | Find in grid |
| `SetPinnedColumnCount` | Frozen columns |

## Theming

The grid reads CSS custom properties from the host page. Define them on `:root`:

```css
:root {
    --rs-grid-bg:              #ffffff;
    --rs-grid-font-family:     "Inter", sans-serif;
    --rs-grid-font-size:       13;
    --rs-grid-cell-color:      #1a1a1a;
    --rs-grid-header-bg:       #f8f8f8;
    --rs-grid-border-color:    #e0e0e0;
    --rs-grid-selection-bg:    rgba(33, 133, 208, 0.15);
    /* ... see examples/basic-leptos/rs-grid-theme.css for full list */
}
```

## Development

```sh
# Check the whole workspace
cargo check --workspace

# Run unit tests (150 tests in rs-grid-core)
cargo test --workspace

# Formatting
cargo fmt --all

# Linting
cargo clippy --workspace -- -D warnings

# Build WASM (production)
cd examples/basic-leptos && trunk build --release
```

### End-to-end tests (Playwright)

```sh
cd e2e && npm install && npx playwright install chromium
cd examples/basic-leptos && trunk build
cd e2e && npm test
```

## Project structure

```
rs-grid/
├── crates/
│   ├── rs-grid-core/           # Headless logic (pure Rust, no WASM)
│   ├── rs-grid-scene/          # Scene graph primitives
│   ├── rs-grid-render-canvas/  # Canvas2D renderer
│   ├── rs-grid-web/            # Browser event handling & rAF loop
│   ├── rs-grid-leptos/         # Leptos <GridCanvas> component
│   └── rs-grid-icons/          # Embedded SVG icons
├── examples/
│   └── basic-leptos/           # Demo app (Leptos + Trunk)
├── e2e/                        # Playwright end-to-end tests
└── docs/                       # Technical documentation
```

## Design constraints

- Row indices are `u64` (not `usize`) to support >4 GB rows on WASM32
- Hit-testing is O(log n) via precomputed column offsets — do not introduce O(n)
- All state mutations go through `GridCommand` — never mutate `GridState` directly
- The core crate has zero WASM dependencies and is testable with `cargo test`
