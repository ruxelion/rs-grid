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
| `css_vars` | `Theme` ↔ `--rs-grid-*` CSS variables (writer + reader, single source of truth; round-trip test enforces parity) |

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

### Locked-cell visual feedback

`emit_cell` takes a `model: &GridModel` parameter (in addition to `col`)
so it can call `ColumnDef::is_cell_editable(row, model)` — the single
source of truth combining `GridModel.editable` (grid-wide toggle), the
static `editable` flag, and the dynamic `editable_predicate`
(rs-grid-core). When a cell resolves to non-editable:

- A `locked_cell_bg` overlay `Rect` is pushed right before the cell's
  content, **regardless of `CellFormat`** — it applies to
  `Styled`/`Image`/`ImageText`/`ProgressBar` composite cells exactly the
  same as plain text, since it's pushed before the format dispatch.
  Skipped entirely when `Theme::locked_cell_bg.a == 0` (mirrors the
  `row_hover_bg` "transparent = disabled" convention — no extra draw call
  for themes that don't opt in).
- The **text color** swap (`Theme::locked_cell_text` instead of
  `Theme::cell_text`) is scoped to the default plain-text renderer only —
  the `Styled`/`Image`/`ImageText`/`ProgressBar` composite branches keep
  their own colors (badge class, image, progress-bar fill). Extending the
  text-color treatment to those is a separate, explicit follow-up if
  needed; the background wash already gives every locked cell a visible
  affordance regardless of format.

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
