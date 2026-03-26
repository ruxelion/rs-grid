# API GridState

## Définition

```rust
pub struct GridState {
    pub model: GridModel,
    pub viewport: ViewportState,
    pub selection: SelectionState,
    pub hovered_row: Option<u64>,
    pub sort: Option<SortState>,
    pub edit: Option<EditCell>,
    pub search: SearchState,
    history: UndoHistory,  // privé
}
```

## Champs

| Champ         | Type                | Description                                                           |
| ------------- | ------------------- | --------------------------------------------------------------------- |
| `model`       | `GridModel`         | Colonnes, source de données, dimensionnement, état tri/filtre         |
| `viewport`    | `ViewportState`     | Position de défilement (`scroll_x`, `scroll_y`), dimensions du canvas |
| `selection`   | `SelectionState`    | Sélection ancre/focus, tampon presse-papiers                          |
| `hovered_row` | `Option<u64>`       | Ligne sous le curseur de la souris                                    |
| `sort`        | `Option<SortState>` | Colonne de tri active et direction                                    |
| `edit`        | `Option<EditCell>`  | Cellule en cours d'édition                                            |
| `search`      | `SearchState`       | Requête de recherche active, correspondances, index courant           |

## Constructeur

```rust
pub fn new(
    model: GridModel,
    viewport_width: f64,
    viewport_height: f64,
) -> Self
```

Crée un `GridState` avec le modèle fourni et les dimensions initiales du
viewport. Tous les autres champs démarrent à leurs valeurs par défaut
(pas de sélection, pas de tri, pas d'édition, etc.).

## Méthodes

### apply

```rust
pub fn apply(&mut self, cmd: GridCommand) -> CommandOutput
```

La **seule** manière de modifier `GridState`. Applique une commande et
retourne le résultat.

Voir l'[API GridCommand](/fr/api/grid-command.md) pour toutes les commandes disponibles.

## Types associés

### SortState

```rust
pub struct SortState {
    pub col_key: String,
    pub dir: SortDir,
}

pub enum SortDir { Asc, Desc }
```

### EditCell

```rust
pub struct EditCell {
    pub row: u64,
    pub col_key: String,
    pub original_value: Option<String>,
}
```

### SearchState

```rust
pub struct SearchState {
    pub query: String,
    pub matches: Vec<CellCoord>,
    pub current: usize,
}
```
