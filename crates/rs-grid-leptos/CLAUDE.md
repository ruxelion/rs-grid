# rs-grid-leptos

Leptos CSR wrapper around `rs-grid-web`. Exposes a `<GridCanvas>` component
for use in Leptos applications.

## Public API

```rust
#[component]
pub fn GridCanvas(
    model: GridModel,
    #[prop(default = "100%".into())] width: String,
    #[prop(default = "600px".into())] height: String,
    #[prop(optional)] theme: Option<Signal<Theme>>,
    #[prop(optional)] locale: Option<Signal<Locale>>,
    #[prop(optional)] on_mount: Option<Box<dyn FnOnce(WebGridCanvas)>>,
    #[prop(optional)] on_validation_error: Option<ValidationErrorCb>,
) -> impl IntoView
```

## Behaviour

- Mounts the grid via `rs_grid_web::GridCanvas::mount()` inside an `Effect::new`.
- The `model` is consumed (moved) on first render — `GridModel` is intentionally
  not `Clone` (because `FnDataSource` closures are not clonable).
- The default theme is read from CSS variables via `theme_from_css_vars()`.
- Canvas dimensions are resolved from `getBoundingClientRect()` at mount time,
  with a fallback to `window.inner_width/height`.

## Critical invariants

- **CSR only** — no SSR. Do not access the DOM outside an `Effect` or a callback.
- The `model_slot: RefCell<Option<GridModel>>` is intentional: it allows moving
  the model into the `Effect` without `Clone`. Do not remove it.
- Do not expose `GridState` as a Leptos signal — mutations go through DOM events
  handled by `rs-grid-web`.

## Usage in a Leptos app

```rust
use rs_grid_leptos::GridCanvas;
use rs_grid_core::model::GridModel;

view! {
    <GridCanvas
        model=my_model
        width="100%"
        height="500px"
    />
}
```
