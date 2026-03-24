import { defineConfig } from '@rspress/core';
import path from 'path';

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
      description: 'High-performance Rust/WASM data grid engine for the web',
    },
    {
      lang: 'fr',
      label: 'Français',
      title: 'rs-grid',
      description: 'Moteur de data grid Rust/WASM haute performance pour le web',
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
