# rs-grid-render-canvas

Backend de rendu Canvas2D. Consomme un `SceneFrame` et dessine sur un
`CanvasRenderingContext2d` via wasm-bindgen.

## Modules

| Module | Rôle |
|---|---|
| `renderer` | `CanvasRenderer` : itère sur `SceneFrame` et appelle les API Canvas2D |

## Invariants critiques

- Ce crate **ne contient pas de logique métier** — il traduit des primitives en
  appels Canvas2D, c'est tout.
- Les coordonnées reçues sont en **pixels logiques**. Le renderer applique lui-même
  le `devicePixelRatio` via une transformation de contexte (`scale(dpr, dpr)`).
- Toujours sauvegarder/restaurer le contexte Canvas (`save()`/`restore()`) autour
  des opérations de clipping ou de transformation.
- Les `PolygonPrimitive` avec `corner_radius > 0` nécessitent `arcTo` sur chaque
  segment — vérifier le comportement sur les polygones non-convexes.

## Ajouter le support d'une nouvelle primitive

Ajouter un `match` dans `renderer.rs` pour la nouvelle variante de `ScenePrimitive`.
Ne pas modifier `rs-grid-scene` depuis cette crate.

## Build

```sh
# Cette crate ne se compile qu'en target WASM
wasm-pack build --target web
```
