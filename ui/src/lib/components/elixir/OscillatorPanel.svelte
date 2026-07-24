<script lang="ts">
	import Knob from '$lib/components/Knob.svelte';
	import PixelSelect from '$lib/components/PixelSelect.svelte';
	import {
		PHASE_DISTORTIONS,
		SPECTRAL_MORPHS,
		UNISON_STYLES,
		type OscillatorState,
		type PhaseDistortion,
		type SpectralMorph,
		type UnisonStyle
	} from '$lib/elixir/oscillator';

	let {
		state,
		onchange,
		onreset
	}: {
		state: OscillatorState;
		onchange: (patch: Partial<OscillatorState>) => void;
		onreset: () => void;
	} = $props();

	const label = (value: string) =>
		value.replaceAll('-', ' ').replace(/\b\w/g, (letter) => letter.toUpperCase());
	const options = (values: readonly string[]) => values.map((value) => ({ value, label: label(value) }));

	const spectralOptions = options(SPECTRAL_MORPHS);
	const phaseOptions = options(PHASE_DISTORTIONS);
	const unisonOptions = options(UNISON_STYLES);

	function phaseAt(t: number): number {
		const amount = state.phaseAmount;
		switch (state.phaseDistortion) {
			case 'quantize': return Math.round(t * (3 + amount * 13)) / (3 + amount * 13);
			case 'bend': return t ** (1 + amount * 3);
			case 'squeeze': return 0.5 + (t - 0.5) * (1 - amount * 0.75);
			case 'sync': return (t * (1 + amount * 4)) % 1;
			case 'pulse-width': return t < 0.5 + amount * 0.4 ? t * 0.75 : t;
			default: return t;
		}
	}

	function sampleAt(t: number): number {
		const phase = phaseAt(t);
		const fundamental = Math.sin(phase * Math.PI * 2);
		const harmonic = Math.sin(phase * Math.PI * (state.spectralMorph === 'low-pass' ? 4 : 6));
		const mode = SPECTRAL_MORPHS.indexOf(state.spectralMorph);
		const shaped = state.spectralMorph === 'passthrough'
			? fundamental
			: fundamental * (1 - state.morphAmount * 0.45)
				+ harmonic * state.morphAmount * 0.3
				+ Math.sin(phase * Math.PI * (mode + 2)) * state.morphAmount * 0.15;
		return Math.max(-1, Math.min(1, shaped));
	}

	function makePath(offset = 0): string {
		return Array.from({ length: 73 }, (_, index) => {
			const x = (index / 72) * 720;
			const t = (index / 72 + offset) % 1;
			const y = 92 - sampleAt(t) * 58;
			return `${index === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`;
		}).join(' ');
	}

	let wavePath = $derived(makePath());
	let unisonPaths = $derived(
		Array.from({ length: Math.min(4, state.unisonVoices - 1) }, (_, index) =>
			makePath(((index + 1) * state.unisonDetuneCents) / 2400)
		)
	);
</script>

