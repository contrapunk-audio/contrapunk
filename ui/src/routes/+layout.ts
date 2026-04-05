import { browser } from '$app/environment';

export const ssr = false;
export const prerender = true;

export const load = async () => {
	if (browser) {
		const key = (import.meta as any).env?.PUBLIC_POSTHOG_KEY;
		if (key) {
			const posthog = (await import('posthog-js')).default;
			posthog.init(key, {
				api_host: 'https://us.i.posthog.com'
			});
		}
	}
};
