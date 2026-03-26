# Framework Integration

## Quick start


**Leptos**

`rs-grid-leptos` provides a `<GridCanvas>` component for Leptos CSR
applications. It wraps the WASM runtime, canvas lifecycle, event handling, and
theming in a single component.
```rust
<GridCanvas
    rows=1_000_000_u64
    cols=50_usize
    row_height=32.0_f64     // optional, default 32px
    header_height=40.0_f64  // optional, default 40px
/>
```


**Vanilla JS**

rs-grid can be used without any framework via the `JsGrid` class exported
by `rs-grid-web`. Build with `wasm-pack`:
```bash
cd crates/rs-grid-web
wasm-pack build --target web
```
This produces an ES module in `pkg/`:
- `rs_grid_web.js` — the JS glue
- `rs_grid_web_bg.wasm` — the WASM binary


**Dioxus**

`rs-grid-dioxus` provides a `GridCanvas` component for Dioxus CSR
applications. It wraps the WASM runtime, canvas lifecycle, event handling, and
theming in a single component.
```rust
rsx! {
    GridCanvas {
        model: ModelSlot::new(model),
        width: "100%",
        height: "600px",
    }
}
```


## Component API


**Leptos**

### Props
| Prop            | Type    | Default  | Description                           |
| --------------- | ------- | -------- | ------------------------------------- |
| `rows`          | `u64`   | required | Total number of data rows             |
| `cols`          | `usize` | required | Total number of columns               |
| `row_height`    | `f64`   | `32.0`   | Height of each data row in CSS pixels |
| `header_height` | `f64`   | `40.0`   | Height of the column header row       |


**Vanilla JS**

### JsGrid API
| Method                           | Description                          |
| -------------------------------- | ------------------------------------ |
| `new JsGrid(canvas, rows, cols)` | Mount a grid on a canvas element     |
| `detach()`                       | Unmount and clean up event listeners |
| `export_patches()`               | Export edited cell values as TSV     |
| `import_patches(tsv)`            | Import TSV patches into the grid     |


**Dioxus**

### Props
| Prop                  | Type                                  | Default   | Description                              |
| --------------------- | ------------------------------------- | --------- | ---------------------------------------- |
| `model`               | `ModelSlot`                           | required  | Grid model wrapped in `ModelSlot::new()` |
| `width`               | `String`                              | `"100%"`  | CSS width                                |
| `height`              | `String`                              | `"600px"` | CSS height                               |
| `theme`               | `Option<Signal<Theme>>`               | `None`    | Optional reactive theme signal           |
| `locale`              | `Option<Signal<Locale>>`              | `None`    | Optional reactive locale signal          |
| `on_mount`            | `EventHandler<WebGridCanvas>`         | no-op     | Called after mount with the grid handle  |
| `on_validation_error` | `EventHandler<(u64, String, String)>` | no-op     | Validation error callback                |


## Theming


**Leptos**

The component reads its color palette from CSS custom properties at mount time
via `rs-grid-web::theme_from_css_vars`. Define the variables in your
stylesheet:
```css title="rs-grid-theme.css"
:root {
  --rs-grid-bg:               #0d1117;
  --rs-grid-header-bg:        #161b22;
  --rs-grid-border:           #30363d;
  --rs-grid-text:             #c9d1d9;
  --rs-grid-selection-bg:     rgba(56, 139, 253, 0.15);
  --rs-grid-selection-border: #388bfd;
}
```
Include the file via your `Trunk.toml` or a `<link>` tag in `index.html`.


**Vanilla JS**

`JsGrid` reads CSS variables at mount time, just like the Leptos integration.
Add `--rs-grid-*` variables to your stylesheet:
```css
:root {
  --rs-grid-bg: #1e1e2e;
  --rs-grid-cell-text: #cdd6f4;
  /* ... */
}
```
See [CSS Variables Reference](/theming/css-variables.md) for the full list.


**Dioxus**

