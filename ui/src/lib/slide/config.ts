import type {
	SlideConfig,
	SlideCurve,
	SlideOverride,
	SlideRole,
	SlideSettings,
	SlideSlot,
	SlideTravel,
	SlideTrigger
} from '$lib/adapter/types';

export const SLIDE_ROLES: SlideRole[] = ['input', 'harmony', 'canon', 'counterpoint'];

const off = (): SlideTravel => ({ kind: 'off' });
const settings = (
	travel: SlideTravel = off(),
	trigger: SlideTrigger = 'legato',
	curve: SlideCurve = 'linear'
): SlideSettings => ({ travel, trigger, curve });
const emptyOverride = (): SlideOverride => ({ travel: null, trigger: null, curve: null });
const voices = (): SlideOverride[] => Array.from({ length: 8 }, emptyOverride);

export function defaultSlideConfig(): SlideConfig {
	return {
		roles: [settings(), settings(), settings(), settings()],
		voices: [voices(), voices(), voices(), voices()]
	};
}

export function cloneSlideConfig(config: SlideConfig): SlideConfig {
	return {
		roles: config.roles.map((role) => ({
			travel: { ...role.travel },
			trigger: role.trigger,
			curve: role.curve
		})) as SlideConfig['roles'],
		voices: config.voices.map((role) =>
			role.map((voice) => ({
				travel: voice.travel ? { ...voice.travel } : null,
				trigger: voice.trigger,
				curve: voice.curve
			}))
		) as SlideConfig['voices']
	};
}

export function resolveSlide(config: SlideConfig, slot: SlideSlot): SlideSettings {
	const roleIndex = SLIDE_ROLES.indexOf(slot.role);
	const parent = config.roles[Math.max(0, roleIndex)];
	const child = config.voices[Math.max(0, roleIndex)]?.[slot.voice] ?? emptyOverride();
	return {
		travel: child.travel ?? parent.travel,
		trigger: child.trigger ?? parent.trigger,
		curve: child.curve ?? parent.curve
	};
}

export function slideRoleLabel(role: SlideRole): string {
	return {
		input: 'Your Voice',
		harmony: 'Harmonic Support',
		canon: 'Canon',
		counterpoint: 'Counterpoint'
	}[role];
}

export function harmonySlotLabel(index: number, count: number): string {
	if (count === 2) return ['Lower', 'Upper'][index] ?? `Harmony ${index + 1}`;
	if (count === 3) return ['Low', 'Middle', 'High'][index] ?? `Harmony ${index + 1}`;
	if (count === 4) return ['Bass', 'Tenor', 'Alto', 'Soprano'][index] ?? `Harmony ${index + 1}`;
	return `Harmony ${index + 1}`;
}

export interface SlidePreset {
	id: string;
	name: string;
	description: string;
	config: SlideConfig;
}

function time(milliseconds: number): SlideTravel {
	return { kind: 'time', milliseconds };
}

function preset(
	id: string,
	name: string,
	description: string,
	roleTimes: [number, number, number, number],
	curve: SlideCurve,
	trigger: SlideTrigger = 'legato'
): SlidePreset {
	const config = defaultSlideConfig();
	config.roles = roleTimes.map((milliseconds) =>
		settings(milliseconds > 0 ? time(milliseconds) : off(), trigger, curve)
	) as SlideConfig['roles'];
	return { id, name, description, config };
}

const liquid = preset(
	'liquid-satb',
	'Liquid SATB',
	'Upper voices arrive first while lower voices move with more weight.',
	[120, 220, 300, 240],
	'exponential'
);
liquid.config.voices[1][0].travel = time(520);
liquid.config.voices[1][1].travel = time(360);
liquid.config.voices[1][2].travel = time(220);
liquid.config.voices[1][3].travel = time(120);

const bloom = preset(
	'harmony-bloom',
	'Harmony Bloom',
	'The performed note stays immediate while generated voices unfold.',
	[0, 220, 420, 260],
	'inverse_exponential'
);
bloom.config.voices[1][0].travel = time(120);
bloom.config.voices[1][1].travel = time(220);
bloom.config.voices[1][2].travel = time(340);

const comet = preset(
	'canon-comet',
	'Canon Comet',
	'Dry input and harmony with long, independently trailing canon entries.',
	[0, 0, 500, 0],
	'exponential',
	'always'
);
comet.config.voices[2][0].travel = time(300);
comet.config.voices[2][1].travel = time(650);
comet.config.voices[2][2].travel = time(1_000);

export const SLIDE_PRESETS: SlidePreset[] = [
	{ id: 'off', name: 'Off', description: 'Immediate pitch movement.', config: defaultSlideConfig() },
	preset(
		'silk-ensemble',
		'Silk Ensemble',
		'Short legato movement across every role.',
		[80, 140, 200, 120],
		'exponential'
	),
	bloom,
	liquid,
	comet,
	preset(
		'crosscurrent',
		'Crosscurrent',
		'The counterpoint line sweeps behind a responsive ensemble.',
		[80, 160, 0, 420],
		'linear'
	),
	preset(
		'slow-orbit',
		'Slow Orbit',
		'Long always-on movement across the full arrangement.',
		[800, 1_100, 1_400, 1_000],
		'inverse_exponential',
		'always'
	)
];
