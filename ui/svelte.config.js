import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter({
			fallback: 'index.html'
		}),
		prerender: {
			handleHttpError: 'warn',
			entries: [
				'*',
				'/diary',
				'/diary/machine-learning',
				'/diary/machine-learning/round-1',
				'/diary/machine-learning/round-2',
				'/diary/machine-learning/round-3',
				'/diary/machine-learning/round-4',
				'/diary/machine-learning/round-5',
				'/diary/machine-learning/the-pivot',
				'/diary/machine-learning/playground'
			]
		}
	}
};

export default config;
