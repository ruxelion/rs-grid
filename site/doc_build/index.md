Open Source · Rust · WebAssembly# The data grid engine
built for performance

Virtualized rendering on Canvas2D, compiled to WebAssembly from Rust. Handles millions of rows with O(log n) hit-testing and 60 fps scrolling.

[Get started](/getting-started)[View on GitHub](https://github.com/bpodwinski/rs-grid)10M+rows virtualized60fpscanvas renderingO(log n)hit-testing5focused cratesWhy rs-grid## Built for the hard constraints

Most grid libraries struggle past 100k rows. rs-grid is designed from the ground up for virtualization, performance, and long-term maintainability.

### Virtualized viewport

Only visible cells are rendered. Memory usage stays constant regardless of row count.

### Zero-copy Rust core

rs-grid-core has no WASM dependency. Pure Rust logic, fully testable natively with cargo test.

### Renderer-agnostic

Scene primitives are decoupled from rendering. Swap Canvas2D for WebGL or any future backend without touching core logic.

### Leptos integration

Drop-in <GridCanvas> component for Leptos CSR. CSS-variable theming, reactive props, zero boilerplate.

Architecture## One direction, no surprises

A strict unidirectional dependency graph keeps each crate focused and independently testable.

GridStatemodel · viewport · selection→SceneBuilderrs-grid-scene→SceneFrameprimitives→CanvasRendererrs-grid-render-canvas→<canvas>browser`rs-grid-core`Headless logic: model, viewport, selection, hit-testing. No WASM dependency.

`rs-grid-scene`Converts GridState to renderer-agnostic ScenePrimitive list.

`rs-grid-render-canvas`Canvas2D backend via wasm-bindgen. Draws primitives to the DOM.

`rs-grid-web`Browser glue: events, DPR, rAF loop, CSS theme parsing.

`rs-grid-leptos`Leptos CSR component wrapping the full pipeline.

## Start building today

Open source, MIT license. Contributions welcome.

[Read the docs](/getting-started)[GitHub ↗](https://github.com/bpodwinski/rs-grid)