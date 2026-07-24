export const SPECTRAL_MORPHS = [
	'passthrough',
	'vocode',
	'form-scale',
	'harmonic-scale',
	'inharmonic-scale',
	'smear',
	'random-amplitudes',
	'low-pass',
	'high-pass',
	'phase-disperse',
	'shepard-tone',
	'skew'
] as const;

export const PHASE_DISTORTIONS = [
	'off',
	'quantize',
	'bend',
	'squeeze',
	'sync',
	'pulse-width',
	'fm-oscillator-a',
	'fm-oscillator-b',
	'fm-sample',
	'rm-oscillator-a',
	'rm-oscillator-b',
	'rm-sample'
] as const;

export const UNISON_STYLES = [
	'centered',
	'octaves',
	'fifths',
	'power-chord',
	'harmonic-series',
	'wide',
	'narrow',
	'organ',
	'suspended',
	'cluster',
	'alternating'
] as const;

export type SpectralMorph = (typeof SPECTRAL_MORPHS)[number];
export type PhaseDistortion = (typeof PHASE_DISTORTIONS)[number];
export type UnisonStyle = (typeof UNISON_STYLES)[number];

export type OscillatorState = {
	spectralMorph: SpectralMorph;
	morphAmount: number;
	phaseDistortion: PhaseDistortion;
	phaseAmount: number;
	unisonStyle: UnisonStyle;
	unisonVoices: number;
	unisonDetuneCents: number;
};

export const DEFAULT_OSCILLATOR_STATE: Readonly<OscillatorState> = Object.freeze({
	spectralMorph: 'passthrough',
	morphAmount: 0,
	phaseDistortion: 'off',
	phaseAmount: 0,
	unisonStyle: 'centered',
	unisonVoices: 1,
	unisonDetuneCents: 8
});

export function createOscillatorState(): OscillatorState {
	return { ...DEFAULT_OSCILLATOR_STATE };
}
