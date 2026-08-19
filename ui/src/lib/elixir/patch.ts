import type { SynthCombineMode, SynthRolePatch } from '$lib/adapter/types';

export const PARTIAL_COUNT = 6;

export const HARMONIC_RECIPES = {
	sine: [1, 0, 0, 0, 0, 0],
	three: [1, 0.5, 0.25, 0, 0, 0],
	odd: [1, 0, 0.45, 0, 0.25, 0],
	saw: [1, 0.5, 0.333, 0.25, 0.2, 0.167],
	dark: [1, 0.25, 0.111, 0.063, 0.04, 0.028]
} as const;

export type HarmonicRecipeName = keyof typeof HARMONIC_RECIPES | 'custom';

export const ROLE_PARAMETER = {
	harmonicAmplitudeStart: 0,
	harmonicPhaseStart: 6,
	combineMode: 12,
	secondarySemitones: 13,
	secondaryFineCents: 14,
	secondaryPhase: 15,
	secondaryLevel: 16,
	attackSecs: 17,
	decaySecs: 18,
	sustainLevel: 19,
	releaseSecs: 20,
	velocitySensitivity: 21,
	expressionSensitivity: 22,
	vibratoRateHz: 23,
	vibratoDepthCents: 24,
	modWheelDepthCents: 25,
	count: 26
} as const;

export function defaultRolePatch(): SynthRolePatch {
	return {
		harmonics: {
			amplitudes: [...HARMONIC_RECIPES.sine],
			phases: Array(PARTIAL_COUNT).fill(0)
		},
		secondary: {
			mode: 'primary_only',
			semitones: 0,
			fineCents: 0,
			phase: 0,
			level: 1
		},
		envelope: {
			attackSecs: 0.005,
			decaySecs: 0,
			sustainLevel: 1,
			releaseSecs: 0.005,
			velocitySensitivity: 1,
			expressionSensitivity: 1
		},
		vibrato: {
			rateHz: 5,
			depthCents: 0,
			modWheelDepthCents: 0
		}
	};
}

export function cloneRolePatch(patch: SynthRolePatch): SynthRolePatch {
	return {
		harmonics: {
			amplitudes: [...patch.harmonics.amplitudes],
			phases: [...patch.harmonics.phases]
		},
		secondary: { ...patch.secondary },
		envelope: { ...patch.envelope },
		vibrato: { ...patch.vibrato }
	};
}

export function rolePatchFromWire(raw: Record<string, any>): SynthRolePatch {
	return {
		harmonics: {
			amplitudes: [...(raw.harmonics?.amplitudes ?? HARMONIC_RECIPES.sine)],
			phases: [...(raw.harmonics?.phases ?? Array(PARTIAL_COUNT).fill(0))]
		},
		secondary: {
			mode: raw.secondary?.mode ?? 'primary_only',
			semitones: raw.secondary?.semitones ?? 0,
			fineCents: raw.secondary?.fine_cents ?? 0,
			phase: raw.secondary?.phase ?? 0,
			level: raw.secondary?.level ?? 1
		},
		envelope: {
			attackSecs: raw.envelope?.attack_secs ?? 0.005,
			decaySecs: raw.envelope?.decay_secs ?? 0,
			sustainLevel: raw.envelope?.sustain_level ?? 1,
			releaseSecs: raw.envelope?.release_secs ?? 0.005,
			velocitySensitivity: raw.envelope?.velocity_sensitivity ?? 1,
			expressionSensitivity: raw.envelope?.expression_sensitivity ?? 1
		},
		vibrato: {
			rateHz: raw.vibrato?.rate_hz ?? 5,
			depthCents: raw.vibrato?.depth_cents ?? 0,
			modWheelDepthCents: raw.vibrato?.mod_wheel_depth_cents ?? 0
		}
	};
}

export function rolePatchToWire(patch: SynthRolePatch) {
	return {
		harmonics: patch.harmonics,
		secondary: {
			mode: patch.secondary.mode,
			semitones: patch.secondary.semitones,
			fine_cents: patch.secondary.fineCents,
			phase: patch.secondary.phase,
			level: patch.secondary.level
		},
		envelope: {
			attack_secs: patch.envelope.attackSecs,
			decay_secs: patch.envelope.decaySecs,
			sustain_level: patch.envelope.sustainLevel,
			release_secs: patch.envelope.releaseSecs,
			velocity_sensitivity: patch.envelope.velocitySensitivity,
			expression_sensitivity: patch.envelope.expressionSensitivity
		},
		vibrato: {
			rate_hz: patch.vibrato.rateHz,
			depth_cents: patch.vibrato.depthCents,
			mod_wheel_depth_cents: patch.vibrato.modWheelDepthCents
		}
	};
}

