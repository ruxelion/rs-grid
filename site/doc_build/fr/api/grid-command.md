# API GridCommand

## Définition

```rust
#[non_exhaustive]
pub enum GridCommand { /* 30 variantes */ }
```

Toutes les mutations de `GridState` passent par `state.apply(GridCommand)`.

## Toutes les variantes

### Sélection

```rust
SelectCell(CellCoord)
ExtendSelection(CellCoord)
ClearSelection
MoveSelection { delta_row: i64, delta_col: i64, extend: bool }
SelectRow(u64)
ExtendRowSelection(u64)
SelectCol(usize)
ExtendColSelection(usize)
```

### Défilement et viewport

```rust
ScrollTo { x: f64, y: f64 }
ScrollBy { dx: f64, dy: f64 }
Resize { width: f64, height: f64 }
```

### Colonnes

```rust
ResizeColumn { col_idx: usize, new_width: f64 }
AutoFitColumn { col_idx: usize, char_width: f64, header_char_width: f64, cell_padding: f64 }
MoveColumn { from_idx: usize, to_idx: usize }
SetPinnedColumnCount { count: usize }
```

### Tri et filtrage

```rust
ToggleSort { col_key: String }
SetColumnFilter { col_key: String, text: String }
ClearAllFilters
```

### Édition

```rust
StartEdit { row: u64, col_key: String }
CommitEdit { row: u64, col_key: String, value: String }
CancelEdit
```

### Presse-papiers

```rust
CopySelection
CutSelection
PasteAt { text: String }
```

### Recherche

```rust
Search { query: String }
SearchNext
SearchPrev
ClearSearch
```

### Annuler / Rétablir

```rust
Undo
Redo
```

### Données et affichage

```rust
SetHoveredRow(Option<u64>)
NotifyPageLoaded
SetTotalRowCount(u64)
```

### Toggles de comportement

```rust
SetEditable(bool)          // édition inline globale on/off
SetSelectable(bool)        // sélection globale on/off (vide la sélection si false)
SetColumnReorderable(bool) // drag-to-reorder des en-têtes on/off (MoveColumn intact)
```

## CommandOutput

```rust
#[non_exhaustive]
pub enum CommandOutput {
    None,
    CopyText(String),
    CopyError(CopyError),
}

#[non_exhaustive]
pub enum CopyError {
    NoSelection,
    TooManyRows,
}
```

| Commande          | Résultat                       |
| ----------------- | ------------------------------ |
| `CopySelection`   | `CopyText(tsv)` ou `CopyError` |
| `CutSelection`    | `CopyText(tsv)` ou `CopyError` |
| Toutes les autres | `None`                         |
