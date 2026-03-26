# rs-grid

A headless data grid engine built with Rust and compiled to WebAssembly.
Designed for millions of rows with smooth 60 fps rendering on Canvas2D.

- [**Getting Started**](/getting-started.md) — Install rs-grid and render your first grid in minutes.
- [**Architecture**](/concepts/architecture.md) — Understand the unidirectional pipeline from state to canvas.
- [**Features**](/features/columns.md) — Columns, editing, sorting, filtering, clipboard, and more.
- [**Theming**](/theming/overview.md) — Customize colors, fonts, and spacing with CSS variables.
- [**Data Sources**](/data/overview.md) — In-memory, virtual, or server-side paginated data.
- [**API Reference**](/api/grid-state.md) — Full reference for GridState, GridCommand, and all types.

## Key features

- **Viewport virtualization** — only visible cells are rendered, regardless of dataset size
- **O(1) hit-testing** — precomputed column offsets and uniform row height
- **Inline editing** — text input and select dropdown editors with undo/redo
- **Sorting & filtering** — per-column sort and text filter, client or server-side
- **Copy/cut/paste** — TSV clipboard with RFC 4180 support
- **Ctrl+F search** — case-insensitive full-text search with highlights
- **Column operations** — resize, move, pin, auto-fit
- **3 data sources** — in-memory, virtual (closure-based), async page cache
- **4 built-in themes** — light, dark, Material 3, Material 3 Dark
- **Renderer-agnostic** — scene graph of primitives, swap the backend
- **Custom scrollbars** — themed, cross-platform consistent
- **Context menu** — built-in and customizable actions
- **Leptos, Dioxus, Yew & vanilla JS** — use with Leptos CSR, Dioxus CSR, or plain JavaScript

## Architecture at a glance

```
GridState → SceneBuilder → SceneFrame → CanvasRenderer → <canvas>
```

All mutations go through `GridState::apply(GridCommand)`. No callbacks, no
two-way bindings. The pipeline is deterministic and easy to test.

## Workspace crates

| Crate                   | Role                                                       | WASM |
| ----------------------- | ---------------------------------------------------------- | ---- |
| `rs-grid-core`          | Headless logic — model, viewport, selection, hit-testing   | No   |
| `rs-grid-scene`         | Converts `GridState` to renderer-agnostic `ScenePrimitive` | No   |
| `rs-grid-render-canvas` | Canvas2D renderer via wasm-bindgen                         | Yes  |
| `rs-grid-web`           | Browser glue — events, DPR, requestAnimationFrame          | Yes  |
| `rs-grid-leptos`        | Leptos CSR component `<GridCanvas>`                        | Yes  |

:::warning
Dependencies flow in one direction: `leptos → web → render-canvas → scene → core`.
Never introduce a reverse dependency.
:::
