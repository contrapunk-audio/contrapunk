import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [svelte()],
  server: {
    host: 'localhost',
    port: 5174,
    strictPort: true,
  },
  build: {
    target: 'es2022',
  },
});
