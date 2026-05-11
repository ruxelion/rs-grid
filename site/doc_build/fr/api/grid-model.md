# API GridModel

## Définition

```rust
pub struct GridModel {
    pub columns: Vec<ColumnDef>,
    pub data: Box<dyn DataSource>,
    pub row_height: f64,
    pub header_height: f64,
    pub column_offsets: ColumnOffsets,
    pub patches: HashMap<(u64, String), String>,
    pub row_number_width: f64,
    pub sort_order: Vec<u64>,
    pub pinned_count: usize,
    pub filters: HashMap<String, String>,
    pub filtered_indices: Vec<u64>,
    pub mode: DataSourceMode,
    pub scrollbar_size: f64,
    pub editable: bool,
    pub selectable: bool,
    pub column_reorderable: bool,
}
```

## Champs

| Champ                | Type                             | Défaut       | Description                                                      |
| -------------------- | -------------------------------- | ------------ | ---------------------------------------------------------------- |
| `columns`            | `Vec<ColumnDef>`                 | —            | Définitions de colonnes ordonnées                                |
| `data`               | `Box<dyn DataSource>`            | —            | Fournisseur de données sous-jacent                               |
| `row_height`         | `f64`                            | —            | Hauteur des lignes de données (px logiques)                      |
| `header_height`      | `f64`                            | —            | Hauteur de la ligne d'en-tête (px logiques)                      |
| `column_offsets`     | `ColumnOffsets`                  | calculé      | Offsets de bord gauche précalculés                               |
| `patches`            | `HashMap<(u64, String), String>` | vide         | Surcharges de valeurs de cellules                                |
| `row_number_width`   | `f64`                            | auto         | Largeur du gutter (auto selon le nombre de lignes)               |
| `sort_order`         | `Vec<u64>`                       | vide         | Correspondance affichage → ligne physique                        |
| `pinned_count`       | `usize`                          | `0`          | Nombre de colonnes épinglées en tête                             |
| `filters`            | `HashMap<String, String>`        | vide         | Filtres texte par colonne                                        |
| `filtered_indices`   | `Vec<u64>`                       | vide         | Indices physiques des lignes filtrées                            |
| `mode`               | `DataSourceMode`                 | `ClientSide` | Données côté client ou côté serveur                              |
| `scrollbar_size`     | `f64`                            | `14.0`       | Espace réservé à la scrollbar                                    |
| `editable`           | `bool`                           | `true`       | Édition inline globale (le drapeau par colonne s'applique aussi) |
| `selectable`         | `bool`                           | `true`       | Sélection globale ; vide la sélection si passée à `false`        |
| `column_reorderable` | `bool`                           | `true`       | Drag-to-reorder des en-têtes. `MoveColumn` fonctionne toujours   |

## Constructeurs

```rust
// Données en mémoire
pub fn new(columns: Vec<ColumnDef>, rows: Vec<RowRecord>, row_height: f64, header_height: f64) -> Self

// Source de données personnalisée
pub fn with_data_source(columns: Vec<ColumnDef>, data: Box<dyn DataSource>, row_height: f64, header_height: f64) -> Self
```

## Méthodes

| Méthode                    | Signature                                     | Description                                                |
| -------------------------- | --------------------------------------------- | ---------------------------------------------------------- |
| `get_cell`                 | `(row: u64, col_key: &str) -> Option<String>` | Lire une cellule (patches d'abord, puis source de données) |
| `set_cell`                 | `(row: u64, col_key: &str, value: String)`    | Écrire une cellule dans la source de données               |
| `logical_to_physical`      | `(logical: u64) -> u64`                       | Correspondance ligne affichée → ligne de données           |
| `display_row_count`        | `() -> u64`                                   | Nombre de lignes visibles (respecte les filtres)           |
| `total_width`              | `() -> f64`                                   | Somme des largeurs de colonnes                             |
| `total_height`             | `() -> f64`                                   | `row_count × row_height + header_height`                   |
| `pinned_width`             | `() -> f64`                                   | Largeur totale des colonnes épinglées                      |
| `compute_row_number_width` | `(row_count: u64) -> f64`                     | Calcul auto de la largeur du gutter                        |

## GridModelBuilder

Un builder ergonomique pour construire un `GridModel` avec des valeurs par defaut sensees :

```rust
pub struct GridModelBuilder { /* ... */ }

impl GridModelBuilder {
    pub fn new(
        columns: Vec<ColumnDef>,
        data: Box<dyn DataSource>,
    ) -> Self;

    pub fn row_height(self, h: f64) -> Self;       // defaut 30.0
    pub fn header_height(self, h: f64) -> Self;     // defaut 40.0
    pub fn pinned_count(self, n: usize) -> Self;    // defaut 0
    pub fn mode(self, m: DataSourceMode) -> Self;   // defaut ClientSide
    pub fn scrollbar_size(self, s: f64) -> Self;    // defaut 14.0
    pub fn selectable(self, v: bool) -> Self;       // defaut true
    pub fn column_reorderable(self, v: bool) -> Self; // defaut true
    pub fn build(self) -> GridModel;
}
```

### Exemple

```rust
let model = GridModelBuilder::new(columns, Box::new(data))
    .row_height(40.0)
    .pinned_count(2)
    .build();
```

`pinned_count` est ramene au nombre de colonnes si la valeur depasse `columns.len()`.

Les constructeurs existants `GridModel::new()` et `GridModel::with_data_source()` restent disponibles.

## DataSourceMode

```rust
pub enum DataSourceMode {
    ClientSide,  // tri/filtre effectué localement (par défaut)
    ServerSide,  // tri/filtre délégué au serveur
}
```