The component reads its color palette from CSS custom properties at mount time
via `theme_from_css_vars`. Same CSS variables as Leptos — define them in
your stylesheet:
```css title="rs-grid-theme.css"
:root {
  --rs-grid-bg:               #0d1117;
  --rs-grid-header-bg:        #161b22;
  --rs-grid-border:           #30363d;
  --rs-grid-text:             #c9d1d9;
  --rs-grid-selection-bg:     rgba(56, 139, 253, 0.15);
  --rs-grid-selection-border: #388bfd;
}
```
Include the file via your `Trunk.toml` or a `<link>` tag in `index.html`.


## Events


**Leptos**

The Leptos component attaches pointer and wheel listeners to the canvas:
| Browser event         | GridCommand                                 |
| --------------------- | ------------------------------------------- |
| `pointerdown`         | `SelectCell` / `SelectRow` / `SelectColumn` |
| `pointerdown` + Shift | `ExtendSelection`                           |
| `wheel`               | `ScrollTo`                                  |
| `ResizeObserver`      | `Resize`                                    |
Events are translated to `GridCommand` values and applied on the next animation
frame. You do not need to manage the event loop manually.


**Vanilla JS**

`JsGrid` automatically attaches pointer, wheel, and resize listeners to the
canvas element at mount time. Events are translated to `GridCommand` values
internally. Call `detach()` to remove all listeners and stop the animation loop.


**Dioxus**

The Dioxus component mounts the grid via `rs-grid-web`, which attaches
pointer, wheel, and resize listeners automatically:
| Browser event         | GridCommand                                 |
| --------------------- | ------------------------------------------- |
| `pointerdown`         | `SelectCell` / `SelectRow` / `SelectColumn` |
| `pointerdown` + Shift | `ExtendSelection`                           |
| `wheel`               | `ScrollTo`                                  |
| `ResizeObserver`      | `Resize`                                    |
Events are translated to `GridCommand` values and applied on the next animation
frame. You do not need to manage the event loop manually.


## Full example


**Leptos**

```rust title="src/main.rs"
use leptos::*;
use rs_grid_leptos::GridCanvas;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main style="width: 100vw; height: 100vh;">
            <GridCanvas
                rows=500_000_u64
                cols=20_usize
            />
        </main>
    }
}

fn main() {
    leptos::mount_to_body(App);
}
```
:::tip
The reference theme file is at `examples/basic-leptos/rs-grid-theme.css`.
Copy it as a starting point for your own theme.
:::


**Vanilla JS**

```html title="index.html"
<!DOCTYPE html>
<html>
<head>
  <style>
    canvas { width: 100%; height: 600px; }
  </style>
</head>
<body>
  <canvas id="grid"></canvas>
  <script type="module">
    import init, { JsGrid } from './pkg/rs_grid_web.js';

    await init();

    const canvas = document.getElementById('grid');
    const grid = new JsGrid(canvas, 1000, 10);
    // Grid is now live with 1000 rows × 10 columns
  </script>
</body>
</html>
```


**Dioxus**

```rust title="src/main.rs"
use dioxus::prelude::*;
use rs_grid_dioxus::{GridCanvas, ModelSlot};
use rs_grid_core::model::GridModel;

fn App() -> Element {
    let model = use_hook(|| {
        ModelSlot::new(GridModel::new(500_000, 20))
    });
    rsx! {
        main { style: "width:100vw;height:100vh;",
            GridCanvas { model: model.clone() }
        }
    }
}

fn main() {
    dioxus::launch(App);
}
```
:::tip
The reference theme file is at `examples/basic-leptos/rs-grid-theme.css`.
Copy it as a starting point for your own theme.
:::


## Limitations


**Leptos**

- rs-grid-leptos is CSR-only — SSR is not supported
- The component expects to be rendered in a browser environment with `<canvas>` support


**Vanilla JS**

- Column definitions use default labels (`Column 0`, `Column 1`, etc.)
- Data is generated with a hash function (demo mode)
- For full control over columns and data, use the Rust API directly
:::note
The vanilla JS API is a lightweight entry point for demos and simple
use cases. For production applications with custom data sources and
column definitions, use the Leptos integration or build a custom
integration on top of `rs-grid-web`.
:::


**Dioxus**

- rs-grid-dioxus is CSR-only — SSR is not supported
- The component expects to be rendered in a browser environment with `<canvas>` support
- `GridModel` is not `Clone` — use `ModelSlot::new()` to wrap it

