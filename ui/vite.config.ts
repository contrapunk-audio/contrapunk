import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit(),
		wasm(),
		topLevelAwait()
	],
	build: {
		rollupOptions: {
			// contrapunk-wasm is built by wasm-pack and loaded at runtime;
			// it is not available as an npm package during the Vite build.
			external: ['contrapunk-wasm']
		}
	},
	ssr: {
		// Prevent SSR from trying to bundle the WASM module
		external: ['contrapunk-wasm']
	}
});
