# Extraction des donnees

rs-grid n'inclut pas d'export CSV/JSON integre. A la place, `GridModel`
expose toutes les donnees necessaires pour construire votre propre export
en quelques lignes de code.

## Surface API cle

| Champ / Methode             | Ce qu'il fournit                                                         |
| --------------------------- | ------------------------------------------------------------------------ |
| `model.columns`             | `Vec<ColumnDef>` — definitions de colonnes ordonnees                     |
| `model.data.row_count()`    | Nombre total de lignes physiques                                         |
| `model.data.get_cell(r, k)` | Valeur de cellule par index physique et cle colonne                      |
| `model.data.get_cell_ref()` | Variante zero-copy `Cow<str>` (sources en memoire)                       |
| `model.sort_order`          | Indices physiques dans l'ordre de tri actuel (vide = ordre naturel)      |
| `model.filtered_indices`    | Indices physiques passant tous les filtres actifs (vide = pas de filtre) |
| `model.patches`             | Valeurs editees `(row, col_key) → value`                                 |

## Exemple de base : exporter toutes les lignes

```rust
let row_count = model.data.row_count();

for row in 0..row_count {
    for col in &model.columns {
        // Verifier les patches d'abord, puis la source de donnees
        let value = model
            .patches
            .get(&(row, col.key.clone()))
            .cloned()
            .or_else(|| model.data.get_cell(row, &col.key))
            .unwrap_or_default();

        // Ecrire `value` dans votre sortie (CSV writer, tableau JSON, etc.)
    }
}
```

## Respecter le tri et le filtrage

Si vous voulez que l'export corresponde a ce que l'utilisateur voit dans la
grille, iterez sur `filtered_indices` (si actif) ou `sort_order` (si actif)
au lieu des indices bruts :

```rust
let indices: Vec<u64> = if !model.filtered_indices.is_empty() {
    // Les indices filtres sont deja dans l'ordre de tri
    model.filtered_indices.clone()
} else if !model.sort_order.is_empty() {
    model.sort_order.clone()
} else {
    (0..model.data.row_count()).collect()
};

for &phys in &indices {
    for col in &model.columns {
        let value = model
            .patches
            .get(&(phys, col.key.clone()))
            .cloned()
            .or_else(|| model.data.get_cell(phys, &col.key))
            .unwrap_or_default();
        // ...
    }
}
```

## En-tetes de colonnes

Utilisez les champs de `ColumnDef` pour les libelles :

```rust
let headers: Vec<&str> = model
    .columns
    .iter()
    .map(|c| c.label.as_str())
    .collect();
```

## Sources de donnees server-side

Pour `PageCacheDataSource` ou les sources server-side personnalisees, seules
les lignes deja chargees dans le cache local sont disponibles. Verifiez
`cell_status()` avant de lire :

```rust
use rs_grid_core::datasource::CellStatus;

match model.data.cell_status(row, &col.key) {
    CellStatus::Ready(val) => { /* utiliser val */ }
    CellStatus::Loading    => { /* page pas encore chargee */ }
    CellStatus::Absent     => { /* pas de valeur */ }
}
```

Pour un export complet de donnees server-side, recupérez toutes les pages
directement depuis votre backend plutot que de lire a travers la grille.
