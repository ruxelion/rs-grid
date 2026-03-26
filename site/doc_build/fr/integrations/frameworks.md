# Intégration Framework

## Démarrage rapide


**Leptos**

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


**Vanilla JS**

rs-grid peut être utilisé sans framework via la classe `JsGrid` exportée
par `rs-grid-web`. Compilez avec `wasm-pack` :
```bash
cd crates/rs-grid-web
wasm-pack build --target web
```
Cela produit un module ES dans `pkg/` :
- `rs_grid_web.js` — le code de liaison JS
- `rs_grid_web_bg.wasm` — le binaire WASM


## API du composant


**Leptos**

### Props
| Prop            | Type    | Défaut | Description                                      |
| --------------- | ------- | ------ | ------------------------------------------------ |
| `rows`          | `u64`   | requis | Nombre total de lignes de données                |
| `cols`          | `usize` | requis | Nombre total de colonnes                         |
| `row_height`    | `f64`   | `32.0` | Hauteur de chaque ligne de données en pixels CSS |
| `header_height` | `f64`   | `40.0` | Hauteur de la ligne d'en-tête des colonnes       |


**Vanilla JS**

### API JsGrid
| Méthode                          | Description                                              |
| -------------------------------- | -------------------------------------------------------- |
| `new JsGrid(canvas, rows, cols)` | Monte une grille sur un élément canvas                   |
| `detach()`                       | Démonte la grille et supprime les écouteurs d'événements |
| `export_patches()`               | Exporte les valeurs de cellules modifiées au format TSV  |
| `import_patches(tsv)`            | Importe des modifications TSV dans la grille             |


## Thème


**Leptos**

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


**Vanilla JS**

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


## Événements


**Leptos**

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


**Vanilla JS**

`JsGrid` attache automatiquement les écouteurs pointer, wheel et resize à
l'élément canvas au moment du montage. Les événements sont convertis en
valeurs `GridCommand` en interne. Appelez `detach()` pour supprimer tous
les écouteurs et arrêter la boucle d'animation.


## Exemple complet


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
Le fichier de thème de référence se trouve dans
`examples/basic-leptos/rs-grid-theme.css`. Copiez-le comme point de départ
pour votre propre thème.
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
    // La grille est active avec 1000 lignes × 10 colonnes
  </script>
</body>
</html>
```


## Limitations


**Leptos**

- rs-grid-leptos est CSR uniquement — le SSR n'est pas supporté
- Le composant s'attend à être rendu dans un environnement navigateur avec support `<canvas>`


**Vanilla JS**

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

