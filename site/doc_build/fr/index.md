Open Source · Rust · WebAssembly# Le moteur de data grid
conçu pour la performance

Un data grid qui ne ralentit jamais. Parcourez des millions de lignes aussi fluidement que les cent premières — édition, tri et sélection inclus.

[Commencer](/fr/getting-started)[Voir sur GitHub](https://github.com/bpodwinski/rs-grid)10M+lignes, zéro latence<1msinteraction 60fps100% Rustcompilé en WASMMITopen sourcePourquoi rs-grid## Conçu pour les contraintes réelles

La plupart des grilles peinent au-delà de 100k lignes. rs-grid est pensé dès le départ pour la virtualisation, la performance et la maintenabilité.

### Des millions de lignes

Seules les cellules visibles sont rendues. La mémoire reste constante que vous ayez 1K ou 10M lignes.

### Édition inline

Double-cliquez pour éditer. Champs texte, menus déroulants avec icônes. Historique undo/redo complet.

### Tri & filtre

Tri par colonne avec indicateurs visuels. Filtre texte, combinable entre colonnes.

### Presse-papiers

Couper, copier, coller avec Ctrl+C/X/V. Format TSV, compatible Excel et Google Sheets.

### Gestion des colonnes

Redimensionnement par drag, auto-fit au double-clic, réorganisation par drag-and-drop, colonnes figées.

### Formats de cellule riches

Nombre, devise, pourcentage, booléen, images, combos image+texte. Formateurs custom pour un contrôle total.

### Navigation clavier

Flèches, Shift+Flèche pour sélection, Enter pour éditer, Escape pour annuler. Tous les raccourcis tableur inclus.

### Recherche plein texte

Ctrl+F pour chercher dans toutes les cellules. Résultats surlignés avec navigation suivant/précédent.

### Thème CSS

Stylez tout via des propriétés CSS. Light, dark, ou créez le vôtre. Changement à chaud.

### Données serveur

Pagination asynchrone avec cache LRU. Tri et filtre délégués à votre backend.

### Menu contextuel

Menu clic droit avec couper, copier, coller, figer colonnes. Actions et items entièrement personnalisables.

### 100% Rust & WebAssembly

Aucun runtime JavaScript. Logique cœur compilée en WASM. Compatible tout framework ou vanilla JS.

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