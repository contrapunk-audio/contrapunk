export { matchers } from './matchers.js';

export const nodes = [
	() => import('./nodes/0'),
	() => import('./nodes/1'),
	() => import('./nodes/2'),
	() => import('./nodes/3'),
	() => import('./nodes/4'),
	() => import('./nodes/5'),
	() => import('./nodes/6'),
	() => import('./nodes/7'),
	() => import('./nodes/8'),
	() => import('./nodes/9'),
	() => import('./nodes/10'),
	() => import('./nodes/11'),
	() => import('./nodes/12')
];

export const server_loads = [];

export const dictionary = {
		"/": [3],
		"/diary": [4,[2]],
		"/diary/machine-learning": [5,[2]],
		"/diary/machine-learning/playground": [6,[2]],
		"/diary/machine-learning/round-1": [7,[2]],
		"/diary/machine-learning/round-2": [8,[2]],
		"/diary/machine-learning/round-3": [9,[2]],
		"/diary/machine-learning/round-4": [10,[2]],
		"/diary/machine-learning/round-5": [11,[2]],
		"/diary/machine-learning/the-pivot": [12,[2]]
	};

export const hooks = {
	handleError: (({ error }) => { console.error(error) }),
	
	reroute: (() => {}),
	transport: {}
};

export const decoders = Object.fromEntries(Object.entries(hooks.transport).map(([k, v]) => [k, v.decode]));
export const encoders = Object.fromEntries(Object.entries(hooks.transport).map(([k, v]) => [k, v.encode]));

export const hash = false;

export const decode = (type, value) => decoders[type](value);

export { default as root } from '../root.js';