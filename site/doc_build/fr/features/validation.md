# Validation

## Vue d'ensemble

rs-grid prend en charge les validateurs par colonne. Un validateur est appelé
**avant** qu'une édition de cellule soit validée. S'il retourne une erreur,
l'édition est annulée et un callback est déclenché — la couche de données n'est
jamais modifiée.

```
StartEdit → l'utilisateur saisit → CommitEdit
  → validator(&value)
      Ok(())  → patch écrit → on_change()
      Err(msg) → CancelEdit  → on_validation_error(row, col, msg)
```

## Ajouter un validateur

Définissez `ColumnDef::validator` avec un `CellValidator` :

```rust
use rs_grid_core::column::{CellValidator, ColumnDef};

let mut col = ColumnDef::new("salary", "Salaire", 120.0);
col.validator = Some(CellValidator::new(|v| {
    v.parse::<f64>()
        .ok()
        .filter(|&n| n >= 0.0)
        .map(|_| ())
        .ok_or_else(|| "Doit être un nombre positif".to_string())
}));
```

La closure reçoit la valeur brute de la cellule et doit retourner :

- `Ok(())` — accepter la valeur
- `Err(String)` — rejeter avec un message d'erreur

## Écouter les erreurs de validation


**[object Object]**

Passez `on_validation_error` comme prop à `<GridCanvas>` :
```rust
let set_error = /* RwSignal<String> */;

view! {
    <GridCanvas
        model=model
        on_validation_error=Some(Box::new(move |_row, col, msg| {
            set_error.set(format!("[{col}] {msg}"));
        }))
    />
}
```


**[object Object]**

Enregistrez un callback sur le handle web du canvas :
```rust
gc.set_on_validation_error(|row, col_key, message| {
    web_sys::console::warn_1(
        &format!("[{col_key}] ligne {row} : {message}").into()
    );
});
```


**[object Object]**

Passez `on_validation_error` comme `EventHandler` a `GridCanvas` :
```rust
let mut error = use_signal(String::new);

rsx! {
    GridCanvas {
        model: model_slot,
        on_validation_error: move |(_, col, msg): (u64, String, String)| {
            error.set(format!("[{col}] {msg}"));
        },
    }
}
```


**[object Object]**

Passez `on_validation_error` comme `Option<ValidationErrorCb>` a `GridCanvas` :
```rust
use std::rc::Rc;
use rs_grid_yew::{GridCanvas, wrap_model, ValidationErrorCb};

let error_cb: ValidationErrorCb = Rc::new(|_row, col, msg| {
    web_sys::console::warn_1(
        &format!("[{col}] {msg}").into()
    );
});

html! {
    <GridCanvas model={model}
        on_validation_error={Some(error_cb)} />
}
```


Le callback est déclenché de façon synchrone après l'annulation de l'édition.
Arguments :

| Argument  | Type   | Description                                          |
| --------- | ------ | ---------------------------------------------------- |
| `row`     | `u64`  | Index de ligne logique de la cellule rejetée         |
| `col_key` | `&str` | Clé de la colonne dont le validateur s'est déclenché |
| `message` | `&str` | Chaîne d'erreur retournée par le validateur          |

## Comportement

- La validation s'exécute **uniquement sur les éditions utilisateur** (`CommitEdit`).
  Les écritures programmatiques via `GridModel::set_cell` contournent le validateur.
- En cas de rejet, `CancelEdit` est appliqué automatiquement — la cellule revient
  à sa valeur précédente. Aucune entrée d'annulation n'est créée.
- Les éditeurs `Select` sont également validés ; la valeur de l'option sélectionnée
  est passée au validateur avant la validation.
- Les validateurs sont **synchrones**. Pour une validation asynchrone (ex. vérification
  serveur), validez de façon optimiste puis revenez en arrière via
  `GridCommand::Undo` si la vérification échoue.

## Voir aussi

- [Édition](/fr/features/editing.md) — cycle de vie de l'édition et types d'éditeurs
- [API ColumnDef](/fr/api/column-def.md) — référence du type `CellValidator`
- [Annuler / Rétablir](/fr/features/undo-redo.md) — comportement de la pile d'annulation
