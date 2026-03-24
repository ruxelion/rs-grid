export const languages = {
  en: 'English',
  fr: 'Français',
} as const;

export type Lang = keyof typeof languages;

export const defaultLang: Lang = 'en';

export const translations: Record<Lang, Record<string, string>> = {
  en: {
    // Header
    'nav.home': 'Home',
    'nav.docs': 'Docs',
    'nav.getStarted': 'Get started',

    // Hero
    'hero.badge': 'Open Source · Rust · WebAssembly',
    'hero.title1': 'The data grid engine',
    'hero.title2': 'built for performance',
    'hero.sub':
      'Virtualized rendering on Canvas2D, compiled to WebAssembly from Rust. Handles millions of rows with O(log n) hit-testing and 60 fps scrolling.',
    'hero.cta': 'Get started',
    'hero.github': 'View on GitHub',
    'hero.stat.rows': 'rows virtualized',
    'hero.stat.fps': 'canvas rendering',
    'hero.stat.hit': 'hit-testing',
    'hero.stat.crates': 'focused crates',

    // Features
    'features.tag': 'Why rs-grid',
    'features.title': 'Built for the hard constraints',
    'features.sub':
      'Most grid libraries struggle past 100k rows. rs-grid is designed from the ground up for virtualization, performance, and long-term maintainability.',
    'features.viewport.title': 'Virtualized viewport',
    'features.viewport.desc':
      'Only visible cells are rendered. Memory usage stays constant regardless of row count.',
    'features.rust.title': 'Zero-copy Rust core',
    'features.rust.desc':
      'rs-grid-core has no WASM dependency. Pure Rust logic, fully testable natively with cargo test.',
    'features.renderer.title': 'Renderer-agnostic',
    'features.renderer.desc':
      'Scene primitives are decoupled from rendering. Swap Canvas2D for WebGL or any future backend without touching core logic.',
    'features.leptos.title': 'Leptos integration',
    'features.leptos.desc':
      'Drop-in <GridCanvas> component for Leptos CSR. CSS-variable theming, reactive props, zero boilerplate.',

    // Architecture
    'arch.tag': 'Architecture',
    'arch.title': 'One direction, no surprises',
    'arch.sub':
      'A strict unidirectional dependency graph keeps each crate focused and independently testable.',
    'arch.core': 'Headless logic: model, viewport, selection, hit-testing. No WASM dependency.',
    'arch.scene': 'Converts GridState to renderer-agnostic ScenePrimitive list.',
    'arch.render': 'Canvas2D backend via wasm-bindgen. Draws primitives to the DOM.',
    'arch.web': 'Browser glue: events, DPR, rAF loop, CSS theme parsing.',
    'arch.leptos': 'Leptos CSR component wrapping the full pipeline.',

    // CTA
    'cta.title': 'Start building today',
    'cta.sub': 'Open source, MIT license. Contributions welcome.',
    'cta.docs': 'Read the docs',

    // Footer
    'footer.desc': 'High-performance Rust/WASM data grid engine.',
    'footer.project': 'Project',
    'footer.documentation': 'Documentation',
    'footer.stack': 'Stack',
    'footer.copyright': '© 2025 rs-grid. Open source under MIT license.',

    // Docs sidebar
    'docs.nav.gettingStarted': 'Getting started',
    'docs.nav.overview': 'Overview',
    'docs.nav.installation': 'Installation',
    'docs.nav.leptos': 'Leptos integration',
    'docs.nav.coreConcepts': 'Core concepts',
    'docs.nav.architecture': 'Architecture',
    'docs.nav.viewport': 'Viewport',
    'docs.nav.selection': 'Selection',
    'docs.nav.deployment': 'Deployment',
    'docs.nav.docker': 'Docker',
  },

  fr: {
    // Header
    'nav.home': 'Accueil',
    'nav.docs': 'Docs',
    'nav.getStarted': 'Commencer',

    // Hero
    'hero.badge': 'Open Source · Rust · WebAssembly',
    'hero.title1': 'Le moteur de data grid',
    'hero.title2': 'conçu pour la performance',
    'hero.sub':
      'Rendu virtualisé sur Canvas2D, compilé en WebAssembly depuis Rust. Gère des millions de lignes avec du hit-testing O(log n) et un scroll à 60 fps.',
    'hero.cta': 'Commencer',
    'hero.github': 'Voir sur GitHub',
    'hero.stat.rows': 'lignes virtualisées',
    'hero.stat.fps': 'rendu canvas',
    'hero.stat.hit': 'hit-testing',
    'hero.stat.crates': 'crates ciblées',

    // Features
    'features.tag': 'Pourquoi rs-grid',
    'features.title': 'Conçu pour les contraintes réelles',
    'features.sub':
      'La plupart des grilles peinent au-delà de 100k lignes. rs-grid est pensé dès le départ pour la virtualisation, la performance et la maintenabilité.',
    'features.viewport.title': 'Viewport virtualisé',
    'features.viewport.desc':
      'Seules les cellules visibles sont rendues. La consommation mémoire reste constante quel que soit le nombre de lignes.',
    'features.rust.title': 'Cœur Rust sans copie',
    'features.rust.desc':
      "rs-grid-core n'a aucune dépendance WASM. Logique Rust pure, testable nativement avec cargo test.",
    'features.renderer.title': 'Renderer-agnostique',
    'features.renderer.desc':
      'Les primitives de scène sont découplées du rendu. Échangez Canvas2D contre WebGL ou tout autre backend sans toucher à la logique.',
    'features.leptos.title': 'Intégration Leptos',
    'features.leptos.desc':
      'Composant <GridCanvas> prêt à l\'emploi pour Leptos CSR. Thème CSS variables, props réactives, zéro boilerplate.',

    // Architecture
    'arch.tag': 'Architecture',
    'arch.title': 'Une direction, pas de surprises',
    'arch.sub':
      'Un graphe de dépendances strictement unidirectionnel garde chaque crate focalisée et testable indépendamment.',
    'arch.core': 'Logique headless : model, viewport, sélection, hit-testing. Pas de WASM.',
    'arch.scene': 'Convertit GridState en liste de ScenePrimitive renderer-agnostiques.',
    'arch.render': 'Backend Canvas2D via wasm-bindgen. Dessine les primitives dans le DOM.',
    'arch.web': 'Glue navigateur : events, DPR, boucle rAF, parsing thème CSS.',
    'arch.leptos': 'Composant Leptos CSR encapsulant tout le pipeline.',

    // CTA
    'cta.title': 'Commencez à construire',
    'cta.sub': 'Open source, licence MIT. Contributions bienvenues.',
    'cta.docs': 'Lire la doc',

    // Footer
    'footer.desc': 'Moteur de data grid Rust/WASM haute performance.',
    'footer.project': 'Projet',
    'footer.documentation': 'Documentation',
    'footer.stack': 'Stack',
    'footer.copyright': '© 2025 rs-grid. Open source sous licence MIT.',

    // Docs sidebar
    'docs.nav.gettingStarted': 'Pour commencer',
    'docs.nav.overview': 'Vue d\'ensemble',
    'docs.nav.installation': 'Installation',
    'docs.nav.leptos': 'Intégration Leptos',
    'docs.nav.coreConcepts': 'Concepts clés',
    'docs.nav.architecture': 'Architecture',
    'docs.nav.viewport': 'Viewport',
    'docs.nav.selection': 'Sélection',
    'docs.nav.deployment': 'Déploiement',
    'docs.nav.docker': 'Docker',
  },
};
