# Édition

## Vue d'ensemble

rs-grid prend en charge l'édition de cellules en ligne. Double-cliquez sur une
cellule (ou appuyez sur Entrée sur une cellule sélectionnée) pour commencer
l'édition. La couche web superpose un élément DOM input par-dessus la cellule
du canvas.

## Cycle de vie de l'édition

```
StartEdit → l'utilisateur saisit → CommitEdit (Entrée) ou CancelEdit (Échap)
```

### Démarrer l'édition

```rust
state.apply(GridCommand::StartEdit {
    row: 5,
    col_key: "name".into(),
});
```

La grille stocke la valeur actuelle de la cellule dans un snapshot `EditCell`
afin de pouvoir la restaurer en cas d'annulation.

### Valider

```rust
state.apply(GridCommand::CommitEdit {
    row: 5,
    col_key: "name".into(),
    value: "New Value".into(),
});
```

La nouvelle valeur est écrite dans la source de données (ou dans la couche
de patches pour les sources en lecture seule). L'édition est enregistrée
dans l'historique d'annulation.

### Annuler

```rust
state.apply(GridCommand::CancelEdit);
```

Restaure la valeur d'origine. Aucune entrée d'annulation n'est créée.

## Éditeurs de cellules

Le type d'éditeur est contrôlé par `ColumnDef::editor` :

### Champ texte (par défaut)

Lorsque `editor` vaut `None` ou `Some(CellEditor::Text)`, un simple
`<input type="text">` est affiché :

```rust
col.editor = Some(CellEditor::Text);
```

### Menu déroulant

Pour les colonnes à choix fixes, utilisez `CellEditor::Select` :

```rust
col.editor = Some(CellEditor::Select {
    options: vec![
        SelectOption {
            value: "active".into(),
            label: "Active".into(),
            icon: None,
        },
        SelectOption {
            value: "inactive".into(),
            label: "Inactive".into(),
            icon: None,
        },
    ],
});
```

Chaque `SelectOption` possède :

- `value` — valeur stockée dans la cellule lors de la validation
- `label` — texte affiché dans le menu déroulant
- `icon` — URL d'icône optionnelle affichée à gauche du libellé

## Support de l'annulation

Les éditions de cellules sont automatiquement enregistrées dans l'historique
d'annulation. Appuyez sur **Ctrl+Z** pour annuler ou **Ctrl+Y** pour rétablir.
Voir [Annuler et Rétablir](/fr/features/undo-redo.md) pour plus de détails.

## Édition avec des sources de données en lecture seule

Même `FnDataSource` (qui n'a pas de `set_cell`) prend en charge l'édition —
la nouvelle valeur est stockée dans `GridModel::patches`, qui remplace la
source de données pour cette cellule.
