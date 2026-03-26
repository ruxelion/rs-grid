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

## Adding support for a new primitive

Add a `match` arm in `renderer.rs` for the new `ScenePrimitive` variant.
Do not modify `rs-grid-scene` from this crate.

## Build

```sh
# This crate only compiles for WASM targets
wasm-pack build --target web
```
