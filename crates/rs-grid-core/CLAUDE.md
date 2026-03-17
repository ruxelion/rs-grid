# rs-grid-core

Crate headless de logique de grille. **Zéro dépendance WASM, zéro dépendance web.**
Elle doit rester testable avec `cargo test` natif standard.

## Modules

| Module | Rôle |
|---|---|
| `model` | `GridModel` : colonnes + source de données |
| `state` | `GridState` : structure centrale combinant model + viewport + selection |
| `viewport` | `ViewportState` : scroll_x, scroll_y, dimensions visibles, virtualisation des lignes |
| `selection` | `SelectionState` : ancre/focus, copie TSV, paste TSV |
| `hit_test` | Hit-testing O(log n) sur cellules, headers de lignes et de colonnes |
| `commands` | `GridCommand` (enum) + `CommandOutput` — toutes les mutations passent par là |
| `datasource` | Trait `DataSource` pour l'abstraction des données |
| `column` | Définition des colonnes (`ColumnDef`) |
| `row` | Métadonnées de ligne |
| `scrollbar` | État des scrollbars (géométrie, dragging) |

## Invariants critiques

- **Pas de `wasm-bindgen`** ici. Si tu as besoin de WASM, ça appartient à `rs-grid-web`.
- Les index de lignes sont **`u64`** (pas `usize`) pour supporter >4 Go de lignes sur WASM32.
- Les mutations de `GridState` passent **uniquement** par `GridState::apply(GridCommand)`.
- Le hit-testing doit rester O(log n) — les offsets de colonnes sont précompilés.

## Commandes utiles

```sh
cargo test -p rs-grid-core
cargo clippy -p rs-grid-core -- -D warnings
```
