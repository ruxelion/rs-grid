# Architecture

## Pipeline

Les donnees circulent dans une seule direction a travers un pipeline fixe :

```
GridState  ──►  SceneBuilder  ──►  SceneFrame  ──►  CanvasRenderer  ──►  <canvas>
```

Chaque etape a une responsabilite unique. Rien ne circule en sens inverse.

## GridState

`GridState` est la structure centrale. Elle contient trois sous-etats :

| Sous-etat   | Type             | Responsabilite                                |
| ----------- | ---------------- | --------------------------------------------- |
| `model`     | `GridModel`      | Definitions de colonnes et donnees des lignes |
| `viewport`  | `ViewportState`  | `scroll_x`, `scroll_y`, largeur, hauteur      |
| `selection` | `SelectionState` | Ancre et focus (cellule, ligne ou colonne)    |

Toutes les mutations passent par `GridState::apply(GridCommand)`. La mutation
directe des champs n'est pas exposee. Cela rend l'etat facile a capturer et
a rejouer.

## Commandes

`GridCommand` est un enum de toutes les mutations possibles :

```rust
GridCommand::ScrollTo { x: f64, y: f64 }
GridCommand::SelectCell { row: u64, col: usize }
GridCommand::Resize { width: f64, height: f64 }
GridCommand::ClearSelection
```

Les commandes sont peu couteuses a construire et peuvent etre mises en file
d'attente. Le renderer appelle `apply` une fois par frame d'animation avec
les commandes en attente.

## SceneFrame

`SceneBuilder::build(&grid_state)` retourne un `SceneFrame` — une liste
plate de valeurs `ScenePrimitive` qui decrivent exactement ce qui doit
apparaitre a l'ecran :

- **`Rect`** — rectangle rempli (fond de cellule, surbrillance de selection)
- **`Text`** — libelle de cellule avec police, couleur et limites de decoupage
- **`Line`** — ligne de bordure de la grille

La scene est agnostique du renderer. Vous pouvez implementer un nouveau
backend (WebGL, SVG, terminal) en consommant `SceneFrame` sans toucher
au core.

## Graphe de dependances des crates

```
rs-grid-leptos
  └── rs-grid-web
       └── rs-grid-render-canvas
            └── rs-grid-scene
                 └── rs-grid-core
```

| Crate                   | Depend de     | WASM requis |
| ----------------------- | ------------- | ----------- |
| `rs-grid-core`          | —             | Non         |
| `rs-grid-scene`         | core          | Non         |
| `rs-grid-render-canvas` | scene         | Oui         |
| `rs-grid-web`           | render-canvas | Oui         |
| `rs-grid-leptos`        | web           | Oui         |

## Ajouter un renderer

1. **Creer un nouveau crate**

   Ajoutez `rs-grid-scene` comme dependance.

2. **Consommer SceneFrame**

   Appelez `SceneBuilder::build(&grid_state)` pour obtenir un `SceneFrame`,
   puis iterez sur `SceneFrame::primitives`.

3. **Emettre les appels de dessin**

   Mappez chaque `ScenePrimitive` (Rect, Text, Line) vers l'API de votre
   backend.

:::warning
Ne modifiez pas `rs-grid-core` ni `rs-grid-scene` lors de l'ajout d'un
renderer. Ces crates doivent rester libres de WASM et testables avec un
simple `cargo test`.
:::
