# Custom Renderer

## Overview

rs-grid's architecture is **renderer-agnostic**. The scene layer produces
a `SceneFrame` — a flat list of drawing primitives — that any rendering
backend can consume. The built-in Canvas2D renderer is just one implementation.

## Architecture

```
GridState → SceneBuilder → SceneFrame → [Your Renderer] → output
```

Your custom renderer only depends on `rs-grid-scene`. It does **not** need
`rs-grid-core`, `rs-grid-web`, or `rs-grid-render-canvas`.

## Steps

### 1. Add the dependency

```toml
[dependencies]
rs-grid-scene = { path = "../crates/rs-grid-scene" }
```

### 2. Build a SceneFrame

```rust
use rs_grid_scene::{SceneBuilder, Theme};

let builder = SceneBuilder::new(device_pixel_ratio, theme);
let frame = builder.build(&grid_state, None);
```

### 3. Iterate primitives

```rust
use rs_grid_scene::primitives::ScenePrimitive;

for prim in frame.primitives() {
    match prim {
        ScenePrimitive::Rect(r) => {
            // Draw filled rectangle at (r.x, r.y, r.width, r.height)
            // with r.fill color, optional r.stroke, r.corner_radius
        }
        ScenePrimitive::Text(t) => {
            // Draw t.text at (t.x, t.y) with t.color, t.font_size
            // Apply t.clip if present, respect t.align
        }
        ScenePrimitive::Line(l) => {
            // Draw line from (l.x1, l.y1) to (l.x2, l.y2)
            // with l.color and l.width
        }
        ScenePrimitive::Polygon(p) => {
            // Draw filled polygon with p.points, p.fill, p.corner_radius
        }
        ScenePrimitive::Image(img) => {
            // Load and draw image from img.url at (img.x, img.y)
            // with img.width, img.height, img.corner_radius
        }
    }
}
```

## Coordinate system

All coordinates are in **logical pixels** (CSS pixels). Your renderer must
apply the device pixel ratio (DPR) scaling:

```
physical_pixel = logical_pixel × dpr
```

## Draw order

Primitives in `SceneFrame` are ordered **back-to-front**. Draw them in
sequence — later primitives should paint over earlier ones.

## Rules

- **Do not modify** `rs-grid-core` or `rs-grid-scene`
- `SceneFrame` is **immutable** — produced fresh each frame
- All state lives in `GridState`, not in your renderer
