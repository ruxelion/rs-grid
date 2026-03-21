# rs-grid — Claude Code guide

## Contexte global

Ce dépôt s'inscrit dans une roadmap globale centralisée dans le dépôt privé :
https://github.com/bpodwinski/roadmap

Si le dossier local `roadmap/` est présent dans ce dépôt, il doit être utilisé
comme source de vérité prioritaire.

Avant toute proposition structurante, consulter en priorité :

- `roadmap/AI_CONTEXT.md`
- `roadmap/docs/00-hub.md`
- `roadmap/docs/02-current-focus.md`
- `roadmap/docs/projects/rs-grid.md`

Si le dossier `roadmap/` n'est pas disponible localement, utiliser le dépôt
GitHub privé comme référence.

## Position dans la roadmap

`rs-grid` est un projet stratégique transverse, mais il n'est pas la priorité
absolue tant que `FDF` n'est pas stabilisé.

Rôle de `rs-grid` dans l'ensemble :

- moteur de data grid Rust/WASM haute performance
- base UI réutilisable pour des outils futurs
- socle potentiel pour un futur produit concurrent d'AG Grid
- brique réutilisable pour le futur `Product Data Editor`

## Règles stratégiques

- Ne pas faire dériver le projet trop tôt vers une parité complète avec AG Grid
- Prioriser d'abord un noyau technique solide
- Éviter les développements business, SaaS ou expansion JS large tant que le
  cœur produit n'est pas suffisamment mature
- Si une décision importante change le cap du projet, proposer une mise à jour
  dans le dépôt `roadmap`, notamment dans `docs/03-decisions.md`

## Priorités actuelles pour rs-grid

À privilégier :

- virtualisation viewport
- rendu fluide
- sélection
- hit-testing performant
- édition basique
- architecture renderer-agnostic
- stabilité du core

À éviter pour l'instant :

- course à la feature contre AG Grid
- dispersion sur des intégrations secondaires
- complexification prématurée de l'API
- expansion commerciale trop tôt

## Architecture

```
GridState  ──►  SceneBuilder  ──►  SceneFrame  ──►  CanvasRenderer  ──►  <canvas>
```

| Crate                   | Rôle                                                                                    |
| ----------------------- | --------------------------------------------------------------------------------------- |
| `rs-grid-core`          | Logique headless : model, viewport, selection, hit-testing. **Aucune dépendance WASM.** |
| `rs-grid-scene`         | Convertit `GridState` en primitives renderer-agnostiques (`ScenePrimitive`)             |
| `rs-grid-render-canvas` | Backend Canvas2D via wasm-bindgen                                                       |
| `rs-grid-web`           | Intégration navigateur : events, DPR, boucle rAF, CSS theme                             |
| `rs-grid-leptos`        | Wrapper composant Leptos CSR (`<GridCanvas>`)                                           |
| `examples/basic-leptos` | Application de démonstration avec Trunk                                                 |

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

## Tests end-to-end (Playwright)

Les tests visuels et fonctionnels sont dans `e2e/`.

```sh
# 1. Installer Playwright (une seule fois)
cd e2e && npm install && npx playwright install chromium

# 2. Builder l'app (requis avant chaque run)
cd examples/basic-leptos && trunk build

# 3. Lancer les tests
cd e2e && npm test

# 4. Générer / regénérer les screenshots de référence
cd e2e && npm run update-snapshots
```

**Structure des tests** (`e2e/tests/grid.spec.ts`) :

- `smoke` — page se charge, canvas visible, valeurs par défaut
- `contrôles` — dropdowns lignes/colonnes
- `interaction canvas` — clics, scroll, shift-clic (coordonnées viewport)
- `visual regression` — comparaison screenshot pixel-à-pixel (tolérance 2 %)

**Attention canvas** : la grille est rendue sur `<canvas>`, pas dans le DOM.
Les tests d'interaction utilisent des coordonnées pixel fixes. Si le layout change,
mettre à jour les coordonnées dans `grid.spec.ts`.

**Commande Claude** : `/e2e` lance `trunk build` puis `npm test` automatiquement.

## Règles de travail pour Claude

- Après toute modification de code dans `rs-grid-core`, toujours lancer `/test`
  pour vérifier que les tests passent.
- Si un test échoue, le corriger avant de continuer.

## Ajouter un nouveau renderer

1. Créer une nouvelle crate dépendant de `rs-grid-scene`
2. Consommer `SceneFrame` et itérer sur les `ScenePrimitive`
3. Ne pas modifier `rs-grid-core` ni `rs-grid-scene`
