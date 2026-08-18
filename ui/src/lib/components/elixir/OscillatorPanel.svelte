<script lang="ts">
	import { synth } from '$lib/stores/synth.svelte';
	import {
		defaultRolePatch,
		HARMONIC_RECIPES,
		recipeName,
		type HarmonicRecipeName
	} from '$lib/elixir/patch';
	import type { SynthCombineMode, SynthRolePatch } from '$lib/adapter/types';

	let { role = 0, disabled = false } = $props<{ role?: number; disabled?: boolean }>();
	const previewFundamental = 220;
	let patch = $derived(synth.rolePatches[role] ?? defaultRolePatch());
	let selectedRecipe = $derived(recipeName(patch.harmonics.amplitudes));
	let secondaryFrequency = $derived(
		previewFundamental * 2 ** ((patch.secondary.semitones + patch.secondary.fineCents / 100) / 12)
	);
	let differenceFrequency = $derived(Math.abs(previewFundamental - secondaryFrequency));
	let sumFrequency = $derived(previewFundamental + secondaryFrequency);

	function update(change: (next: SynthRolePatch) => void) {
		void synth.updateRolePatch(role, change);
	}

	function setRecipe(name: HarmonicRecipeName) {
		if (name === 'custom') return;
		update((next) => {
			next.harmonics.amplitudes = [...HARMONIC_RECIPES[name]];
		});
	}

	function setHarmonic(index: number, field: 'amplitudes' | 'phases', value: number) {
		update((next) => {
			next.harmonics[field][index] = value;
		});
	}

	function setInteraction(mode: SynthCombineMode) {
		update((next) => { next.secondary.mode = mode; });
	}

	let waveformPath = $derived.by(() => {
		const amplitudes = patch.harmonics.amplitudes;
		const phases = patch.harmonics.phases;
		const energy = Math.sqrt(amplitudes.reduce((sum, amplitude) => sum + amplitude ** 2, 0)) || 1;
		const ratio = secondaryFrequency / previewFundamental;
		return Array.from({ length: 241 }, (_, index) => {
			const cycles = 2 * index / 240;
			const primary = amplitudes.reduce(
				(sum, amplitude, harmonic) => sum + amplitude * Math.sin(2 * Math.PI * ((harmonic + 1) * cycles + phases[harmonic])),
				0
			) / energy;
			const secondary = patch.secondary.level * Math.sin(2 * Math.PI * (ratio * cycles + patch.secondary.phase));
			const sample = patch.secondary.mode === 'add'
				? primary + secondary
				: patch.secondary.mode === 'ring'
					? primary * secondary
					: primary;
			const x = 18 + index / 240 * 684;
			const y = 82 - sample * 44;
			return `${index ? 'L' : 'M'}${x.toFixed(2)} ${y.toFixed(2)}`;
		}).join(' ');
	});

	type ComplexLine = { frequency: number; real: number; imaginary: number };
	let spectrum = $derived.by(() => {
		const lines = new Map<string, ComplexLine>();
		const energy = Math.sqrt(patch.harmonics.amplitudes.reduce((sum, amplitude) => sum + amplitude ** 2, 0)) || 1;
		const addLine = (frequency: number, amplitude: number, phase: number, cosine = false) => {
			if (frequency < 0) {
				frequency = -frequency;
				phase = -phase;
			}
			if (frequency >= 24_000 || amplitude === 0) return;
			const angle = phase + (cosine ? 0 : -Math.PI / 2);
			const key = frequency.toFixed(3);
			const line = lines.get(key) ?? { frequency, real: 0, imaginary: 0 };
			line.real += amplitude * Math.cos(angle);
			line.imaginary += amplitude * Math.sin(angle);
			lines.set(key, line);
		};

		patch.harmonics.amplitudes.forEach((amplitude, index) => {
			const frequency = previewFundamental * (index + 1);
			const phase = patch.harmonics.phases[index] * Math.PI * 2;
			if (patch.secondary.mode === 'ring') {
				const secondaryPhase = patch.secondary.phase * Math.PI * 2;
				const level = 0.5 * amplitude * patch.secondary.level / energy;
				addLine(frequency - secondaryFrequency, level, phase - secondaryPhase, true);
				addLine(frequency + secondaryFrequency, -level, phase + secondaryPhase, true);
			} else {
				addLine(frequency, amplitude / energy, phase);
			}
		});
		if (patch.secondary.mode === 'add') {
			addLine(secondaryFrequency, patch.secondary.level, patch.secondary.phase * Math.PI * 2);
		}
		return [...lines.values()]
			.map((line) => ({ ...line, amplitude: Math.hypot(line.real, line.imaginary) }))
			.filter((line) => line.amplitude > 0.001)
			.sort((a, b) => a.frequency - b.frequency);
	});
	let spectrumMaximum = $derived(Math.max(1, ...spectrum.map((line) => line.amplitude)));
	let spectrumFrequencyMaximum = $derived(Math.max(1500, ...spectrum.map((line) => line.frequency)));

	let envelopePath = $derived.by(() => {
		const sustainY = 238 - patch.envelope.sustainLevel * 72;
		return `M18 238 L126 166 L234 ${sustainY.toFixed(2)} L558 ${sustainY.toFixed(2)} L702 238`;
	});
