# rs-grid-leptos

Wrapper Leptos CSR autour de `rs-grid-web`. Expose un composant `<GridCanvas>`
utilisable dans une application Leptos.

## API publique

```rust
#[component]
pub fn GridCanvas(
    model: GridModel,
    #[prop(default = "100%".into())] width: String,
    #[prop(default = "600px".into())] height: String,
    #[prop(optional)] theme: Option<Theme>,
) -> impl IntoView
```

## Comportement

- Monte la grille via `rs_grid_web::GridCanvas::mount()` dans un `Effect::new`.
- Le `model` est consommé (moved) au premier render — `GridModel` n'est pas `Clone`
  intentionnellement (les `FnDataSource` ne sont pas clonables).
- Le thème par défaut est lu depuis les variables CSS via `theme_from_css_vars()`.
- Les dimensions du canvas sont résolues depuis `getBoundingClientRect()` au
  moment du mount, avec fallback sur `window.inner_width/height`.

## Invariants critiques

- **CSR uniquement** — pas de SSR. Ne pas essayer d'accéder au DOM hors d'un
  `Effect` ou d'un callback.
- Le `model_slot: RefCell<Option<GridModel>>` est intentionnel : il permet de
  move le model dans le `Effect` sans `Clone`. Ne pas le supprimer.
- Ne pas exposer `GridState` comme signal Leptos — les mutations passent par
  les events DOM gérés par `rs-grid-web`.

## Usage dans une app Leptos

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
