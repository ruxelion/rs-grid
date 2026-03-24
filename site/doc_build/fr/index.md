Open Source · Rust · WebAssembly# Le moteur de data grid
conçu pour la performance

Rendu virtualisé sur Canvas2D, compilé en WebAssembly depuis Rust. Gère des millions de lignes avec du hit-testing O(log n) et un scroll à 60 fps.

[Commencer](/fr/getting-started)[Voir sur GitHub](https://github.com/bpodwinski/rs-grid)10M+lignes virtualisées60fpsrendu canvasO(log n)hit-testing5crates cibléesPourquoi rs-grid## Conçu pour les contraintes réelles

La plupart des grilles peinent au-delà de 100k lignes. rs-grid est pensé dès le départ pour la virtualisation, la performance et la maintenabilité.

### Viewport virtualisé

Seules les cellules visibles sont rendues. La consommation mémoire reste constante quel que soit le nombre de lignes.

### Cœur Rust sans copie

rs-grid-core n'a aucune dépendance WASM. Logique Rust pure, testable nativement avec cargo test.

### Renderer-agnostique

Les primitives de scène sont découplées du rendu. Échangez Canvas2D contre WebGL ou tout autre backend sans toucher à la logique.

### Intégration Leptos

Composant <GridCanvas> prêt à l'emploi pour Leptos CSR. Thème CSS variables, props réactives, zéro boilerplate.

Architecture## Une direction, pas de surprises

Un graphe de dépendances strictement unidirectionnel garde chaque crate focalisée et testable indépendamment.

GridStatemodel · viewport · selection→SceneBuilderrs-grid-scene→SceneFrameprimitives→CanvasRendererrs-grid-render-canvas→<canvas>browser`rs-grid-core`Logique headless : model, viewport, sélection, hit-testing. Pas de WASM.

`rs-grid-scene`Convertit GridState en liste de ScenePrimitive renderer-agnostiques.

`rs-grid-render-canvas`Backend Canvas2D via wasm-bindgen. Dessine les primitives dans le DOM.

`rs-grid-web`Glue navigateur : events, DPR, boucle rAF, parsing thème CSS.

`rs-grid-leptos`Composant Leptos CSR encapsulant tout le pipeline.

Démo live## Voyez par vous-même

Ceci est une véritable instance rs-grid fonctionnant dans votre navigateur via WebAssembly. Scrollez, sélectionnez des cellules, redimensionnez les colonnes — le tout à 60 fps.

1K lignes100K lignes1M lignes## Commencez à construire

Open source, licence MIT. Contributions bienvenues.

[Lire la doc](/fr/getting-started)[GitHub ↗](https://github.com/bpodwinski/rs-grid)