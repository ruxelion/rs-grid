# rs-grid — Claude Code guide

## Architecture

```
GridState  ──►  SceneBuilder  ──►  SceneFrame  ──►  CanvasRenderer  ──►  <canvas>
```

| Crate | Rôle |
|---|---|
| `rs-grid-core` | Logique headless : model, viewport, selection, hit-testing. **Aucune dépendance WASM.** |
| `rs-grid-scene` | Convertit `GridState` en primitives renderer-agnostiques (`ScenePrimitive`) |
| `rs-grid-render-canvas` | Backend Canvas2D via wasm-bindgen |
| `rs-grid-web` | Intégration navigateur : events, DPR, boucle rAF, CSS theme |
| `rs-grid-leptos` | Wrapper composant Leptos CSR (`<GridCanvas>`) |
| `examples/basic-leptos` | Application de démonstration avec Trunk |

La dépendance va dans un seul sens : `leptos → web → render-canvas → scene → core`.
Ne jamais introduire de dépendance inverse.

## Commandes courantes

```sh
# Vérification rapide (tout le workspace)
cargo check --workspace

# Build natif (pour les tests unitaires de rs-grid-core)
cargo build -p rs-grid-core

# Tests unitaires
cargo test --workspace

# Formatage
cargo fmt --all

# Linting
cargo clippy --workspace -- -D warnings

# Build WASM (exemple Leptos)
cd examples/basic-leptos
trunk build

# Serveur de dev
cd examples/basic-leptos
trunk serve
```

## Conventions de code

- **Edition** : Rust 2021
- **Largeur max** : 80 caractères (rustfmt.toml)
- **Imports** : groupés par `StdExternalCrate`, granularité `Crate`
- **Commentaires** : wrappés à 80 chars, formatés dans les doc-comments
- Pas de `unwrap()` dans le code de production — utiliser `expect("raison")` ou propagation d'erreur

## Limites importantes

- **Row count** : `u64` (max ~9×10¹⁴ avec précision f64). Voir `docs/row-count-limits.md`.
- **WASM32** : adressage 32 bits, `usize` = 4 Go max. Les index de lignes sont `u64`, pas `usize`.
- **Hit-testing** : O(log n) grâce aux offsets de colonnes précompilés. Ne pas introduire de O(n) dans ce chemin.

## Modèle de données

`GridState` est la structure centrale :
- `model: GridModel` — colonnes + données
- `viewport: ViewportState` — scroll_x, scroll_y, largeur, hauteur
- `selection: SelectionState` — ancre + focus (cellule, ligne ou colonne)

Les mutations passent exclusivement par `GridState::apply(GridCommand)`.

## Thème

Le thème est lu depuis les variables CSS (`rs-grid-web::theme_from_css_vars`).
Le fichier de référence est `examples/basic-leptos/rs-grid-theme.css`.

## Ajouter un nouveau renderer

1. Créer une nouvelle crate dépendant de `rs-grid-scene`
2. Consommer `SceneFrame` et itérer sur les `ScenePrimitive`
3. Ne pas modifier `rs-grid-core` ni `rs-grid-scene`
