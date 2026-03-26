# Renderer personnalisé

## Aperçu

L'architecture de rs-grid est **agnostique du renderer**. La couche scène
produit un `SceneFrame` — une liste plate de primitives de dessin — que
n'importe quel backend de rendu peut consommer. Le renderer Canvas2D intégré
n'est qu'une implémentation parmi d'autres.

## Architecture

```
GridState → SceneBuilder → SceneFrame → [Votre Renderer] → sortie
```

Votre renderer personnalisé ne dépend que de `rs-grid-scene`. Il n'a **pas**
besoin de `rs-grid-core`, `rs-grid-web` ni `rs-grid-render-canvas`.

## Étapes

### 1. Ajouter la dépendance

```toml
[dependencies]
rs-grid-scene = { path = "../crates/rs-grid-scene" }
```

### 2. Construire un SceneFrame

```rust
use rs_grid_scene::{SceneBuilder, Theme};

let builder = SceneBuilder::new(device_pixel_ratio, theme);
let frame = builder.build(&grid_state, None);
```

### 3. Itérer sur les primitives

```rust
use rs_grid_scene::primitives::ScenePrimitive;

for prim in frame.primitives() {
    match prim {
        ScenePrimitive::Rect(r) => {
            // Dessiner un rectangle rempli à (r.x, r.y, r.width, r.height)
            // avec la couleur r.fill, optionnellement r.stroke, r.corner_radius
        }
        ScenePrimitive::Text(t) => {
            // Dessiner t.text à (t.x, t.y) avec t.color, t.font_size
            // Appliquer t.clip si présent, respecter t.align
        }
        ScenePrimitive::Line(l) => {
            // Dessiner une ligne de (l.x1, l.y1) à (l.x2, l.y2)
            // avec l.color et l.width
        }
        ScenePrimitive::Polygon(p) => {
            // Dessiner un polygone rempli avec p.points, p.fill, p.corner_radius
        }
        ScenePrimitive::Image(img) => {
            // Charger et dessiner l'image depuis img.url à (img.x, img.y)
            // avec img.width, img.height, img.corner_radius
        }
    }
}
```

## Système de coordonnées

Toutes les coordonnées sont en **pixels logiques** (pixels CSS). Votre
renderer doit appliquer la mise à l'échelle du device pixel ratio (DPR) :

```
pixel_physique = pixel_logique × dpr
```

## Ordre de dessin

Les primitives dans `SceneFrame` sont ordonnées de **l'arrière vers l'avant**.
Dessinez-les dans l'ordre — les primitives ultérieures se superposent aux
précédentes.

## Règles

- **Ne pas modifier** `rs-grid-core` ni `rs-grid-scene`
- `SceneFrame` est **immuable** — produit à neuf à chaque frame
- Tout l'état réside dans `GridState`, pas dans votre renderer