</script>

<section class="oscillator" aria-labelledby="oscillator-heading">
	<header class="panel-title">
		<div><a href="https://contrapunk.com/learn/wavetable-synthesis/chapter-1/" target="_blank" rel="noopener">CHAPTER 1 ↗</a><h2 id="oscillator-heading">Harmonic colour</h2></div>
		<label>
			<span>Recipe</span>
			<select value={selectedRecipe} {disabled} onchange={(event) => setRecipe(event.currentTarget.value as HarmonicRecipeName)}>
				<option value="sine">Sine</option>
				<option value="three">Three harmonics</option>
				<option value="odd">Odd only</option>
				<option value="saw">Saw-like 1/k</option>
				<option value="dark">Dark 1/k²</option>
				{#if selectedRecipe === 'custom'}<option value="custom">Custom</option>{/if}
			</select>
		</label>
	</header>

	<div class="scope" role="group" aria-label="Current waveform and one-sided spectrum preview">
		<svg viewBox="0 0 720 270" role="img" aria-labelledby="scope-title scope-description">
			<title id="scope-title">Elixir harmonic waveform and spectrum</title>
			<desc id="scope-description">Two cycles of the selected interaction above spectral lines calculated at a 220 hertz preview fundamental.</desc>
			<path class="grid" d="M18 38 H702 M18 82 H702 M18 126 H702 M18 166 H702 M18 238 H702 M18 18 V250 M189 18 V250 M360 18 V250 M531 18 V250 M702 18 V250" />
			<text x="18" y="15">TWO CYCLES · 220 HZ PREVIEW</text>
			<path class="wave" d={waveformPath} />
			<text x="18" y="154">ONE-SIDED COMPONENTS</text>
			{#each spectrum as line}
				{@const x = 18 + Math.min(1, line.frequency / spectrumFrequencyMaximum) * 684}
				{@const top = 238 - line.amplitude / spectrumMaximum * 60}
				<line class="spectral-line" x1={x} y1="238" x2={x} y2={top} />
				<text class="frequency" x={x} y={Math.max(170, top - 5)} text-anchor="middle">{Math.round(line.frequency)}</text>
			{/each}
			<path class="envelope" d={envelopePath} />
		</svg>
		<div class="scope-legend"><span>waveform</span><span>spectrum in Hz</span><span>ADSR trajectory</span></div>
	</div>

	<fieldset class="harmonics" {disabled}>
		<legend>Relative harmonic amplitudes</legend>
		{#each patch.harmonics.amplitudes as amplitude, index}
			<label>
				<span><b>H{index + 1}</b><small>{previewFundamental * (index + 1)} Hz</small></span>
				<input aria-label={`Harmonic ${index + 1} amplitude`} type="range" min="0" max="1" step="0.01" value={amplitude} onchange={(event) => setHarmonic(index, 'amplitudes', Number(event.currentTarget.value))} />
				<output>{amplitude.toFixed(2)}</output>
			</label>
		{/each}
	</fieldset>

	<details class="phase-editor">
		<summary>Phase offsets <span>Magnitude alone does not define the waveform</span></summary>
		<div class="phase-grid">
			{#each patch.harmonics.phases as phase, index}
				<label><span>H{index + 1}</span><input aria-label={`Harmonic ${index + 1} phase`} type="range" min="0" max="0.9972222" step="0.0027778" value={phase} {disabled} onchange={(event) => setHarmonic(index, 'phases', Number(event.currentTarget.value))} /><output>{Math.round(phase * 360)}°</output></label>
			{/each}
		</div>
	</details>

	<section class="interaction" aria-labelledby="interaction-heading">
		<header><div><a href="https://contrapunk.com/learn/wavetable-synthesis/chapter-2/" target="_blank" rel="noopener">CHAPTER 2 ↗</a><h3 id="interaction-heading">Oscillator interaction</h3></div><p>Linear addition keeps existing components. Ring multiplication creates sum and difference components.</p></header>
		<div class="mode-switch" role="group" aria-label="Oscillator interaction mode">
			{#each [['primary_only', 'A only'], ['add', 'Add'], ['ring', 'Ring']] as mode}
				<button type="button" class:active={patch.secondary.mode === mode[0]} aria-pressed={patch.secondary.mode === mode[0]} {disabled} onclick={() => setInteraction(mode[0] as SynthCombineMode)}>{mode[1]}</button>
			{/each}
		</div>
		<div class="control-grid">
			<label><span>Operator B interval</span><input type="range" min="-24" max="24" step="1" value={patch.secondary.semitones} {disabled} onchange={(event) => update((next) => { next.secondary.semitones = Number(event.currentTarget.value); })} /><output>{patch.secondary.semitones > 0 ? '+' : ''}{patch.secondary.semitones.toFixed(0)} st</output></label>
			<label><span>Fine</span><input type="range" min="-100" max="100" step="1" value={patch.secondary.fineCents} {disabled} onchange={(event) => update((next) => { next.secondary.fineCents = Number(event.currentTarget.value); })} /><output>{patch.secondary.fineCents > 0 ? '+' : ''}{patch.secondary.fineCents.toFixed(0)} c</output></label>
			<label><span>Relative phase</span><input type="range" min="0" max="0.9972222" step="0.0027778" value={patch.secondary.phase} {disabled} onchange={(event) => update((next) => { next.secondary.phase = Number(event.currentTarget.value); })} /><output>{Math.round(patch.secondary.phase * 360)}°</output></label>
			<label><span>Operator B level</span><input type="range" min="0" max="1" step="0.01" value={patch.secondary.level} {disabled} onchange={(event) => update((next) => { next.secondary.level = Number(event.currentTarget.value); })} /><output>{Math.round(patch.secondary.level * 100)}%</output></label>
		</div>
		<p class="component-readout"><span>A {previewFundamental.toFixed(1)} Hz</span><span>B {secondaryFrequency.toFixed(1)} Hz</span><span>Difference {differenceFrequency.toFixed(1)} Hz</span><span>Sum {sumFrequency.toFixed(1)} Hz</span></p>
	</section>

	<section class="trajectory-grid">
		<div class="trajectory" aria-labelledby="amplitude-heading">
			<header><span>AMPLITUDE TRAJECTORY</span><h3 id="amplitude-heading">Articulation</h3></header>
			<div class="control-grid envelope-controls">
				<label><span>Attack</span><input type="range" min="0" max="5" step="0.005" value={patch.envelope.attackSecs} {disabled} onchange={(event) => update((next) => { next.envelope.attackSecs = Number(event.currentTarget.value); })} /><output>{patch.envelope.attackSecs.toFixed(3)} s</output></label>
				<label><span>Decay</span><input type="range" min="0" max="5" step="0.01" value={patch.envelope.decaySecs} {disabled} onchange={(event) => update((next) => { next.envelope.decaySecs = Number(event.currentTarget.value); })} /><output>{patch.envelope.decaySecs.toFixed(2)} s</output></label>
				<label><span>Sustain level</span><input type="range" min="0" max="1" step="0.01" value={patch.envelope.sustainLevel} {disabled} onchange={(event) => update((next) => { next.envelope.sustainLevel = Number(event.currentTarget.value); })} /><output>{Math.round(patch.envelope.sustainLevel * 100)}%</output></label>
				<label><span>Release</span><input type="range" min="0" max="10" step="0.01" value={patch.envelope.releaseSecs} {disabled} onchange={(event) => update((next) => { next.envelope.releaseSecs = Number(event.currentTarget.value); })} /><output>{patch.envelope.releaseSecs.toFixed(2)} s</output></label>
				<label><span>Velocity</span><input type="range" min="0" max="1" step="0.01" value={patch.envelope.velocitySensitivity} {disabled} onchange={(event) => update((next) => { next.envelope.velocitySensitivity = Number(event.currentTarget.value); })} /><output>{Math.round(patch.envelope.velocitySensitivity * 100)}%</output></label>
				<label><span>Expression</span><input type="range" min="0" max="1" step="0.01" value={patch.envelope.expressionSensitivity} {disabled} onchange={(event) => update((next) => { next.envelope.expressionSensitivity = Number(event.currentTarget.value); })} /><output>{Math.round(patch.envelope.expressionSensitivity * 100)}%</output></label>
			</div>
		</div>
		<div class="trajectory" aria-labelledby="pitch-heading">
			<header><span>PITCH TRAJECTORY</span><h3 id="pitch-heading">Vibrato</h3></header>
			<div class="control-grid vibrato-controls">
				<label><span>Rate</span><input type="range" min="1" max="8" step="0.1" value={patch.vibrato.rateHz} {disabled} onchange={(event) => update((next) => { next.vibrato.rateHz = Number(event.currentTarget.value); })} /><output>{patch.vibrato.rateHz.toFixed(1)} Hz</output></label>
				<label><span>Depth</span><input type="range" min="0" max="50" step="1" value={patch.vibrato.depthCents} {disabled} onchange={(event) => update((next) => { next.vibrato.depthCents = Number(event.currentTarget.value); })} /><output>{patch.vibrato.depthCents.toFixed(0)} c</output></label>
				<label><span>Mod-wheel add</span><input type="range" min="0" max="50" step="1" value={patch.vibrato.modWheelDepthCents} {disabled} onchange={(event) => update((next) => { next.vibrato.modWheelDepthCents = Number(event.currentTarget.value); })} /><output>{patch.vibrato.modWheelDepthCents.toFixed(0)} c</output></label>
			</div>
			<p>Glide remains in Contrapunk’s shared Slide controls so one authoritative trajectory drives internal audio and routed MIDI.</p>
		</div>
	</section>
</section>

<style>
	.oscillator { min-width: 0; display: grid; gap: 9px; }
	.panel-title, .interaction > header, .trajectory > header { display: flex; align-items: end; justify-content: space-between; gap: 16px; }
	.panel-title { min-height: 44px; padding: 8px 10px; border: 1px solid #3b3b3b; background: #292929; }
	.panel-title div, .interaction header div, .trajectory header { min-width: 0; }
	h2, h3 { margin: 1px 0 0; color: #ddd; font: 600 12px var(--font-ui); letter-spacing: .04em; }
	.panel-title span, .panel-title a, .interaction header a, .trajectory header span, legend { color: #7f9eb2; font: 700 8px var(--font-code); letter-spacing: .11em; }
	.panel-title a, .interaction header a { text-decoration: none; }
	.panel-title a:hover, .interaction header a:hover { color: #d6e5ed; }
	.panel-title label { display: grid; grid-template-columns: auto minmax(150px, 210px); align-items: center; gap: 8px; }
	label > span, summary { color: #999; font: 9px var(--font-code); }
	select, input { accent-color: #8aaac0; }
	select { min-width: 0; height: 28px; border: 1px solid #4a4a4a; background: #1b1b1b; color: #ddd; font: 10px var(--font-ui); }
	.scope { border: 1px solid #3b3b3b; background: #151515; }
	svg { display: block; width: 100%; min-height: 260px; }
	svg text { fill: #777; font: 8px var(--font-code); letter-spacing: .05em; }
	.grid { fill: none; stroke: #292929; stroke-width: 1; }
	.wave { fill: none; stroke: #8aaac0; stroke-width: 2; vector-effect: non-scaling-stroke; }
	.spectral-line { stroke: #c49268; stroke-width: 3; vector-effect: non-scaling-stroke; }
	.frequency { fill: #b79b82; font-size: 7px; }
	.envelope { fill: none; stroke: #7ca27a; stroke-width: 1.5; stroke-dasharray: 4 3; }
	.scope-legend { display: flex; gap: 15px; padding: 5px 9px 7px; border-top: 1px solid #292929; color: #777; font: 8px var(--font-code); text-transform: uppercase; }
	.scope-legend span::before { content: ''; display: inline-block; width: 12px; height: 2px; margin: 0 5px 2px 0; background: #8aaac0; }
	.scope-legend span:nth-child(2)::before { background: #c49268; }
	.scope-legend span:nth-child(3)::before { background: #7ca27a; }
	.harmonics { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 7px; padding: 10px; border: 1px solid #3b3b3b; background: #222; }
	.harmonics legend { padding: 0 5px; }
	.harmonics label, .phase-grid label, .control-grid label { display: grid; min-width: 0; grid-template-columns: minmax(74px, .8fr) minmax(70px, 1fr) 48px; align-items: center; gap: 6px; }
	.harmonics label span { display: grid; }
	.harmonics b { color: #ddd; font: 600 10px var(--font-ui); }
	.harmonics small { color: #777; font: 7px var(--font-code); }
	input[type='range'] { width: 100%; min-width: 0; }
	output { color: #aaa; font: 8px var(--font-code); text-align: right; }
	.phase-editor { border: 1px solid #353535; background: #1d1d1d; }
	.phase-editor summary { display: flex; justify-content: space-between; padding: 8px 10px; cursor: pointer; }
	.phase-editor summary span { color: #666; }
	.phase-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 7px 16px; padding: 8px 10px 10px; border-top: 1px solid #303030; }
	.interaction, .trajectory { padding: 10px; border: 1px solid #3b3b3b; background: #222; }
	.interaction > header { align-items: start; padding-bottom: 9px; border-bottom: 1px solid #333; }
	.interaction header p, .trajectory p { max-width: 460px; margin: 0; color: #858585; font: 9px/1.4 var(--font-ui); }
	.mode-switch { display: flex; gap: 4px; margin: 9px 0; }
	button { min-height: 27px; padding: 0 12px; border: 1px solid #454545; background: #292929; color: #999; font: 9px var(--font-ui); }
	button.active { border-color: #8aaac0; background: #26333b; color: #d7e4eb; }
	.control-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 7px 14px; }
	.component-readout { display: grid; grid-template-columns: repeat(4, 1fr); gap: 1px; margin: 9px 0 0; background: #383838; }
	.component-readout span { padding: 6px; background: #191919; color: #aaa; font: 8px var(--font-code); text-align: center; }
	.trajectory-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 9px; }
	.trajectory header { align-items: start; justify-content: start; margin-bottom: 9px; padding-bottom: 7px; border-bottom: 1px solid #333; }
	.trajectory header span { display: block; }
	.envelope-controls, .vibrato-controls { grid-template-columns: 1fr; }
	.vibrato-controls { margin-bottom: 10px; }
	input:disabled, select:disabled, button:disabled { opacity: .4; }
	@media (max-width: 850px) {
		.harmonics, .phase-grid, .trajectory-grid { grid-template-columns: 1fr; }
		.control-grid { grid-template-columns: 1fr; }
		.component-readout { grid-template-columns: repeat(2, 1fr); }
	}
</style>
