# Vue d'ensemble du theming

## Pipeline

```
CSS variables (:root) → theme_from_css_vars() → Theme struct → SceneBuilder → canvas
```

1. Vous definissez des proprietes CSS personnalisees `--rs-grid-*` sur `:root`
2. Au montage, `theme_from_css_vars()` les lit depuis le style calcule
3. Les variables manquantes utilisent les valeurs par defaut de `Theme::light()`
4. Le `Theme` est transmis au `SceneBuilder` qui l'utilise pour tout le rendu

## Structure Theme

La structure `Theme` contient toutes les proprietes visuelles :

- **Couleurs** — arriere-plan, texte, bordures, selection, scrollbar, survol, recherche
- **Typographie** — tailles de police, gras pour les en-tetes
- **Espacement** — padding des cellules, largeur et rayon de la scrollbar

Voir la [Reference des variables CSS](/fr/theming/css-variables.md) pour la liste complete.

## Deux facons de definir le theme

### 1. Variables CSS (recommande)

Ajoutez des variables a votre feuille de style :

```css
:root {
  --rs-grid-bg: #1a1b26;
  --rs-grid-cell-text: #c0caf5;
}
```

### 2. Structure Theme programmatique

Creez un `Theme` directement en Rust :

```rust
let theme = Theme {
    bg: Color::rgb(26, 27, 38),
    cell_text: Color::rgb(192, 202, 245),
    ..Theme::dark()
};
```

Pour appliquer le theme dynamiquement :


**[object Object]**

Passez le theme en tant que signal pour des mises a jour reactives :
```rust
let (theme, set_theme) = create_signal(Theme::dark());
// Mise a jour dynamique du theme
set_theme.set(Theme::light());
```


**[object Object]**

Remontez la grille avec le nouveau theme :
```rust
let theme = theme_from_css_vars(&canvas);
let gc = GridCanvas::mount(canvas, state, theme, locale);
```


**[object Object]**

Passez le theme en tant que signal pour des mises a jour reactives :
```rust
let mut theme = use_signal(|| Theme::dark());
// Mise a jour dynamique du theme
theme.set(Theme::light());
```


**[object Object]**

Passez le theme comme prop, ou utilisez le callback `on_mount` pour des
mises a jour dynamiques :
```rust
use rs_grid_yew::{GridCanvas, wrap_model, WebGridCanvas};
use rs_grid_scene::Theme;

html! {
    <GridCanvas model={model}
        theme={Some(Theme::dark())} />
}
```


## Quand les variables sont-elles lues ?

Les variables CSS sont lues **une seule fois** au montage. Elles ne sont pas
relues a chaque frame. Pour changer le theme dynamiquement :


**[object Object]**

Utilisez un signal de theme — le composant se re-rendra automatiquement
lorsque le signal est mis a jour.


**[object Object]**

Remontez la grille apres avoir modifie les variables CSS. Appelez `detach()`
d'abord, mettez a jour les proprietes CSS, puis creez une nouvelle instance
`JsGrid`.


**[object Object]**

Utilisez un signal de theme — le composant se re-rendra automatiquement
lorsque le signal est mis a jour. Meme comportement que Leptos.


**[object Object]**

Utilisez le callback `on_mount` pour obtenir le handle de la grille,
puis appelez `gc.set_theme(theme_from_css_vars())` apres avoir modifie
les proprietes CSS.


## Themes par defaut

rs-grid est livre avec deux themes programmatiques par defaut :

| Methode          | Palette                                |
| ---------------- | -------------------------------------- |
| `Theme::light()` | Palette claire inspiree d'AG Grid      |
| `Theme::dark()`  | Palette sombre inspiree de Tokyo Night |

Voir [Themes integres](/fr/theming/built-in-themes.md) pour les details et les themes supplementaires.
