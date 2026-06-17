# rs-grid-scene

Scene graph layer. Converts a `GridState` into a list of renderer-agnostic
drawing primitives.

## Modules

| Module | Role |
|---|---|
| `builder` | `SceneBuilder`: takes a `&GridState` + `Theme` and produces a `SceneFrame` |
| `frame` | `SceneFrame`: ordered list of `ScenePrimitive` for one frame |
| `primitives` | Primitive types: `RectPrimitive`, `TextPrimitive`, `LinePrimitive`, `PolygonPrimitive` |
| `theme` | `Theme`: colours and sizes for rendering |

## Critical invariants

- This crate **knows nothing about Canvas2D, WebGL, or any renderer**. It
  produces data — it does not draw.
- `SceneFrame` is an immutable value produced each frame — no mutable internal
  state between frames.
- Always reason in **logical coordinates** (DPR-independent pixels).
  The renderer applies the `devicePixelRatio`.
- The order of primitives in `SceneFrame` defines the draw order (back-to-front).

## Available primitives

- `ScenePrimitive::Rect` — filled rectangle, optional stroke, optional rounded corners
- `ScenePrimitive::Text` — clipped text, left/right alignment
- `ScenePrimitive::Line` — line segment
- `ScenePrimitive::Polygon` — filled convex polygon, optional rounded corners

## Cell content rendering (`builder/cells.rs`)

`emit_cell` dispatches on `ColumnDef.format`. Composite renderers build their
output from the primitives above — no new primitive type is added:

- `CellFormat::ProgressBar` → `emit_progress_bar`: a track `Rect`
  (`Theme::progress_track`) + a fill `Rect` scaled by the value fraction (colour
  from the resolved class `background`, else `Theme::progress_fill`) + an
  optional right-aligned `"NN%"` label. Geometry via `Theme::progress_height`
  and `Theme::progress_radius`.

## Adding a primitive

1. Add the struct in `primitives.rs`
2. Add the variant in `ScenePrimitive`
3. Implement rendering in `rs-grid-render-canvas/src/renderer.rs`

## Useful commands

```sh
cargo test -p rs-grid-scene
cargo clippy -p rs-grid-scene -- -D warnings
cargo bench -p rs-grid-scene --bench scene_builder
cargo bench -p rs-grid-scene --bench scroll_frame
```
