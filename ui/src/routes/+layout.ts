import posthog from 'posthog-js';
import { browser } from '$app/environment';

export const ssr = false;
export const prerender = true;

export const load = async () => {
	if (browser) {
		posthog.init('phc_mVtN8aKej9mDnm2fdW7dnPCBCJTeG5pC5vVabyAQffXx', {
			api_host: 'https://us.i.posthog.com'
		});
	}
};
