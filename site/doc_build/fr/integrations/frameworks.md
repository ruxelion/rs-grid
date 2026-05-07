# Intégration Framework

## Démarrage rapide


**[object Object]**

`rs-grid-leptos` fournit un composant `<GridCanvas>` pour les applications
Leptos CSR. Il encapsule le runtime WASM, le cycle de vie du canvas, la
gestion des événements et le theming dans un seul composant.
```rust
<GridCanvas
    rows=1_000_000_u64
    cols=50_usize
    row_height=32.0_f64     // optionnel, défaut 32px
    header_height=40.0_f64  // optionnel, défaut 40px
/>
```


**[object Object]**

rs-grid peut être utilisé sans framework via la classe `JsGrid` exportée
par `rs-grid-web`. Compilez avec `wasm-pack` :
```bash
cd crates/rs-grid-web
wasm-pack build --target web
```
Cela produit un module ES dans `pkg/` :
- `rs_grid_web.js` — le code de liaison JS
- `rs_grid_web_bg.wasm` — le binaire WASM


**[object Object]**

`rs-grid-dioxus` fournit un composant `GridCanvas` pour les applications
Dioxus CSR. Il encapsule le runtime WASM, le cycle de vie du canvas, la
gestion des evenements et le theming dans un seul composant.
```rust
rsx! {
    GridCanvas {
        model: ModelSlot::new(model),
        width: "100%",
        height: "600px",
    }
}
```


**[object Object]**

`rs-grid-yew` fournit un composant fonction `GridCanvas` pour les applications
Yew CSR. Il encapsule le runtime WASM, le cycle de vie du canvas, la gestion
des evenements et le theming dans un seul composant.
```rust
use rs_grid_yew::{GridCanvas, wrap_model};

let model = wrap_model(my_model);
html! {
    <GridCanvas model={model}
        width="100%" height="600px" />
}
```


## API du composant


**[object Object]**

### Props
| Prop            | Type    | Défaut | Description                                      |
| --------------- | ------- | ------ | ------------------------------------------------ |
| `rows`          | `u64`   | requis | Nombre total de lignes de données                |
| `cols`          | `usize` | requis | Nombre total de colonnes                         |
| `row_height`    | `f64`   | `32.0` | Hauteur de chaque ligne de données en pixels CSS |
| `header_height` | `f64`   | `40.0` | Hauteur de la ligne d'en-tête des colonnes       |


**[object Object]**

