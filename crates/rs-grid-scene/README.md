# rs-grid-scene

Scene graph layer for [rs-grid](https://rs-grid.com). Converts a `GridState` into renderer-agnostic drawing primitives (`SceneFrame`).

This crate knows nothing about Canvas2D or any renderer — it produces data, it does not draw.

## Primitives

- `Rect` — filled rectangle with optional stroke and rounded corners
- `Text` — clipped text with left/right alignment
- `Line` — line segment
- `Polygon` — filled convex polygon with optional rounded corners

## Usage

```rust
use rs_grid_scene::{builder::SceneBuilder, theme::Theme};

let theme = Theme::light();
let frame = SceneBuilder::new(&grid_state, &theme).build();
// Pass frame to a renderer (e.g. rs-grid-render-canvas)
```

[Documentation](https://rs-grid.com/getting-started.html) · [Repository](https://github.com/ruxelion/rs-grid)

## License

MIT
