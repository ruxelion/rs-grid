# High-performance data grid engine built in Rust

<p align="center"><img src="rs-grid-logo.png" /></p>

[![CI](https://github.com/ruxelion/rs-grid/actions/workflows/ci.yml/badge.svg)](https://github.com/ruxelion/rs-grid/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/bpodwinski/83f89a42144442293ea8d8111edd7c98/raw/coverage.json)](https://github.com/ruxelion/rs-grid/actions/workflows/coverage.yml)
[![Security Audit](https://github.com/ruxelion/rs-grid/actions/workflows/audit.yml/badge.svg)](https://github.com/ruxelion/rs-grid/actions/workflows/audit.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A high-performance data grid built in Rust and compiled to WebAssembly.
Renders on Canvas2D with viewport virtualisation. Viewport work is bounded by
what's on screen, not by dataset size, so it stays smooth from thousands of
rows up to very large datasets (row indices are `u64`).

> **Status:** early development (alpha) — the API may change. Performance
> figures are illustrative, not yet backed by a published benchmark suite.

**[▶ Live demo](https://ruxelion.github.io/rs-grid/)** ·
**[📖 Documentation](https://rs-grid.com/getting-started.html)**

![rs-grid demo](rsgrid4k.webp)

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

| Crate                   | Role                                                                                  |
| ----------------------- | ------------------------------------------------------------------------------------- |
| `rs-grid-core`          | Headless grid logic: model, viewport, selection, hit-testing. **No WASM dependency.** |
| `rs-grid-scene`         | Converts `GridState` into renderer-agnostic primitives (`ScenePrimitive`)             |
| `rs-grid-render-canvas` | Canvas2D rendering backend via `wasm-bindgen`                                         |
| `rs-grid-web`           | Browser integration: DOM events, DPR, rAF loop, CSS theme, clipboard                  |
| `rs-grid-leptos`        | Leptos CSR component wrapper (`<GridCanvas>`)                                         |
| `rs-grid-dioxus`        | Dioxus CSR component wrapper (`GridCanvas`)                                           |
| `rs-grid-yew`           | Yew CSR component wrapper (`GridCanvas`)                                              |
| `rs-grid-icons`         | Embedded SVG icons (country flags, gender symbols)                                    |

## Quick start

### Prerequisites

- [Rust](https://rust-lang.org/) (edition 2021)
- [Trunk](https://github.com/trunk-rs/trunk) — `cargo install trunk`

### Run a demo

The framework demos live in standalone repos — clone one and run it:

```sh
git clone https://github.com/ruxelion/rs-grid-example-leptos
cd rs-grid-example-leptos
trunk serve
```

Open <http://localhost:9080>. The demo generates fake data on-the-fly via a
deterministic hash — no server or database needed. Demos are also available for
[Dioxus](https://github.com/ruxelion/rs-grid-example-dioxus),
[Yew](https://github.com/ruxelion/rs-grid-example-yew) and
[vanilla JS](https://github.com/ruxelion/rs-grid-example-js).

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

| Mode      | Type            | Use case                                    |
| --------- | --------------- | ------------------------------------------- |
| In-memory | `VecDataSource` | Small datasets loaded upfront               |
| Virtual   | `FnDataSource`  | Large/infinite datasets generated on demand |

```rust
// Virtual data source — generates cells on the fly
let source = FnDataSource::new(1_000_000, |row, col_key| {
    Some(format!("Row {} / {}", row, col_key))
});
let model = GridModel::with_data_source(columns, Box::new(source), 32.0, 40.0);
```

## Commands

All mutations go through `GridState::apply(GridCommand)`. The full command list:

| Command                                   | Description               |
| ----------------------------------------- | ------------------------- |
| `SelectCell`                              | Set single-cell selection |
| `ExtendSelection`                         | Shift-click extend        |
| `ScrollTo` / `ScrollBy`                   | Absolute or delta scroll  |
| `Resize`                                  | Update canvas dimensions  |
| `ClearSelection`                          | Deselect                  |
| `CopySelection` / `CutSelection`          | Clipboard operations      |
| `MoveSelection`                           | Arrow-key navigation      |
| `PasteAt`                                 | Paste TSV at anchor       |
| `SelectRow` / `SelectCol`                 | Row or column selection   |
| `ResizeColumn` / `AutoFitColumn`          | Column width              |
| `MoveColumn`                              | Drag & drop reorder       |
| `ToggleSort`                              | Cycle sort direction      |
| `SetColumnFilter` / `ClearAllFilters`     | Text filtering            |
| `StartEdit` / `CommitEdit` / `CancelEdit` | Inline editing            |
| `Undo` / `Redo`                           | History stack (max 100)   |
| `Search` / `SearchNext` / `SearchPrev`    | Find in grid              |
| `SetPinnedColumnCount`                    | Frozen columns            |

## Theming

The grid reads CSS custom properties from the host page. Define them on `:root`:

```css
:root {
  --rs-grid-bg: #ffffff;
  --rs-grid-header-bg: #f8f9fb;
  --rs-grid-header-text: #181d1f;
  --rs-grid-cell-text: #181d1f;
  --rs-grid-grid-line: #e2e8f0;
  --rs-grid-header-border: #babfc7;
  --rs-grid-selection-fill: rgba(33, 150, 243, 0.2);
  --rs-grid-selection-border: rgba(33, 150, 243, 0.85);
  /* see examples/example-common/themes/light.css for the full list */
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

# Build the e2e fixture app (WASM)
cd e2e/fixture-leptos && trunk build --release
```

### End-to-end tests (Playwright)

```sh
cd e2e && npm install && npx playwright install chromium
cd e2e/fixture-leptos && trunk build
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
│   └── example-common/         # Shared demo data + theme CSS (used by demo repos)
├── e2e/
│   ├── fixture-leptos/         # Minimal Leptos app — the e2e/CI/Pages target
│   └── tests/                  # Playwright end-to-end tests
├── tools/class-map/            # DaisyUI → class_map_data.rs codegen
└── docs/                       # Internal refs only — full docs at ruxelion.com
```

## Design constraints

- Row indices are `u64` (not `usize`) to support >4 GB rows on WASM32
- Hit-testing is O(log n) via precomputed column offsets — do not introduce O(n)
- All state mutations go through `GridCommand` — never mutate `GridState` directly
- The core crate has zero WASM dependencies and is testable with `cargo test`
