# Theming Overview

## Pipeline

```
CSS variables (:root) → theme_from_css_vars() → Theme struct → SceneBuilder → canvas
```

1. You define `--rs-grid-*` CSS custom properties on `:root`
2. At mount time, `theme_from_css_vars()` reads them from the computed style
3. Missing variables fall back to `Theme::light()` defaults
4. The `Theme` is passed to `SceneBuilder` which uses it for all rendering

## Theme struct

The `Theme` struct holds all visual properties:

- **Colors** — background, text, borders, selection, scrollbar, hover, search
- **Typography** — font sizes, header bold
- **Spacing** — cell padding, scrollbar width and radius

See [CSS Variables Reference](/theming/css-variables.md) for the complete list.

## Two ways to set the theme

### 1. CSS variables (recommended)

Add variables to your stylesheet:

```css
:root {
  --rs-grid-bg: #1a1b26;
  --rs-grid-cell-text: #c0caf5;
}
```

### 2. Programmatic Theme struct

Create a `Theme` directly in Rust:

```rust
let theme = Theme {
    bg: Color::rgb(26, 27, 38),
    cell_text: Color::rgb(192, 202, 245),
    ..Theme::dark()
};
```

To apply the theme dynamically:


**[object Object]**

Pass the theme as a signal for reactive updates:
```rust
let (theme, set_theme) = create_signal(Theme::dark());
// Update theme dynamically
set_theme.set(Theme::light());
```


**[object Object]**

Re-mount the grid with the new theme:
```rust
let theme = theme_from_css_vars(&canvas);
let gc = GridCanvas::mount(canvas, state, theme, locale);
```


**[object Object]**

Pass the theme as a signal for reactive updates:
```rust
let mut theme = use_signal(|| Theme::dark());
// Update theme dynamically
theme.set(Theme::light());
```


**[object Object]**

Pass the theme as a prop, or use the `on_mount` callback for dynamic updates:
```rust
use rs_grid_yew::{GridCanvas, wrap_model, WebGridCanvas};
use rs_grid_scene::Theme;

html! {
    <GridCanvas model={model}
        theme={Some(Theme::dark())} />
}
```


## When are variables read?

CSS variables are read **once** at mount time. They are not re-read on
every frame. To change the theme dynamically:


**[object Object]**

Use a theme signal — the component will re-render automatically when the
signal updates.


**[object Object]**

Re-mount the grid after changing CSS variables. Call `detach()` first,
update the CSS properties, then create a new `JsGrid` instance.


**[object Object]**

Use a theme signal — the component will re-render automatically when the
signal updates. Same behaviour as Leptos.


**[object Object]**

Use the `on_mount` callback to get the grid handle, then call
`gc.set_theme(theme_from_css_vars())` after updating CSS properties.


## Default themes

rs-grid ships with two programmatic defaults:

| Method           | Palette                           |
| ---------------- | --------------------------------- |
| `Theme::light()` | AG Grid-inspired light palette    |
| `Theme::dark()`  | Tokyo Night-inspired dark palette |

See [Built-in Themes](/theming/built-in-themes.md) for details and additional themes.