<section class="oscillator" aria-labelledby="oscillator-heading">
	<header>
		<div>
			<p class="eyebrow font-code">VOICE SOURCE 01</p>
			<h2 id="oscillator-heading">Oscillator</h2>
		</div>
		<button class="reset font-ui" type="button" onclick={onreset}>Reset oscillator</button>
	</header>

	<div class="scope" aria-label="Live oscillator shape preview">
		<svg viewBox="0 0 720 184" role="img">
			<title>Local oscillator shape preview</title>
			<defs>
				<linearGradient id="elixir-wave" x1="0" x2="1">
					<stop offset="0" stop-color="var(--color-accent-cyan)" />
					<stop offset="0.55" stop-color="var(--color-accent-teal)" />
					<stop offset="1" stop-color="var(--color-accent-magenta)" />
				</linearGradient>
			</defs>
			<path class="axis" d="M0 92 H720" />
			{#each unisonPaths as path, index}
				<path class="unison" d={path} style:opacity={0.2 - index * 0.035} />
			{/each}
			<path class="wave" d={wavePath} />
		</svg>
		<div class="scope-readout font-code">
			<span>{label(state.spectralMorph)}</span>
			<span>{state.unisonVoices} voice{state.unisonVoices === 1 ? '' : 's'}</span>
			<span>{state.unisonDetuneCents.toFixed(1)} ct</span>
		</div>
	</div>

	<div class="control-grid">
		<div class="select-control">
			<label for="spectral-morph">Spectral morph</label>
			<PixelSelect
				label="Spectral morph"
				options={spectralOptions}
				value={state.spectralMorph}
				onchange={(value) => onchange({ spectralMorph: value as SpectralMorph })}
			/>
		</div>
		<Knob
			label="Morph"
			help="Blends the selected spectral transform into the source wave."
			value={state.morphAmount}
			defaultValue={0}
			size={66}
			onchange={(morphAmount) => onchange({ morphAmount })}
		/>
		<div class="select-control">
			<label for="phase-distortion">Phase distortion</label>
			<PixelSelect
				label="Phase distortion"
				options={phaseOptions}
				value={state.phaseDistortion}
				onchange={(value) => onchange({ phaseDistortion: value as PhaseDistortion })}
			/>
		</div>
		<Knob
			label="Phase"
			help="Controls the depth of the selected phase-domain transform."
			value={state.phaseAmount}
			defaultValue={0}
			size={66}
			accent="var(--color-accent-magenta)"
			onchange={(phaseAmount) => onchange({ phaseAmount })}
		/>
		<div class="select-control">
			<label for="unison-style">Unison style</label>
			<PixelSelect
				label="Unison style"
				options={unisonOptions}
				value={state.unisonStyle}
				onchange={(value) => onchange({ unisonStyle: value as UnisonStyle })}
			/>
		</div>
		<Knob
			label="Voices"
			help="Copies inside this oscillator's unison stack."
			value={state.unisonVoices}
			min={1}
			max={16}
			step={1}
			defaultValue={1}
			size={66}
			onchange={(unisonVoices) => onchange({ unisonVoices })}
		/>
		<Knob
			label="Detune"
			help="Pitch spread between unison copies, in cents."
			value={state.unisonDetuneCents}
			min={0}
			max={100}
			step={0.5}
			defaultValue={8}
			size={66}
			format={(value) => `${value.toFixed(1)} ct`}
			onchange={(unisonDetuneCents) => onchange({ unisonDetuneCents })}
		/>
	</div>
</section>

<style>
	.oscillator {
		border: 1px solid var(--color-border);
		background: linear-gradient(155deg, rgba(26, 24, 51, 0.96), rgba(15, 14, 26, 0.98));
		box-shadow: 0 20px 80px rgba(0, 0, 0, 0.34), inset 0 1px rgba(255, 255, 255, 0.025);
		min-width: 0;
	}
	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 24px;
		padding: 18px 20px 14px;
		border-bottom: 1px solid var(--color-border);
	}
	h2 { margin: 2px 0 0; font-size: var(--font-size-lg); font-weight: 600; letter-spacing: -0.03em; }
	.eyebrow { margin: 0; color: var(--color-accent-cyan); font-size: var(--font-size-xs); letter-spacing: 0.18em; }
	.reset {
		border: 1px solid var(--color-border);
		background: var(--color-widget-bg);
		color: var(--color-text-secondary);
		padding: 7px 10px;
		font-size: var(--font-size-xs);
		cursor: pointer;
	}
	.reset:hover, .reset:focus-visible { border-color: var(--color-accent-cyan); color: var(--color-text-primary); outline: none; }
	.scope { position: relative; margin: 18px 20px 0; border: 1px solid #242240; background: #090916; overflow: hidden; }
	.scope::before {
		content: '';
		position: absolute;
		inset: 0;
		background-image: linear-gradient(rgba(51, 221, 255, 0.055) 1px, transparent 1px), linear-gradient(90deg, rgba(51, 221, 255, 0.055) 1px, transparent 1px);
		background-size: 36px 23px;
		pointer-events: none;
	}
	svg { display: block; width: 100%; height: 184px; }
	.axis { fill: none; stroke: var(--color-border); stroke-width: 1; stroke-dasharray: 4 8; }
	.wave, .unison { fill: none; vector-effect: non-scaling-stroke; }
	.wave { stroke: url(#elixir-wave); stroke-width: 2.5; filter: drop-shadow(0 0 7px rgba(51, 221, 255, 0.5)); }
	.unison { stroke: var(--color-accent-magenta); stroke-width: 1.2; }
	.scope-readout { display: flex; justify-content: space-between; gap: 12px; padding: 7px 10px; border-top: 1px solid #242240; color: var(--color-text-dim); font-size: var(--font-size-xs); text-transform: uppercase; }
	.control-grid { display: grid; grid-template-columns: minmax(145px, 1.3fr) 82px minmax(145px, 1.3fr) 82px; gap: 20px 14px; align-items: end; padding: 20px; }
	.select-control { display: grid; gap: 8px; align-self: center; }
	.select-control label { color: var(--color-text-secondary); font: 500 var(--font-size-xs) var(--font-ui); letter-spacing: 0.08em; text-transform: uppercase; }
	@media (max-width: 760px) {
		.control-grid { grid-template-columns: minmax(130px, 1fr) 76px; }
		.scope-readout span:first-child { display: none; }
	}
</style>
