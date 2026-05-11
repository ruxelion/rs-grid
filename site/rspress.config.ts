import { defineConfig } from '@rspress/core';
import path from 'path';

const sidebarEN = [
  { text: 'gettingStarted', link: '/getting-started' },
  { text: 'Installation', link: '/installation' },
  {
    text: 'coreConcepts',
    items: [
      { text: 'Architecture', link: '/concepts/architecture' },
      { text: 'GridState', link: '/concepts/grid-state' },
      { text: 'Data Model', link: '/concepts/data-model' },
      { text: 'Viewport', link: '/concepts/viewport' },
      { text: 'Selection', link: '/concepts/selection' },
    ],
  },
  {
    text: 'features',
    items: [
      { text: 'Columns', link: '/features/columns' },
      { text: 'Rows', link: '/features/rows' },
      { text: 'Cell Formatting', link: '/features/cell-formatting' },
      { text: 'Cell Buttons', link: '/features/cell-buttons' },
      { text: 'Editing', link: '/features/editing' },
      { text: 'Validation', link: '/features/validation' },
      { text: 'Callbacks & Persistence', link: '/features/callbacks' },
      { text: 'Sorting', link: '/features/sorting' },
      { text: 'Filtering', link: '/features/filtering' },
      {
        text: 'Selection & Clipboard',
        link: '/features/selection-clipboard',
      },
      { text: 'Search', link: '/features/search' },
      { text: 'Undo / Redo', link: '/features/undo-redo' },
      { text: 'Scrollbars', link: '/features/scrollbars' },
      { text: 'Keyboard', link: '/features/keyboard' },
      { text: 'Context Menu', link: '/features/context-menu' },
      { text: 'Localization', link: '/features/localization' },
      { text: 'Hit-Testing', link: '/features/hit-testing' },
    ],
  },
  {
    text: 'dataSources',
    items: [
      { text: 'Overview', link: '/data/overview' },
      { text: 'VecDataSource', link: '/data/vec-datasource' },
      { text: 'FnDataSource', link: '/data/fn-datasource' },
      { text: 'PageCache', link: '/data/page-cache' },
      { text: 'Extracting Data', link: '/data/extracting-data' },
    ],
  },
  {
    text: 'theming',
    items: [
      { text: 'Overview', link: '/theming/overview' },
      { text: 'CSS Variables', link: '/theming/css-variables' },
      { text: 'Built-in Themes', link: '/theming/built-in-themes' },
    ],
  },
  {
    text: 'integrations',
    items: [
      { text: 'Frameworks', link: '/integrations/frameworks' },
      { text: 'Custom Renderer', link: '/integrations/custom-renderer' },
      { text: 'MCP Server', link: '/integrations/mcp' },
    ],
  },
  {
    text: 'sceneGraph',
    items: [
      { text: 'Primitives', link: '/scene/primitives' },
      { text: 'SceneBuilder', link: '/scene/scene-builder' },
    ],
  },
  {
    text: 'deploymentDev',
    items: [{ text: 'Docker', link: '/deployment/docker' }],
  },
  {
    text: 'Development',
    items: [
      { text: 'Workspace', link: '/development/workspace' },
      { text: 'Testing', link: '/development/testing' },
    ],
  },
  {
    text: 'apiReference',
    items: [
      { text: 'GridState', link: '/api/grid-state' },
      { text: 'GridCommand', link: '/api/grid-command' },
      { text: 'GridModel', link: '/api/grid-model' },
      { text: 'ColumnDef', link: '/api/column-def' },
      { text: 'Theme', link: '/api/theme' },
      { text: 'ScenePrimitive', link: '/api/scene-primitive' },
    ],
  },
];

