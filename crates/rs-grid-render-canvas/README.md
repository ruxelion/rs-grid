# rs-grid-render-canvas

[![crates.io](https://img.shields.io/crates/v/rs-grid-render-canvas.svg)](https://crates.io/crates/rs-grid-render-canvas)

Canvas2D rendering backend for [rs-grid](https://rs-grid.com). Consumes a `SceneFrame` from `rs-grid-scene` and draws onto a `CanvasRenderingContext2d` via wasm-bindgen.

Incoming coordinates are in logical pixels. The renderer applies `devicePixelRatio` via a context transform for HiDPI screens.

## Usage

```rust
use rs_grid_render_canvas::renderer::CanvasRenderer;

let renderer = CanvasRenderer::new(context2d);
renderer.render(&scene_frame, device_pixel_ratio);
```

[Documentation](https://rs-grid.com/getting-started.html) · [Repository](https://github.com/ruxelion/rs-grid) · [crates.io](https://crates.io/crates/rs-grid-render-canvas)

## License

MIT
