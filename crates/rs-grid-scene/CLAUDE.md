# rs-grid-scene

Couche de scene graph. Convertit un `GridState` en une liste de primitives de
rendu indépendantes de tout backend.

## Modules

| Module | Rôle |
|---|---|
| `builder` | `SceneBuilder` : prend un `&GridState` + `Theme` et produit un `SceneFrame` |
| `frame` | `SceneFrame` : liste ordonnée de `ScenePrimitive` pour une frame |
| `primitives` | Types de primitives : `RectPrimitive`, `TextPrimitive`, `LinePrimitive`, `PolygonPrimitive` |
| `theme` | `Theme` : couleurs et tailles pour le rendu |

## Invariants critiques

- Cette crate **ne connaît pas Canvas2D, WebGL, ni aucun renderer**. Elle produit
  des données, elle ne dessine pas.
- `SceneFrame` est une valeur immutable produite à chaque frame — pas d'état
  interne mutable entre frames.
- Toujours raisonner en **coordonnées logiques** (pixels indépendants du DPR).
  C'est le renderer qui applique le `devicePixelRatio`.
- L'ordre des primitives dans `SceneFrame` définit l'ordre de dessin (back-to-front).

## Primitives disponibles

- `ScenePrimitive::Rect` — rectangle rempli, stroke optionnel, coin arrondi optionnel
- `ScenePrimitive::Text` — texte clippé, alignement gauche/droite
- `ScenePrimitive::Line` — segment de droite
- `ScenePrimitive::Polygon` — polygone convexe rempli, coin arrondi optionnel

## Ajouter une primitive

1. Ajouter la struct dans `primitives.rs`
2. Ajouter la variante dans `ScenePrimitive`
3. Implémenter le rendu dans `rs-grid-render-canvas/src/renderer.rs`
