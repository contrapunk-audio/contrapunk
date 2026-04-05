import { browser } from '$app/environment';

export const ssr = false;
export const prerender = true;

export const load = async () => {
	if (browser) {
		try {
			const { PUBLIC_POSTHOG_KEY } = await import('$env/static/public');
			if (PUBLIC_POSTHOG_KEY) {
				const posthog = (await import('posthog-js')).default;
				posthog.init(PUBLIC_POSTHOG_KEY, {
					api_host: 'https://us.i.posthog.com'
				});
			}
		} catch {
			// PostHog key not set, skip analytics
		}
	}
};
