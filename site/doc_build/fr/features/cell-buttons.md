# Boutons de cellule

## Vue d'ensemble

Les boutons de cellule sont des éléments cliquables dessinés sur canvas,
positionnés à droite du contenu d'une cellule.  Ils sont définis au niveau
de la **colonne** — les mêmes boutons apparaissent dans chaque ligne.  Un clic
déclenche un callback avec l'index de ligne, la clé de colonne et l'identifiant
du bouton.

```rust
use rs_grid_core::column::{ButtonDef, ButtonStyle, ColumnDef};

let mut col = ColumnDef::new("actions", "Actions", 160.0);
col.cell_buttons = vec![
    ButtonDef::new("edit",   "Modifier", ButtonStyle::Primary),
    ButtonDef::new("delete", "Supprimer", ButtonStyle::Danger),
];
```

## ButtonDef

```rust
pub struct ButtonDef {
    /// Identifiant stable passé au callback de clic.
    /// Doit être unique dans une colonne.
    pub id: String,
    /// Label affiché sur le bouton.
    pub label: String,
    /// Variante de style visuel.
    pub style: ButtonStyle,
}
```

Utiliser `ButtonDef::new(id, label, style)` :

```rust
ButtonDef::new("approve", "Approuver", ButtonStyle::Primary)
```

## ButtonStyle

| Variante    | Apparence                                  |
| ----------- | ------------------------------------------ |
| `Primary`   | Fond bleu uni, texte blanc                 |
| `Secondary` | Fond gris neutre, texte sombre             |
| `Danger`    | Fond rouge uni, texte blanc                |
| `Ghost`     | Fond transparent, bordure et texte colorés |

```rust
pub enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
    Ghost,
}
```

## Disposition

Les boutons sont disposés de **droite à gauche** : le premier élément de
`cell_buttons` est le bouton le plus à droite.  Cela garantit que les
positions restent stables lorsque d'autres boutons sont ajoutés.

La largeur de chaque bouton est estimée à partir du nombre de caractères du
label — aucune mesure Canvas2D n'est effectuée dans la couche scene.  Une
largeur de colonne d'au moins
`(nb_chars × font_size × 0.65 + padding_x × 2) × nb_boutons + gaps + marge`
est recommandée.

## Méthode builder

```rust
let col = ColumnDef::new("actions", "Actions", 160.0)
    .with_cell_buttons(vec![
        ButtonDef::new("edit",   "Modifier",  ButtonStyle::Primary),
        ButtonDef::new("delete", "Supprimer", ButtonStyle::Danger),
    ]);
```

## Callback de clic

### Couche web (`WebGridCanvas`)

```rust
gc.set_on_cell_button_click(|row, col_key, button_id| {
    web_sys::console::log_1(
        &format!("row={row} col={col_key} btn={button_id}").into(),
    );
});
```

### Composant Leptos

```rust
use rs_grid_leptos::{CellButtonClickCb, GridCanvas};

let on_click: CellButtonClickCb = Box::new(|row, col, btn| {
    match btn.as_str() {
        "edit"   => { /* ouvrir la modale d'édition */ }
        "delete" => { /* demander confirmation */ }
        _ => {}
    }
});

view! {
    <GridCanvas
        model=model
        on_cell_button_click=on_click
    />
}
```

La signature du callback est `(row: u64, col_key: String, button_id: String)`.

## Combinaison avec le contenu de la cellule

Les boutons sont rendus **par-dessus** le texte de la cellule.  Pour éviter
tout chevauchement, élargir la colonne ou supprimer le texte via un
`CellFormat::Boolean` avec des labels vides :

```rust
// Afficher uniquement les boutons — pas de texte dans la cellule
col.format = Some(CellFormat::Boolean {
    true_label:  String::new(),
    false_label: String::new(),
});
col.cell_buttons = vec![
    ButtonDef::new("view", "Voir", ButtonStyle::Ghost),
];
```

## Variables de thème

Toutes les couleurs et dimensions des boutons sont configurables via des
propriétés CSS personnalisées :

| Variable CSS                        | Champ Theme               | Défaut (light)          |
| ----------------------------------- | ------------------------- | ----------------------- |
| `--rs-grid-cell-btn-primary-bg`     | `cell_btn_primary_bg`     | `#2196f3`               |
| `--rs-grid-cell-btn-primary-text`   | `cell_btn_primary_text`   | `#ffffff`               |
| `--rs-grid-cell-btn-secondary-bg`   | `cell_btn_secondary_bg`   | `#e2e8f0`               |
| `--rs-grid-cell-btn-secondary-text` | `cell_btn_secondary_text` | `#181d1f`               |
| `--rs-grid-cell-btn-danger-bg`      | `cell_btn_danger_bg`      | `#ef4444`               |
| `--rs-grid-cell-btn-danger-text`    | `cell_btn_danger_text`    | `#ffffff`               |
| `--rs-grid-cell-btn-ghost-color`    | `cell_btn_ghost_color`    | `rgba(33,150,243,0.78)` |
| `--rs-grid-cell-btn-radius`         | `cell_btn_radius`         | `4px`                   |
| `--rs-grid-cell-btn-padding-y`      | `cell_btn_padding_y`      | `4px`                   |
| `--rs-grid-cell-btn-padding-x`      | `cell_btn_padding_x`      | `8px`                   |
| `--rs-grid-cell-btn-gap`            | `cell_btn_gap`            | `4px`                   |
| `--rs-grid-cell-btn-margin-r`       | `cell_btn_margin_r`       | `8px`                   |

## Notes d'interaction

- Un clic sur un bouton **ne sélectionne pas la cellule** — l'événement est
  consommé avant la logique de sélection.
- Les boutons sont testés contre la liste de `ButtonZone` du dernier frame
  rendu, ils sont donc toujours synchronisés avec la disposition visible.
- L'effet hover n'est pas encore implémenté ; il est prévu pour une version
  future.
