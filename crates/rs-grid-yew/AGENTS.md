# rs-grid-yew

Yew CSR wrapper around `rs-grid-web`. Exposes a `GridCanvas` function
component for use in Yew applications.

## Public API

```rust
#[function_component]
pub fn GridCanvas(props: &GridCanvasProps) -> Html

pub struct ModelSlot;  // newtype wrapping Rc<RefCell<Option<GridModel>>>
impl ModelSlot {
    pub fn new(model: GridModel) -> Self;
}

pub struct GridCanvasProps {
    pub model: ModelSlot,            // wrap with ModelSlot::new
    pub width: AttrValue,            // default "100%"
    pub height: AttrValue,           // default "600px"
    pub theme: Option<Theme>,
    pub locale: Option<Locale>,
    pub on_mount: Option<Callback<WebGridCanvas>>,
    pub on_validation_error: Option<ValidationErrorCb>,
    pub on_validation_state_changed: Option<ValidationStateChangedCb>,
}

// Deprecated — use ModelSlot::new instead
pub fn wrap_model(model: GridModel) -> ModelSlot;
```

`on_validation_state_changed(Some((row, col_key, message)) | None)` fires
on every `StartEdit`/`ValidateEdit`/`CommitEdit`/`CancelEdit`, reflecting
the *live* validation state (every keystroke, not just commits) — unlike
`on_validation_error` which fires only when a commit is rejected. Use it
to drive a custom validation UI (tooltip, banner, icon) with your own
state/CSS; rs-grid does not impose one. A zero-config native `title`
tooltip is applied by `rs-grid-web` by default (see
`GridCanvas::set_native_validation_tooltip` on the raw handle from
`on_mount` to disable it).

## Behaviour

- Mounts the grid via `rs_grid_web::GridCanvas::mount()` inside
  `use_effect_with(())` (runs once after first render).
- The `model` is consumed via `ModelSlot` on first mount —
  `GridModel` is intentionally not `Clone`.
- The default theme is read from CSS variables via `theme_from_css_vars()`.
- Canvas dimensions are resolved from `getBoundingClientRect()` at mount
  time, with a fallback to `window.inner_width/height`.
- Theme and locale changes are applied in-place via separate
  `use_effect_with` hooks.

## Critical invariants

- **CSR only** — no SSR.
- The `model` prop uses `ModelSlot` because Yew `Properties` requires
  `PartialEq`, and `GridModel` is not `Clone` or `PartialEq`. Use
  `ModelSlot::new(model)` to construct it.
- Do not expose `GridState` as Yew state — mutations go through DOM
  events handled by `rs-grid-web`.

## Usage in a Yew app

```rust
use rs_grid_yew::{GridCanvas, ModelSlot};
use rs_grid_core::model::GridModel;

let slot = ModelSlot::new(my_model);
html! {
    <GridCanvas model={slot}
                width="100%" height="500px" />
}
```
