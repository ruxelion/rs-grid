# rs-grid-render-canvas

Canvas2D rendering backend. Consumes a `SceneFrame` and draws onto a
`CanvasRenderingContext2d` via wasm-bindgen.

## Modules

| Module | Role |
|---|---|
| `renderer` | `CanvasRenderer`: iterates over `SceneFrame` and calls Canvas2D APIs |

## Critical invariants

- This crate **contains no business logic** — it translates primitives into
  Canvas2D calls, nothing more.
- Incoming coordinates are in **logical pixels**. The renderer applies the
  `devicePixelRatio` itself via a context transform (`scale(dpr, dpr)`).
- Always save/restore the Canvas context (`save()`/`restore()`) around
  clipping or transform operations.
- `PolygonPrimitive` with `corner_radius > 0` requires `arcTo` on each
  segment — verify behaviour on non-convex polygons.
- **`RectPrimitive` with a `stroke` insets the stroked path by half the
  stroke width, for both sharp and rounded rects.** Canvas always
  centers a stroke on the path it's given — stroking the exact same
  path used for the fill bleeds half the stroke width outside the
  rect's nominal bounds. For an opaque stroke this is easy to miss (a
  cell-bounds border quietly spilling half a pixel into the next
  cell); for a *translucent* one (e.g. `Theme::filter_row_input_border`)
  it's obvious — the bleeding half anti-aliases against whatever's
  behind the rect, reading as a soft/blurry edge instead of a crisp
  line. `draw_rect` fixes this by tracing the fill path at the given
  bounds, then — only if there's a stroke — tracing a **second**, inset
  path (`trace_rounded_rect`, factored out so it can be called twice at
  different bounds) before calling `stroke()`/`stroke_rect()`. When
  adding a new bordered primitive, reuse this pattern rather than
  stroking the fill path directly.

## Adding support for a new primitive

Add a `match` arm in `renderer.rs` for the new `ScenePrimitive` variant.
Do not modify `rs-grid-scene` from this crate.

## Useful commands

```sh
# This crate only compiles for WASM — do not run cargo build natively
cargo clippy -p rs-grid-render-canvas --target wasm32-unknown-unknown -- -D warnings

# Exercise via the e2e fixture (builds the full WASM bundle)
cd e2e/leptos-harness && trunk build
```
