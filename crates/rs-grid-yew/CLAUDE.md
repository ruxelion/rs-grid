# rs-grid-yew

Yew CSR wrapper around `rs-grid-web`. Exposes a `GridCanvas` function
component for use in Yew applications.

## Public API

```rust
#[function_component]
pub fn GridCanvas(props: &GridCanvasProps) -> Html

pub struct GridCanvasProps {
    pub model: Rc<RefCell<Option<GridModel>>>,
    pub width: AttrValue,       // default "100%"
    pub height: AttrValue,      // default "600px"
    pub theme: Option<Theme>,
    pub locale: Option<Locale>,
    pub on_mount: Option<Callback<WebGridCanvas>>,
    pub on_validation_error: Option<ValidationErrorCb>,
}

pub fn wrap_model(model: GridModel) -> Rc<RefCell<Option<GridModel>>>;
```

## Behaviour

- Mounts the grid via `rs_grid_web::GridCanvas::mount()` inside
  `use_effect_with(())` (runs once after first render).
- The `model` is consumed via `Rc<RefCell<Option>>` on first mount —
  `GridModel` is intentionally not `Clone`.
- The default theme is read from CSS variables via `theme_from_css_vars()`.
- Canvas dimensions are resolved from `getBoundingClientRect()` at mount
  time, with a fallback to `window.inner_width/height`.
- Theme and locale changes are applied in-place via separate
  `use_effect_with` hooks.

## Critical invariants

- **CSR only** — no SSR.
- The `model` prop uses `Rc<RefCell<Option<GridModel>>>` because Yew
  `Properties` requires `PartialEq`, and `GridModel` is not `Clone` or
  `PartialEq`. Use `wrap_model()` for convenience.
- Do not expose `GridState` as Yew state — mutations go through DOM
  events handled by `rs-grid-web`.

## Usage in a Yew app

```rust
use rs_grid_yew::{GridCanvas, wrap_model};
use rs_grid_core::model::GridModel;

let model_slot = wrap_model(my_model);
html! {
    <GridCanvas model={model_slot}
                width="100%" height="500px" />
}
```
