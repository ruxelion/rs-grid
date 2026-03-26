# Donnees en memoire (VecDataSource)

## Presentation

`VecDataSource` est la source de donnees la plus simple — un `Vec<RowRecord>`
stocke entierement en memoire.

## Creation

```rust
use rs_grid_core::{datasource::VecDataSource, row::RowRecord};

let rows: Vec<RowRecord> = (0..1000).map(|i| {
    let mut r = RowRecord::new(i);
    r.set("name", format!("User {i}"));
    r.set("email", format!("user{i}@example.com"));
    r
}).collect();

let data = VecDataSource::new(rows);
```

Ou utilisez le raccourci `GridModel::new()` qui encapsule automatiquement
un `Vec<RowRecord>` dans un `VecDataSource` :

```rust
let model = GridModel::new(columns, rows, 32.0, 36.0);
```

## Proprietes

| Propriete     | Valeur                                            |
| ------------- | ------------------------------------------------- |
| Memoire       | O(n) — toutes les lignes en memoire               |
| Editable      | Oui — `set_cell()` modifie le vecteur sous-jacent |
| Clonable      | Oui — copie profonde de toutes les lignes         |
| `cell_status` | Toujours `Ready` ou `Absent` (jamais `Loading`)   |

## Quand l'utiliser

- Jeux de donnees jusqu'a environ 100 000 lignes
- Donnees mutables (edition directe sans la couche de patches)
- Lorsque vous avez besoin de cloner la source de donnees

## Quand NE PAS l'utiliser

- Tres grands jeux de donnees (millions de lignes) — utilisez `FnDataSource` a la place
- Donnees cote serveur — utilisez `PageCacheDataSource`
