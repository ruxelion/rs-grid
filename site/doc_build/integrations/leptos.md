# Leptos Integration

`rs-grid-leptos` provides a `<GridCanvas>` component for Leptos CSR
applications. It wraps the WASM runtime, canvas lifecycle, event handling, and
theming in a single component.

## Component API

```rust
<GridCanvas
    rows=1_000_000_u64
    cols=50_usize
    row_height=32.0_f64     // optional, default 32px
    header_height=40.0_f64  // optional, default 40px
/>
```

### Props

| Prop            | Type    | Default  | Description                           |
| --------------- | ------- | -------- | ------------------------------------- |
| `rows`          | `u64`   | required | Total number of data rows             |
| `cols`          | `usize` | required | Total number of columns               |
| `row_height`    | `f64`   | `32.0`   | Height of each data row in CSS pixels |
| `header_height` | `f64`   | `40.0`   | Height of the column header row       |

## Theming

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

## Events

The Leptos component attaches pointer and wheel listeners to the canvas:

| Browser event         | GridCommand                                 |
| --------------------- | ------------------------------------------- |
| `pointerdown`         | `SelectCell` / `SelectRow` / `SelectColumn` |
| `pointerdown` + Shift | `ExtendSelection`                           |
| `wheel`               | `ScrollTo`                                  |
| `ResizeObserver`      | `Resize`                                    |

Events are translated to `GridCommand` values and applied on the next animation
frame. You do not need to manage the event loop manually.

## Full example

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
