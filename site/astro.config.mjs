import { defineConfig } from 'astro/config';

export default defineConfig({
  output: 'static',
  trailingSlash: 'never',
  vite: {
    server: {
      allowedHosts: true,
    },
  },
});
