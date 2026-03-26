# Localisation

## Vue d'ensemble

rs-grid est livré avec **15 locales intégrées** et supporte les traductions
personnalisées. Toutes les chaînes visibles dans l'interface de la grille
(menu contextuel, barre de recherche) sont pilotées par un struct `Locale`
interchangeable à tout moment.

La grille peut aussi **détecter automatiquement la langue du navigateur** et
sélectionner la locale correspondante.

## Locales intégrées

| Code | Langue     | Constructeur   |
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

## Détection de la langue du navigateur

`Locale::from_browser()` lit `navigator.language` et retourne la locale
correspondante. Le matching se fait sur le sous-tag principal uniquement —
`"fr-FR"`, `"fr-CA"` et `"fr"` donnent tous `Locale::fr()`. Les langues
non reconnues retombent sur l'anglais.

```rust
// Détection auto — la grille démarre dans la langue de l'utilisateur
let locale = Locale::from_browser();
```

Vous pouvez aussi matcher un tag BCP 47 manuellement :

```rust
let locale = Locale::from_language_tag("pt-BR"); // → Locale::pt()
```

## Utilisation par framework


**[object Object]**

Le composant `<GridCanvas>` accepte une prop réactive optionnelle `locale`.
Quand le signal change, la grille met à jour ses textes sans se remonter.
```rust
use rs_grid_leptos::{GridCanvas, Locale};

// Détecte la langue au démarrage
let locale = RwSignal::new(Locale::from_browser());

view! {
    <GridCanvas
        model=model
        locale=Signal::derive(move || locale.get())
    />
}
```
Pour changer de langue à la volée :
```rust
locale.set(Locale::de());
```


**[object Object]**

```rust
use rs_grid_web::{GridCanvas, Locale};

// Monte avec la locale auto-détectée
let gc = GridCanvas::mount(
    canvas,
    state,
    theme,
    Locale::from_browser(),
);

// Change de langue plus tard
gc.set_locale(Locale::es());
```


**[object Object]**

Le composant `GridCanvas` accepte un prop reactif `locale` optionnel.
Lorsque le signal change, la grille met a jour ses chaines d'interface
sans remontage.
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
Pour changer de langue au runtime :
```rust
locale.set(Locale::de());
```


**[object Object]**

Le composant `GridCanvas` accepte un prop `locale` optionnel.
Passez une valeur `Locale` au montage.
```rust
use rs_grid_yew::{GridCanvas, wrap_model, Locale};

let locale = Some(Locale::from_browser());

html! {
    <GridCanvas model={model}
        locale={locale} />
}
```
Pour changer de langue au runtime, utilisez le callback `on_mount` pour
obtenir le handle de la grille et appelez `gc.set_locale(Locale::de())`.


## Chaînes traduites

Le struct `Locale` contient toutes les chaînes de l'interface :

| Champ                  | Valeur par défaut (EN) | Utilisé dans       |
| ---------------------- | ---------------------- | ------------------ |
| `cut`                  | Cut                    | Menu contextuel    |
| `copy`                 | Copy                   | Menu contextuel    |
| `copy_with_headers`    | Copy with headers      | Menu contextuel    |
| `paste`                | Paste                  | Menu contextuel    |
| `pin_column`           | Pin Column             | Menu d'en-tête     |
| `unpin_column`         | Unpin Column           | Menu d'en-tête     |
| `sort_ascending`       | Sort Ascending         | Menu d'en-tête     |
| `sort_descending`      | Sort Descending        | Menu d'en-tête     |
| `clear_sort`           | Clear Sort             | Menu d'en-tête     |
| `autosize_this_column` | Autosize This Column   | Menu d'en-tête     |
| `autosize_all_columns` | Autosize All Columns   | Menu d'en-tête     |
| `shortcut_cut`         | Ctrl+X                 | Raccourci menu     |
| `shortcut_copy`        | Ctrl+C                 | Raccourci menu     |
| `shortcut_paste`       | Ctrl+V                 | Raccourci menu     |
| `search_placeholder`   | Find…                  | Champ de recherche |

:::note
Les en-têtes de colonnes ne font **pas** partie du système de locale. Ils
proviennent de `ColumnDef.label` et doivent être traduits par votre code
applicatif.
:::

## Locale personnalisée

Vous pouvez construire une locale entièrement personnalisée :

```rust
let locale = Locale {
    cut: "カット".into(),
    copy: "コピー".into(),
    // ... remplir tous les champs
    ..Locale::en() // fallback pour les champs omis
};
```

## Format des fichiers de locale

Les locales intégrées sont stockées dans des fichiers TOML plats dans le crate
`rs-grid-web` sous `src/locale/`. Chaque fichier contient des paires
`clé = "valeur"` :

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

Les fichiers sont embarqués à la compilation via `include_str!` — pas d'I/O
au runtime.

## Ajouter une nouvelle locale

1. Copier un fichier `.toml` existant (ex : `en.toml` → `xx.toml`)
2. Traduire toutes les valeurs
3. Dans `locale/mod.rs`, ajouter :
   - Un constructeur `pub fn xx()` appelant `parse_toml(include_str!("xx.toml"))`
   - Une branche `"xx"` dans `from_language_tag`

## Surcharges par item

Les labels du menu contextuel peuvent aussi être surchargés
individuellement, indépendamment de la locale, via
`ContextMenuItem::with_label()` :

```rust
let config = ContextMenuConfig {
    cell_items: Some(vec![
        ContextMenuItem::copy().with_label("📋 Copier"),
    ]),
    ..Default::default()
};
gc.set_context_menu(config);
```

Ces surcharges par item ont priorité sur la locale.
