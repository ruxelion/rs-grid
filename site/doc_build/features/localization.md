# Localization

## Overview

rs-grid ships with **15 built-in locales** and supports custom translations.
All user-visible strings in the grid chrome (context menu, search bar) are
driven by a `Locale` struct that can be swapped at any time.

The grid can also **auto-detect the browser language** and select the
matching locale automatically.

## Built-in locales

| Code | Language   | Constructor    |
| ---- | ---------- | -------------- |
| `en` | English    | `Locale::en()` |
| `fr` | Français   | `Locale::fr()` |
| `de` | Deutsch    | `Locale::de()` |
| `es` | Español    | `Locale::es()` |
| `it` | Italiano   | `Locale::it()` |
| `pt` | Português  | `Locale::pt()` |
| `nl` | Nederlands | `Locale::nl()` |
| `pl` | Polski     | `Locale::pl()` |
| `tr` | Türkçe     | `Locale::tr()` |
| `ru` | Русский    | `Locale::ru()` |
| `uk` | Українська | `Locale::uk()` |
| `ar` | العربية    | `Locale::ar()` |
| `ja` | 日本語        | `Locale::ja()` |
| `zh` | 中文         | `Locale::zh()` |
| `ko` | 한국어        | `Locale::ko()` |

## Browser language detection

`Locale::from_browser()` reads `navigator.language` and returns the best
matching built-in locale. It matches on the primary subtag only — `"fr-FR"`,
`"fr-CA"`, and `"fr"` all resolve to `Locale::fr()`. Unknown languages fall
back to English.

```rust
// Auto-detect — the grid starts in the user's language
let locale = Locale::from_browser();
```

You can also match a BCP 47 tag manually:

```rust
let locale = Locale::from_language_tag("pt-BR"); // → Locale::pt()
```

## Framework usage


**[object Object]**

The `<GridCanvas>` component accepts an optional reactive `locale` prop.
When the signal changes, the grid updates its UI strings in place without
remounting.
```rust
use rs_grid_leptos::{GridCanvas, Locale};

// Detect browser language on startup
let locale = RwSignal::new(Locale::from_browser());

view! {
    <GridCanvas
        model=model
        locale=Signal::derive(move || locale.get())
    />
}
```
To switch language at runtime:
```rust
locale.set(Locale::de());
```


**[object Object]**

```rust
use rs_grid_web::{GridCanvas, Locale};

// Mount with auto-detected locale
let gc = GridCanvas::mount(
    canvas,
    state,
    theme,
    Locale::from_browser(),
);

// Switch language later
gc.set_locale(Locale::es());
```


**[object Object]**

The `GridCanvas` component accepts an optional reactive `locale` prop.
When the signal changes, the grid updates its UI strings in place without
remounting.
```rust
use rs_grid_dioxus::{GridCanvas, ModelSlot, Locale};

let locale = use_signal(|| Locale::from_browser());

rsx! {
    GridCanvas {
        model: model_slot,
        locale: locale,
    }
}
```
To switch language at runtime:
```rust
locale.set(Locale::de());
```


**[object Object]**

The `GridCanvas` component accepts an optional `locale` prop.
Pass a `Locale` value at mount time.
```rust
use rs_grid_yew::{GridCanvas, wrap_model, Locale};

let locale = Some(Locale::from_browser());

html! {
    <GridCanvas model={model}
        locale={locale} />
}
```
To switch language at runtime, use the `on_mount` callback to get the
grid handle and call `gc.set_locale(Locale::de())`.


## Translated strings

The `Locale` struct contains all UI chrome strings:

| Field                  | English default      | Used in            |
| ---------------------- | -------------------- | ------------------ |
| `cut`                  | Cut                  | Context menu       |
| `copy`                 | Copy                 | Context menu       |
| `copy_with_headers`    | Copy with headers    | Context menu       |
| `paste`                | Paste                | Context menu       |
| `pin_column`           | Pin Column           | Column header menu |
| `unpin_column`         | Unpin Column         | Column header menu |
| `sort_ascending`       | Sort Ascending       | Column header menu |
| `sort_descending`      | Sort Descending      | Column header menu |
| `clear_sort`           | Clear Sort           | Column header menu |
| `autosize_this_column` | Autosize This Column | Column header menu |
| `autosize_all_columns` | Autosize All Columns | Column header menu |
| `shortcut_cut`         | Ctrl+X               | Context menu hint  |
| `shortcut_copy`        | Ctrl+C               | Context menu hint  |
| `shortcut_paste`       | Ctrl+V               | Context menu hint  |
| `search_placeholder`   | Find…                | Search input       |

:::note
Column headers are **not** part of the locale system. They come from
`ColumnDef.label` and should be translated by your application code.
:::

## Custom locale

You can build a fully custom locale for any language:

```rust
let locale = Locale {
    cut: "カット".into(),
    copy: "コピー".into(),
    // ... fill all fields
    ..Locale::en() // fallback for any field you omit
};
```

## Locale file format

Built-in locales are stored as flat TOML files in the `rs-grid-web` crate
under `src/locale/`. Each file contains simple `key = "value"` pairs:

```toml
# French — Français
cut                = "Couper"
copy               = "Copier"
copy_with_headers  = "Copier avec en-têtes"
paste              = "Coller"
pin_column         = "Épingler la colonne"
sort_ascending     = "Tri croissant"
search_placeholder = "Rechercher…"
```

Files are embedded at compile time via `include_str!` — no runtime I/O.

## Adding a new locale

1. Copy an existing `.toml` file (e.g. `en.toml` → `xx.toml`)
2. Translate all values
3. In `locale/mod.rs`, add:
   - A `pub fn xx()` constructor calling `parse_toml(include_str!("xx.toml"))`
   - A `"xx"` branch in `from_language_tag`

## Per-item overrides

Individual context menu labels can also be overridden per-item, independently
of the locale, using `ContextMenuItem::with_label()`:

```rust
let config = ContextMenuConfig {
    cell_items: Some(vec![
        ContextMenuItem::copy().with_label("📋 Copy"),
    ]),
    ..Default::default()
};
gc.set_context_menu(config);
```

These per-item overrides take precedence over the locale.
