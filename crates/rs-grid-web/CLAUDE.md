# rs-grid-web

Intégration navigateur. Gère le cycle de vie complet d'une instance de grille
dans le DOM : events souris/clavier, boucle rAF, redimensionnement, DPR, thème CSS.

## Modules

| Module | Rôle |
|---|---|
| `canvas` | `GridCanvas` : monte la grille sur un `HtmlCanvasElement`, gère rAF et events |
| `css_theme` | `theme_from_css_vars()` : lit les variables CSS pour construire un `Theme` |

## Responsabilités de `GridCanvas`

- Redimensionnement via `ResizeObserver` (mise à jour du viewport)
- Boucle `requestAnimationFrame` : `SceneBuilder` → `SceneFrame` → `CanvasRenderer`
- Gestion des events : `mousemove`, `mousedown`, `mouseup`, `wheel`, `keydown`,
  `copy`, `paste`
- Ajustement du canvas au `devicePixelRatio` pour les écrans HiDPI
- Auto-scroll pendant le drag de sélection

## Invariants critiques

- `GridCanvas::mount()` est l'unique point d'entrée public — un canvas = une instance.
- Les events sont convertis en `GridCommand` avant d'être appliqués à `GridState`.
  **Ne pas manipuler `GridState` directement depuis les handlers d'events.**
- Le DPR est lu une seule fois au mount et à chaque resize. Ne pas le relire
  à chaque frame.
- `theme_from_css_vars()` lit le DOM — appeler uniquement au mount, pas à chaque frame.

## Thème CSS

Les variables CSS sont préfixées `--rs-grid-*`. Le fichier de référence est
`examples/basic-leptos/rs-grid-theme.css`. Pour ajouter une couleur de thème :
1. Ajouter la variable dans le CSS de l'exemple
2. Lire la variable dans `css_theme.rs`
3. Ajouter le champ dans `Theme` (`rs-grid-scene/src/theme.rs`)