export function recipeName(amplitudes: number[]): HarmonicRecipeName {
	for (const [name, recipe] of Object.entries(HARMONIC_RECIPES)) {
		if (recipe.every((value, index) => Math.abs(value - (amplitudes[index] ?? 0)) < 0.0005)) {
			return name as keyof typeof HARMONIC_RECIPES;
		}
	}
	return 'custom';
}

export function rolePatchParameters(patch: SynthRolePatch): number[] {
	const mode: Record<SynthCombineMode, number> = { primary_only: 0, add: 1, ring: 2 };
	return [
		...patch.harmonics.amplitudes.slice(0, PARTIAL_COUNT),
		...patch.harmonics.phases.slice(0, PARTIAL_COUNT),
		mode[patch.secondary.mode],
		patch.secondary.semitones,
		patch.secondary.fineCents,
		patch.secondary.phase,
		patch.secondary.level,
		patch.envelope.attackSecs,
		patch.envelope.decaySecs,
		patch.envelope.sustainLevel,
		patch.envelope.releaseSecs,
		patch.envelope.velocitySensitivity,
		patch.envelope.expressionSensitivity,
		patch.vibrato.rateHz,
		patch.vibrato.depthCents,
		patch.vibrato.modWheelDepthCents
	];
}

function patchWith(update: (patch: SynthRolePatch) => void): SynthRolePatch {
	const patch = defaultRolePatch();
	update(patch);
	return patch;
}

export const FACTORY_PATCHES: Array<{
	id: string;
	name: string;
	description: string;
	patches: SynthRolePatch[];
}> = [
	{
		id: 'harmonic-family',
		name: 'Harmonic family',
		description: 'The published three-harmonic tone: 1, 0.5, and 0.25.',
		patches: Array.from({ length: 4 }, () => patchWith((patch) => {
			patch.harmonics.amplitudes = [...HARMONIC_RECIPES.three];
		}))
	},
	{
		id: 'ensemble-colours',
		name: 'Ensemble colours',
		description: 'Input, Harmony, Canon, and Counterpoint use different harmonic identities.',
		patches: ['sine', 'dark', 'odd', 'three'].map((name) => patchWith((patch) => {
			patch.harmonics.amplitudes = [...HARMONIC_RECIPES[name as keyof typeof HARMONIC_RECIPES]];
		}))
	},
	{
		id: 'phase-reinforcement',
		name: 'Phase reinforcement',
		description: 'Two matched sine waves add at zero degrees.',
		patches: Array.from({ length: 4 }, () => patchWith((patch) => {
			patch.secondary.mode = 'add';
		}))
	},
	{
		id: 'phase-cancellation',
		name: 'Phase cancellation',
		description: 'Two matched sine waves cancel at 180 degrees.',
		patches: Array.from({ length: 4 }, () => patchWith((patch) => {
			patch.secondary.mode = 'add';
			patch.secondary.phase = 0.5;
		}))
	},
	{
		id: 'ring-difference',
		name: 'Ring difference',
		description: 'A second sine one octave down creates sum and difference components.',
		patches: Array.from({ length: 4 }, () => patchWith((patch) => {
			patch.secondary.mode = 'ring';
			patch.secondary.semitones = -12;
		}))
	},
	{
		id: 'passive-ring-down',
		name: 'Passive ring-down',
		description: 'Fast excitation, 1.2-second decay, and zero sustain.',
		patches: Array.from({ length: 4 }, () => patchWith((patch) => {
			patch.envelope = { ...patch.envelope, decaySecs: 1.2, sustainLevel: 0, releaseSecs: 0.012 };
		}))
	},
	{
		id: 'maintained-vibrato',
		name: 'Maintained vibrato',
		description: 'An 80-millisecond startup settles into 18-cent vibrato at 5 Hz.',
		patches: Array.from({ length: 4 }, () => patchWith((patch) => {
			patch.envelope = { ...patch.envelope, attackSecs: 0.08, releaseSecs: 0.12 };
			patch.vibrato.depthCents = 18;
		}))
	}
];
