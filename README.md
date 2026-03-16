# rs-grid

A high-performance Rust/WASM data grid engine.

## Architecture

```
GridState  ──►  SceneBuilder  ──►  SceneFrame  ──►  CanvasRenderer  ──►  <canvas>
```

| Crate | Role |
|---|---|
| `rs-grid-core` | Headless grid logic — model, viewport, selection, hit-testing |
| `rs-grid-scene` | Scene graph — converts `GridState` into renderer-agnostic primitives |
| `rs-grid-render-canvas` | Canvas2D backend |
| `rs-grid-web` | Browser event handling, DPR sizing, rAF loop |
| `rs-grid-leptos` | Leptos CSR component wrapper |

## Quick start (basic-web)

```sh
# Install wasm-pack if needed
cargo install wasm-pack

# Build the WASM package
cd examples/basic-web
wasm-pack build --target web --out-dir pkg

# Serve locally (any static server)
npx serve .
```

Then open `http://localhost:3000`.

## Features

- Viewport virtualisation — only visible rows are computed and drawn
- Overscan rows — smooth scrolling without blank rows
- Precomputed column offsets — O(log n) hit testing
- Device pixel ratio aware — crisp on HiDPI screens
- Renderer agnostic — swap Canvas2D for WebGPU without touching core logic
