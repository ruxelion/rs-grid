# Themes integres

## Themes disponibles

### Light (par defaut)

Palette claire inspiree d'AG Grid avec un arriere-plan blanc et des accents gris subtils.

```rust
let theme = Theme::light();
```

| Propriete        | Valeur                     |
| ---------------- | -------------------------- |
| Arriere-plan     | `#ffffff`                  |
| En-tete bg       | `rgb(248, 249, 250)`       |
| Texte            | `rgb(24, 29, 31)`          |
| Lignes de grille | `rgb(224, 224, 224)`       |
| Selection        | `rgba(31, 119, 220, 0.18)` |
| Taille de police | 14px                       |

### Dark

Palette sombre inspiree de Tokyo Night avec des tons bleu profond.

```rust
let theme = Theme::dark();
```

| Propriete        | Valeur                     |
| ---------------- | -------------------------- |
| Arriere-plan     | `rgb(26, 27, 38)`          |
| En-tete bg       | `rgb(36, 40, 59)`          |
| Texte            | `rgb(192, 202, 245)`       |
| Lignes de grille | `rgb(42, 47, 69)`          |
| Selection        | `rgba(122, 162, 255, 0.2)` |
| Taille de police | 14px                       |

### Themes Material 3

Les exemples incluent les themes Material 3 Light et Material 3 Dark via des
fichiers CSS. Ceux-ci utilisent le systeme de couleurs Material Design 3.

## Changer de theme

### Via des classes CSS

L'application d'exemple change de theme en basculant une classe sur `<html>` :

```css
/* Default: light */
:root {
  --rs-grid-bg: #ffffff;
  --rs-grid-cell-text: rgb(24, 29, 31);
}

/* Dark override */
html.dark {
  --rs-grid-bg: rgb(26, 27, 38);
  --rs-grid-cell-text: rgb(192, 202, 245);
}
```

### Via le code


**[object Object]**

```rust
let (theme, set_theme) = create_signal(Theme::light());

// Toggle
set_theme.update(|t| {
    *t = if *t == Theme::light() {
        Theme::dark()
    } else {
        Theme::light()
    };
});
```


**[object Object]**

```rust
// Remonter la grille avec un nouveau theme
let theme = theme_from_css_vars(&canvas);
let gc = GridCanvas::mount(canvas, state, theme, locale);
```


**[object Object]**

```rust
let mut theme = use_signal(|| Theme::light());

// Toggle
let current = theme.read().clone();
theme.set(if current == Theme::light() {
    Theme::dark()
} else {
    Theme::light()
});
```


**[object Object]**

```rust
// Utilisez le callback on_mount pour obtenir le handle de la grille
let on_mount = Callback::from(|gc: WebGridCanvas| {
    // Basculer le theme via le handle de la grille
    gc.set_theme(Theme::dark());
});

html! {
    <GridCanvas model={model}
        on_mount={Some(on_mount)} />
}
```


## Creer un theme personnalise

Partez d'un theme integre et surchargez les champs souhaites :

```rust
let custom = Theme {
    bg: Color::rgb(30, 30, 46),
    header_bg: Color::rgb(49, 50, 68),
    cell_text: Color::rgb(205, 214, 244),
    selection_fill: Color::rgba(137, 180, 250, 51),
    ..Theme::dark()  // inherit everything else
};
```

Ou definissez tout via CSS — voir la [Reference des variables CSS](/fr/theming/css-variables.md).
