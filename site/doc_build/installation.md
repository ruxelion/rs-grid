# Installation

## Prerequisites

1. **Rust toolchain**

   Install Rust via [rustup](https://rustup.rs/):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **WASM target**

   Add the WebAssembly compilation target:

   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **Trunk (Leptos users)**

   Install [Trunk](https://trunkrs.dev/) for building and serving Leptos apps:

   ```bash
   cargo install trunk
   ```

## Add rs-grid to your project


**Leptos**

Add the required crates to your `Cargo.toml`:
```toml
[dependencies]
rs-grid-core   = { path = "../crates/rs-grid-core" }
rs-grid-scene  = { path = "../crates/rs-grid-scene" }
rs-grid-web    = { path = "../crates/rs-grid-web" }
rs-grid-leptos = { path = "../crates/rs-grid-leptos" }
```
:::note
rs-grid is not yet published on crates.io. Use local path dependencies
or git dependencies for now.
:::


**Vanilla JS**

Build the WASM package with `wasm-pack`:
```bash
cd crates/rs-grid-web
wasm-pack build --target web
```
Then import the ES module in your HTML:
```html
<script type="module">
  import init, { JsGrid } from './pkg/rs_grid_web.js';
  await init();
  const grid = new JsGrid(canvas, 1000, 10);
</script>
```


## Crate dependency graph

The dependency flows in one direction — never introduce reverse dependencies:

```
rs-grid-leptos → rs-grid-web → rs-grid-render-canvas → rs-grid-scene → rs-grid-core
```

| Crate                   | Role                                                                             |
| ----------------------- | -------------------------------------------------------------------------------- |
| `rs-grid-core`          | Headless logic: model, viewport, selection, hit-testing. **No WASM dependency.** |
| `rs-grid-scene`         | Converts `GridState` into renderer-agnostic `ScenePrimitive` values              |
| `rs-grid-render-canvas` | Canvas2D backend via `wasm-bindgen`                                              |
| `rs-grid-web`           | Browser integration: events, DPR, rAF loop, CSS theme                            |
| `rs-grid-leptos`        | Leptos CSR wrapper component (`<GridCanvas>`)                                    |

## Verify your setup

```bash
# Check the entire workspace compiles
cargo check --workspace

# Run unit tests
cargo test --workspace

# Build and serve the demo app
cd examples/basic-leptos
trunk serve
```
