import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import wasm from 'vite-plugin-wasm';
import { defineConfig } from 'vite';

const isolationHeaders = {
	'Cross-Origin-Opener-Policy': 'same-origin',
	'Cross-Origin-Embedder-Policy': 'require-corp'
};
const isolationPlugin = {
	name: 'contrapunk-cross-origin-isolation',
	configureServer(server: { middlewares: { use: (handler: (_req: unknown, response: { setHeader: (name: string, value: string) => void }, next: () => void) => void) => void } }) {
		server.middlewares.use((_request, response, next) => {
			for (const [name, value] of Object.entries(isolationHeaders)) response.setHeader(name, value);
			next();
		});
	}
};

export default defineConfig({
	plugins: [
		isolationPlugin,
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
	server: { headers: isolationHeaders },
	preview: { headers: isolationHeaders },
	optimizeDeps: {
		exclude: ['$lib/wasm-pkg']
	}
});
