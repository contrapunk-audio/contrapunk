import posthog from 'posthog-js';
import { browser } from '$app/environment';
import { PUBLIC_POSTHOG_KEY } from '$env/static/public';

export const ssr = false;
export const prerender = true;

export const load = async () => {
	if (browser && PUBLIC_POSTHOG_KEY) {
		posthog.init(PUBLIC_POSTHOG_KEY, {
			api_host: 'https://us.i.posthog.com'
		});
	}
};