const sidebarFR = [
  { text: 'gettingStarted', link: '/fr/getting-started' },
  { text: 'Installation', link: '/fr/installation' },
  {
    text: 'coreConcepts',
    items: [
      { text: 'Architecture', link: '/fr/concepts/architecture' },
      { text: 'GridState', link: '/fr/concepts/grid-state' },
      { text: 'Modèle de données', link: '/fr/concepts/data-model' },
      { text: 'Viewport', link: '/fr/concepts/viewport' },
      { text: 'Sélection', link: '/fr/concepts/selection' },
    ],
  },
  {
    text: 'features',
    items: [
      { text: 'Colonnes', link: '/fr/features/columns' },
      { text: 'Lignes', link: '/fr/features/rows' },
      {
        text: 'Formatage des cellules',
        link: '/fr/features/cell-formatting',
      },
      { text: 'Boutons de cellule', link: '/fr/features/cell-buttons' },
      { text: 'Édition', link: '/fr/features/editing' },
      { text: 'Validation', link: '/fr/features/validation' },
      { text: 'Callbacks et persistance', link: '/fr/features/callbacks' },
      { text: 'Tri', link: '/fr/features/sorting' },
      { text: 'Filtrage', link: '/fr/features/filtering' },
      {
        text: 'Sélection & Presse-papiers',
        link: '/fr/features/selection-clipboard',
      },
      { text: 'Recherche', link: '/fr/features/search' },
      { text: 'Annuler / Rétablir', link: '/fr/features/undo-redo' },
      { text: 'Barres de défilement', link: '/fr/features/scrollbars' },
      { text: 'Clavier', link: '/fr/features/keyboard' },
      { text: 'Menu contextuel', link: '/fr/features/context-menu' },
      { text: 'Localisation', link: '/fr/features/localization' },
      { text: 'Hit-Testing', link: '/fr/features/hit-testing' },
    ],
  },
  {
    text: 'dataSources',
    items: [
      { text: 'Vue d\'ensemble', link: '/fr/data/overview' },
      { text: 'VecDataSource', link: '/fr/data/vec-datasource' },
      { text: 'FnDataSource', link: '/fr/data/fn-datasource' },
      { text: 'PageCache', link: '/fr/data/page-cache' },
      { text: 'Extraction des données', link: '/fr/data/extracting-data' },
    ],
  },
  {
    text: 'theming',
    items: [
      { text: 'Vue d\'ensemble', link: '/fr/theming/overview' },
      { text: 'Variables CSS', link: '/fr/theming/css-variables' },
      { text: 'Thèmes intégrés', link: '/fr/theming/built-in-themes' },
    ],
  },
  {
    text: 'integrations',
    items: [
      { text: 'Frameworks', link: '/fr/integrations/frameworks' },
      {
        text: 'Renderer personnalisé',
        link: '/fr/integrations/custom-renderer',
      },
      { text: 'Serveur MCP', link: '/fr/integrations/mcp' },
    ],
  },
  {
    text: 'sceneGraph',
    items: [
      { text: 'Primitives', link: '/fr/scene/primitives' },
      { text: 'SceneBuilder', link: '/fr/scene/scene-builder' },
    ],
  },
  {
    text: 'deploymentDev',
    items: [{ text: 'Docker', link: '/fr/deployment/docker' }],
  },
  {
    text: 'Développement',
    items: [
      { text: 'Workspace', link: '/fr/development/workspace' },
      { text: 'Tests', link: '/fr/development/testing' },
    ],
  },
  {
    text: 'apiReference',
    items: [
      { text: 'GridState', link: '/fr/api/grid-state' },
      { text: 'GridCommand', link: '/fr/api/grid-command' },
      { text: 'GridModel', link: '/fr/api/grid-model' },
      { text: 'ColumnDef', link: '/fr/api/column-def' },
      { text: 'Theme', link: '/fr/api/theme' },
      { text: 'ScenePrimitive', link: '/fr/api/scene-primitive' },
    ],
  },
];

export default defineConfig({
  root: 'docs',
  title: 'rs-grid',
  description: 'High-performance Rust/WASM data grid engine for the web',
  icon: '/images/favicon.svg',
  logo: {
    dark: '/images/logo-dark.svg',
    light: '/images/logo-light.svg',
  },
  logoText: 'rs-grid',
  lang: 'en',
  locales: [
    {
      lang: 'en',
      label: 'English',
      title: 'rs-grid',
      description:
        'High-performance Rust/WASM data grid engine for the web',
    },
    {
      lang: 'fr',
      label: 'Français',
      title: 'rs-grid',
      description:
        'Moteur de data grid Rust/WASM haute performance pour le web',
    },
  ],
  head: [
    [
      'link',
      {
        rel: 'preconnect',
        href: 'https://fonts.googleapis.com',
      },
    ],
    [
      'link',
      {
        rel: 'preconnect',
        href: 'https://fonts.gstatic.com',
        crossorigin: '',
      },
    ],
    [
      'link',
      {
        href: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap',
        rel: 'stylesheet',
      },
    ],
    ['script', {}, 'window.RSPRESS_THEME = "dark"'],
  ],
  themeConfig: {
    darkMode: true,
    nav: [
      { text: 'nav.docs', link: '/getting-started' },
      { text: 'nav.demo', link: '/#demo' },
    ],
    sidebar: {
      '/': sidebarEN,
      '/fr/': sidebarFR,
    },
    socialLinks: [
      {
        icon: 'github',
        mode: 'link',
        content: 'https://github.com/bpodwinski/rs-grid',
      },
    ],
    footer: {
      message: '© 2025 rs-grid. Open source under MIT license.',
    },
  },
  globalStyles: path.join(__dirname, 'theme', 'index.css'),
  llms: true,
  search: {
    codeBlocks: true,
  },
});