### API JsGrid
| Méthode                          | Description                                                                                                                                 |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `new JsGrid(canvas, rows, cols)` | Monte une grille sur un élément canvas                                                                                                      |
| `detach()`                       | Démonte la grille et supprime les écouteurs d'événements                                                                                    |
| `export_patches()`               | Exporte les cellules editees en TSV versionne — voir [Persister les modifications](/fr/data/extracting-data.md#persister-les-modifications) |
| `import_patches(tsv)`            | Importe des patches TSV dans la grille — voir [Persister les modifications](/fr/data/extracting-data.md#persister-les-modifications)        |


**[object Object]**

### Props
| Prop                  | Type                                  | Defaut    | Description                                         |
| --------------------- | ------------------------------------- | --------- | --------------------------------------------------- |
| `model`               | `ModelSlot`                           | requis    | Modele de grille encapsule dans `ModelSlot::new()`  |
| `width`               | `String`                              | `"100%"`  | Largeur CSS                                         |
| `height`              | `String`                              | `"600px"` | Hauteur CSS                                         |
| `theme`               | `Option<Signal<Theme>>`               | `None`    | Signal de theme reactif optionnel                   |
| `locale`              | `Option<Signal<Locale>>`              | `None`    | Signal de locale reactif optionnel                  |
| `on_mount`            | `EventHandler<WebGridCanvas>`         | no-op     | Appele apres le montage avec le handle de la grille |
| `on_validation_error` | `EventHandler<(u64, String, String)>` | no-op     | Callback d'erreur de validation                     |


**[object Object]**

### Props
| Prop                  | Type                              | Defaut    | Description                                         |
| --------------------- | --------------------------------- | --------- | --------------------------------------------------- |
| `model`               | `Rc<RefCell<Option<GridModel>>>`  | requis    | Modele de grille encapsule via `wrap_model()`       |
| `width`               | `AttrValue`                       | `"100%"`  | Largeur CSS                                         |
| `height`              | `AttrValue`                       | `"600px"` | Hauteur CSS                                         |
| `theme`               | `Option<Theme>`                   | `None`    | Theme optionnel                                     |
| `locale`              | `Option<Locale>`                  | `None`    | Locale optionnelle                                  |
| `on_mount`            | `Option<Callback<WebGridCanvas>>` | `None`    | Appele apres le montage avec le handle de la grille |
| `on_validation_error` | `Option<ValidationErrorCb>`       | `None`    | Callback d'erreur de validation                     |


## Thème


**[object Object]**

Le composant lit sa palette de couleurs à partir des propriétés CSS
personnalisées au moment du montage via
`rs-grid-web::theme_from_css_vars`. Définissez les variables dans votre
feuille de style :
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
Incluez le fichier via votre `Trunk.toml` ou une balise `<link>` dans
`index.html`.


**[object Object]**

`JsGrid` lit les variables CSS au moment du montage, tout comme l'intégration
Leptos. Ajoutez les variables `--rs-grid-*` à votre feuille de styles :
```css
:root {
  --rs-grid-bg: #1e1e2e;
  --rs-grid-cell-text: #cdd6f4;
  /* ... */
}
```
Consultez la [Référence des variables CSS](/fr/theming/css-variables.md) pour la liste complète.


**[object Object]**

Le composant lit sa palette de couleurs a partir des proprietes CSS
personnalisees au moment du montage via `theme_from_css_vars`. Memes
variables CSS que Leptos — definissez-les dans votre feuille de style :
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
Incluez le fichier via votre `Trunk.toml` ou une balise `<link>` dans
`index.html`.


**[object Object]**

Le composant lit sa palette de couleurs a partir des proprietes CSS
personnalisees au moment du montage via `theme_from_css_vars`. Memes
variables CSS que Leptos — definissez-les dans votre feuille de style :
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
Incluez le fichier via votre `Trunk.toml` ou une balise `<link>` dans
`index.html`.


## Événements


**[object Object]**

Le composant Leptos attache des écouteurs pointer et wheel au canvas :
| Événement navigateur  | GridCommand                                 |
| --------------------- | ------------------------------------------- |
| `pointerdown`         | `SelectCell` / `SelectRow` / `SelectColumn` |
| `pointerdown` + Shift | `ExtendSelection`                           |
| `wheel`               | `ScrollTo`                                  |
| `ResizeObserver`      | `Resize`                                    |
Les événements sont convertis en valeurs `GridCommand` et appliqués à la
prochaine frame d'animation. Vous n'avez pas besoin de gérer la boucle
d'événements manuellement.


**[object Object]**

`JsGrid` attache automatiquement les écouteurs pointer, wheel et resize à
l'élément canvas au moment du montage. Les événements sont convertis en
valeurs `GridCommand` en interne. Appelez `detach()` pour supprimer tous
les écouteurs et arrêter la boucle d'animation.


**[object Object]**

Le composant Dioxus monte la grille via `rs-grid-web`, qui attache
automatiquement les ecouteurs pointer, wheel et resize :
| Evenement navigateur  | GridCommand                                 |
| --------------------- | ------------------------------------------- |
| `pointerdown`         | `SelectCell` / `SelectRow` / `SelectColumn` |
| `pointerdown` + Shift | `ExtendSelection`                           |
| `wheel`               | `ScrollTo`                                  |
| `ResizeObserver`      | `Resize`                                    |
Les evenements sont convertis en valeurs `GridCommand` et appliques a la
prochaine frame d'animation. Vous n'avez pas besoin de gerer la boucle
d'evenements manuellement.


**[object Object]**

Le composant Yew monte la grille via `rs-grid-web`, qui attache
automatiquement les ecouteurs pointer, wheel et resize :
| Evenement navigateur  | GridCommand                                 |
| --------------------- | ------------------------------------------- |
| `pointerdown`         | `SelectCell` / `SelectRow` / `SelectColumn` |
| `pointerdown` + Shift | `ExtendSelection`                           |
| `wheel`               | `ScrollTo`                                  |
| `ResizeObserver`      | `Resize`                                    |
Les evenements sont convertis en valeurs `GridCommand` et appliques a la
prochaine frame d'animation. Vous n'avez pas besoin de gerer la boucle
d'evenements manuellement.


## Exemple complet


**[object Object]**

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
Le fichier de thème de référence se trouve dans
`examples/basic-leptos/rs-grid-theme.css`. Copiez-le comme point de départ
pour votre propre thème.
:::


**[object Object]**

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
    // La grille est active avec 1000 lignes × 10 colonnes
  </script>
</body>
</html>
```


**[object Object]**

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
Le fichier de theme de reference se trouve dans
`examples/basic-leptos/rs-grid-theme.css`. Copiez-le comme point de depart
pour votre propre theme.
:::


**[object Object]**

```rust title="src/main.rs"
use yew::prelude::*;
use rs_grid_yew::{GridCanvas, wrap_model};
use rs_grid_core::model::GridModel;

#[function_component]
fn App() -> Html {
    let model = wrap_model(GridModel::new(500_000, 20));
    html! {
        <main style="width:100vw;height:100vh;">
            <GridCanvas model={model} />
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
```
:::tip
Le fichier de theme de reference se trouve dans
`examples/basic-leptos/rs-grid-theme.css`. Copiez-le comme point de depart
pour votre propre theme.
:::


## Limitations


**[object Object]**

- rs-grid-leptos est CSR uniquement — le SSR n'est pas supporté
- Le composant s'attend à être rendu dans un environnement navigateur avec support `<canvas>`


**[object Object]**

- Les définitions de colonnes utilisent des libellés par défaut (`Column 0`, `Column 1`, etc.)
- Les données sont générées par une fonction de hachage (mode démo)
- Pour un contrôle complet sur les colonnes et les données, utilisez l'API Rust directement
:::note
L'API vanilla JS est un point d'entrée léger pour les démos et les cas
d'usage simples. Pour les applications en production avec des sources de
données personnalisées et des définitions de colonnes, utilisez
l'intégration Leptos ou construisez une intégration personnalisée
par-dessus `rs-grid-web`.
:::


**[object Object]**

- rs-grid-dioxus est CSR uniquement — le SSR n'est pas supporte
- Le composant s'attend a etre rendu dans un environnement navigateur avec support `<canvas>`
- `GridModel` n'est pas `Clone` — utilisez `ModelSlot::new()` pour l'encapsuler


**[object Object]**

- rs-grid-yew est CSR uniquement — le SSR n'est pas supporte
- Le composant s'attend a etre rendu dans un environnement navigateur avec support `<canvas>`
- `GridModel` n'est pas `Clone` — utilisez `wrap_model()` pour l'encapsuler

