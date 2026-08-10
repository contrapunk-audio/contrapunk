/** @typedef {{ kind: 'synth' } | { kind: 'off' } | { kind: 'midi_port', port: number }} VoiceTarget */
/** @typedef {{ index: number, name: string }} MidiOutput */
/** @typedef {{ kind: 'off' } | { kind: 'midi_port', deviceName: string }} StoredTarget */

/** @param {string} route */
function isRoute(route) {
	return route === 'input' || route === 'pattern_low' || route === 'pattern_counter' ||
		/^(harmony|canon|counterpoint):[0-7]$/.test(route);
}

/**
 * @param {Record<string, VoiceTarget | undefined>} targets
 * @param {MidiOutput[]} outputs
 * @param {Record<string, StoredTarget | undefined>} [unavailable]
 * @returns {Record<string, StoredTarget>}
 */
export function buildStoredVoiceRoutes(targets, outputs, unavailable = {}) {
	/** @type {Record<string, StoredTarget>} */
	const routes = {};
	for (const [route, target] of Object.entries(unavailable)) {
		if (target) routes[route] = target;
	}
	for (const [route, target] of Object.entries(targets)) {
		if (!isRoute(route) || !target || target.kind === 'synth') continue;
		if (target.kind === 'off') {
			routes[route] = target;
			continue;
		}
		const device = outputs.find((output) => output.index === target.port);
		if (device) routes[route] = { kind: 'midi_port', deviceName: device.name };
	}
	return routes;
}
