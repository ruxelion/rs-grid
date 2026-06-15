# rs-grid-dioxus

Dioxus CSR wrapper around `rs-grid-web`. Exposes a `GridCanvas` component
for use in Dioxus applications.

## Public API

```rust
#[component]
pub fn GridCanvas(
    model: GridModel,
    #[props(default = "100%".into())] width: String,
    #[props(default = "600px".into())] height: String,
    #[props(optional)] theme: Option<Signal<Theme>>,
    #[props(optional)] locale: Option<Signal<Locale>>,
    #[props(optional)] on_mount: Option<Box<dyn FnOnce(WebGridCanvas)>>,
    #[props(optional)] on_validation_error: Option<ValidationErrorCb>,
) -> Element
```

## Behaviour

- Mounts the grid via `rs_grid_web::GridCanvas::mount()` inside the
  `onmounted` event handler.
- The `model` is consumed (moved) on first mount — `GridModel` is
  intentionally not `Clone` (because `FnDataSource` closures are not
  clonable).
- The default theme is read from CSS variables via `theme_from_css_vars()`.
- Canvas dimensions are resolved from `getBoundingClientRect()` at mount
  time, with a fallback to `window.inner_width/height`.
- Each instance gets a unique `id` attribute via a global `AtomicU32`
  counter, supporting multiple grids on one page.

## Critical invariants

- **CSR only** — no SSR. Do not access the DOM outside `onmounted` or an
  effect.
- The `model_slot: RefCell<Option<GridModel>>` is intentional: it allows
  moving the model into the `onmounted` handler without `Clone`. Do not
  remove it.
- No `SendWrapper` needed — Dioxus WASM does not impose `Send` on
  `use_drop` closures.
- Do not expose `GridState` as a Dioxus signal — mutations go through
  DOM events handled by `rs-grid-web`.

## Usage in a Dioxus app

```rust
use rs_grid_dioxus::GridCanvas;
use rs_grid_core::model::GridModel;

rsx! {
    GridCanvas {
        model: my_model,
        width: "100%",
        height: "500px",
    }
}
```
