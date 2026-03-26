# Lignes

## RowRecord

Chaque ligne est un `RowRecord` — un wrapper léger autour d'un `HashMap<String, String>`
avec un identifiant numérique :

```rust
pub struct RowRecord {
    pub id: u64,
    fields: HashMap<String, String>,
}

let mut row = RowRecord::new(0);
row.set("name", "Alice");
row.set("age", "30");
```

## Hauteur de ligne

Toutes les lignes partagent la même hauteur (`model.row_height`), définie à la création :

```rust
let model = GridModel::new(columns, rows, 32.0, 36.0);
//                                        ^^^^
//                                     row_height
```

Une hauteur de ligne uniforme permet un accès O(1) : `row_index = floor((y - header_height) / row_height)`.

## Gouttière de numérotation

Une colonne fixe à gauche affiche les numéros de ligne (base 1). La largeur de
la gouttière s'ajuste automatiquement selon le nombre de chiffres :

```
width = max(digits × 9px + 24px, 40px)
```

| Nombre de lignes | Chiffres | Largeur        |
| ---------------- | -------- | -------------- |
| 1–9              | 1        | 40px (minimum) |
| 10–99            | 2        | 42px           |
| 1 000–9 999      | 4        | 60px           |
| 1 000 000+       | 7+       | 87px+          |

## Survol de ligne

`SetHoveredRow` met en surbrillance la ligne sous le curseur :

```rust
state.apply(GridCommand::SetHoveredRow(Some(42)));
state.apply(GridCommand::SetHoveredRow(None)); // la souris a quitté la grille
```

La couleur de survol est contrôlée par `--rs-grid-row-hover-bg` (par défaut : surcouche semi-transparente).

## Fonds de lignes alternés

Les lignes impaires utilisent la couleur de thème `row_alt_bg` pour un effet
de zébrure subtil. Personnalisable via `--rs-grid-row-alt-bg`.

## Limites du nombre de lignes

Les indices de ligne sont des `u64` (et non des `usize`), permettant jusqu'à \~9×10^14 lignes
avec une précision f64 complète. En WASM32, `usize` est limité à 32 bits (4 Go),
c'est pourquoi les indices de ligne utilisent `u64` partout.

:::warning
Au-delà de \~9×10^14 lignes, la précision f64 se dégrade (les entiers consécutifs
deviennent indistinguables). Le code de hit-testing utilise une décomposition
préservant la précision pour atténuer ce problème aux positions de défilement extrêmes.
:::
