# Architecture

## Pipeline

Data flows in one direction through a fixed pipeline:

```
GridState  ──►  SceneBuilder  ──►  SceneFrame  ──►  CanvasRenderer  ──►  <canvas>
```

Each step has a single responsibility. Nothing flows backwards.

## GridState

`GridState` is the central structure. It holds three sub-states:

| Sub-state   | Type             | Responsibility                          |
| ----------- | ---------------- | --------------------------------------- |
| `model`     | `GridModel`      | Column definitions and row data         |
| `viewport`  | `ViewportState`  | `scroll_x`, `scroll_y`, width, height   |
| `selection` | `SelectionState` | Anchor and focus (cell, row, or column) |

All mutations go through `GridState::apply(GridCommand)`. Direct field
mutation is not exposed. This makes the state easy to snapshot and replay.

## Commands

`GridCommand` is an enum of all possible mutations:

```rust
GridCommand::ScrollTo { x: f64, y: f64 }
GridCommand::SelectCell { row: u64, col: usize }
GridCommand::Resize { width: f64, height: f64 }
GridCommand::ClearSelection
```

Commands are cheap to construct and can be queued. The renderer calls `apply`
once per animation frame with any pending commands.

## SceneFrame

`SceneBuilder::build(&grid_state)` returns a `SceneFrame` — a flat list of
`ScenePrimitive` values that describe exactly what should appear on screen:

- **`Rect`** — filled rectangle (cell background, selection highlight)
- **`Text`** — cell label with font, color, and clipping bounds
- **`Line`** — grid border line

The scene is renderer-agnostic. You can implement a new backend (WebGL, SVG,
terminal) by consuming `SceneFrame` without touching core.

## Crate dependency graph

```
rs-grid-leptos
  └── rs-grid-web
       └── rs-grid-render-canvas
            └── rs-grid-scene
                 └── rs-grid-core
```

| Crate                   | Depends on    | WASM required |
| ----------------------- | ------------- | ------------- |
| `rs-grid-core`          | —             | No            |
| `rs-grid-scene`         | core          | No            |
| `rs-grid-render-canvas` | scene         | Yes           |
| `rs-grid-web`           | render-canvas | Yes           |
| `rs-grid-leptos`        | web           | Yes           |

## Adding a renderer

1. **Create a new crate**

   Add `rs-grid-scene` as a dependency.

2. **Consume SceneFrame**

   Call `SceneBuilder::build(&grid_state)` to get a `SceneFrame`, then
   iterate over `SceneFrame::primitives`.

3. **Issue draw calls**

   Map each `ScenePrimitive` (Rect, Text, Line) to your backend's API.

:::warning
Do not modify `rs-grid-core` or `rs-grid-scene` when adding a renderer.
These crates must remain WASM-free and testable with plain `cargo test`.
:::
