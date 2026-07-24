import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import wasm from 'vite-plugin-wasm';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit(),
		wasm()
	],
	build: {
		target: 'esnext'
	},
	worker: {
		format: 'es'
	},
	optimizeDeps: {
		exclude: ['$lib/wasm-pkg']
	}
});
