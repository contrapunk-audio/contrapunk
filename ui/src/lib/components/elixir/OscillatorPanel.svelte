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
	const previewSampleRate = 48_000;
	const previewFrameCount = Math.ceil(2 * previewSampleRate / previewFundamental) + 1;
	const roleNames = ['Input', 'Harmony', 'Canon', 'Counterpoint'];
	let roleName = $derived(roleNames[role] ?? 'Input');
	let patch = $derived(synth.rolePatches[role] ?? defaultRolePatch());
	let selectedRecipe = $derived(recipeName(patch.harmonics.amplitudes));
	let secondaryFrequency = $derived(
		previewFundamental * 2 ** ((patch.secondary.semitones + patch.secondary.fineCents / 100) / 12)
	);
	let differenceFrequency = $derived(Math.abs(previewFundamental - secondaryFrequency));
	let sumFrequency = $derived(previewFundamental + secondaryFrequency);
	let outputFormula = $derived(
		patch.secondary.mode === 'add' ? 'A + B' : patch.secondary.mode === 'ring' ? 'A × B' : 'A'
	);

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

	type SignalPoint = { sourceA: number; sourceB: number; output: number };
	let signalPoints = $derived.by((): SignalPoint[] => {
		const amplitudes = patch.harmonics.amplitudes;
		const phases = patch.harmonics.phases;
		const energy = Math.sqrt(amplitudes.reduce((sum, amplitude) => sum + amplitude ** 2, 0)) || 1;
		const ratio = secondaryFrequency / previewFundamental;
		return Array.from({ length: previewFrameCount }, (_, index) => {
			const cycles = index * previewFundamental / previewSampleRate;
			const sourceA = amplitudes.reduce(
				(sum, amplitude, harmonic) => sum + amplitude * Math.sin(2 * Math.PI * ((harmonic + 1) * cycles + phases[harmonic])),
				0
			) / energy;
			const sourceB = patch.secondary.level * Math.sin(2 * Math.PI * (ratio * cycles + patch.secondary.phase));
			const output = patch.secondary.mode === 'add'
				? sourceA + sourceB
				: patch.secondary.mode === 'ring'
					? sourceA * sourceB
					: sourceA;
			return { sourceA, sourceB, output };
		});
	});

	function peak(field: keyof SignalPoint) {
		return Math.max(0, ...signalPoints.map((point) => Math.abs(point[field])));
	}

	let sourceAPeak = $derived(peak('sourceA'));
	let sourceBPeak = $derived(peak('sourceB'));
	let outputPeak = $derived(peak('output'));
	let signalScale = $derived(Math.max(
		1,
		Math.ceil(Math.max(sourceAPeak, patch.secondary.mode === 'primary_only' ? 0 : sourceBPeak, outputPeak) * 4) / 4
	));

	function signalPath(field: keyof SignalPoint) {
		return signalPoints.map((point, index) => {
			const x = 30 + index / (signalPoints.length - 1) * 674;
			const y = 64 - point[field] / signalScale * 46;
			return `${index ? 'L' : 'M'}${x.toFixed(2)} ${y.toFixed(2)}`;
		}).join(' ');
	}

	let sourceAPath = $derived(signalPath('sourceA'));
	let sourceBPath = $derived(signalPath('sourceB'));
	let outputPath = $derived(signalPath('output'));

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
		const sustainY = 90 - patch.envelope.sustainLevel * 75;
		return `M0 90 L80 10 L150 ${sustainY.toFixed(2)} L300 ${sustainY.toFixed(2)} L400 90`;
	});
	let vibratoPath = $derived.by(() => Array.from({ length: 121 }, (_, index) => {
		const position = index / 120;
		const depth = patch.vibrato.depthCents / 50;
		const x = position * 400;
		const y = 50 - Math.sin(2 * Math.PI * patch.vibrato.rateHz * position) * depth * 36;
		return `${index ? 'L' : 'M'}${x.toFixed(2)} ${y.toFixed(2)}`;
	}).join(' '));
</script>

<section class="oscillator" aria-labelledby="oscillator-heading">
	<header class="panel-title">
		<div><span>TONE GENERATOR</span><h2 id="oscillator-heading">{roleName} oscillator</h2></div>
		<label>
			<span>Wave recipe</span>
			<select aria-label="Wave recipe" value={selectedRecipe} {disabled} onchange={(event) => setRecipe(event.currentTarget.value as HarmonicRecipeName)}>
				<option value="sine">Sine</option>
				<option value="three">Three harmonics</option>
				<option value="odd">Odd only</option>
				<option value="saw">Saw-like 1/k</option>
				<option value="dark">Dark 1/k²</option>
				{#if selectedRecipe === 'custom'}<option value="custom">Custom</option>{/if}
			</select>
		</label>
	</header>

	<div class="workbench">
		<div class="control-bank">
			<fieldset class="harmonics" {disabled}>
				<legend>Oscillator A · harmonic mix</legend>
				{#each patch.harmonics.amplitudes as amplitude, index}
					<label>
						<span><b>H{index + 1}</b><small>{previewFundamental * (index + 1)} Hz</small></span>
						<input aria-label={`Harmonic ${index + 1} amplitude`} type="range" min="0" max="1" step="0.01" value={amplitude} oninput={(event) => setHarmonic(index, 'amplitudes', Number(event.currentTarget.value))} />
						<output>{amplitude.toFixed(2)}</output>
					</label>
				{/each}
			</fieldset>

			<details class="phase-editor">
				<summary>Phase offsets <span>Shape each partial</span></summary>
				<div class="phase-grid">
					{#each patch.harmonics.phases as phase, index}
						<label><span>H{index + 1}</span><input aria-label={`Harmonic ${index + 1} phase`} type="range" min="0" max="0.9972222" step="0.0027778" value={phase} {disabled} oninput={(event) => setHarmonic(index, 'phases', Number(event.currentTarget.value))} /><output>{Math.round(phase * 360)}°</output></label>
					{/each}
				</div>
			</details>

			<section class="interaction" aria-labelledby="interaction-heading">
				<header><div><span>OPERATOR B</span><h3 id="interaction-heading">Combine</h3></div><p>Add preserves both sources. Ring multiplies them.</p></header>
				<div class="mode-switch" role="group" aria-label="Oscillator interaction mode">
					{#each [['primary_only', 'A only'], ['add', 'Add'], ['ring', 'Ring']] as mode}
						<button type="button" class:active={patch.secondary.mode === mode[0]} aria-pressed={patch.secondary.mode === mode[0]} {disabled} onclick={() => setInteraction(mode[0] as SynthCombineMode)}>{mode[1]}</button>
					{/each}
				</div>
				<div class="control-grid">
					<label><span>B interval</span><input type="range" min="-24" max="24" step="1" value={patch.secondary.semitones} {disabled} oninput={(event) => update((next) => { next.secondary.semitones = Number(event.currentTarget.value); })} /><output>{patch.secondary.semitones > 0 ? '+' : ''}{patch.secondary.semitones.toFixed(0)} st</output></label>
					<label><span>B fine</span><input type="range" min="-100" max="100" step="1" value={patch.secondary.fineCents} {disabled} oninput={(event) => update((next) => { next.secondary.fineCents = Number(event.currentTarget.value); })} /><output>{patch.secondary.fineCents > 0 ? '+' : ''}{patch.secondary.fineCents.toFixed(0)} c</output></label>
					<label><span>B phase</span><input aria-label="Operator B phase" type="range" min="0" max="0.9972222" step="0.0027778" value={patch.secondary.phase} {disabled} oninput={(event) => update((next) => { next.secondary.phase = Number(event.currentTarget.value); })} /><output>{Math.round(patch.secondary.phase * 360)}°</output></label>
					<label><span>B level</span><input type="range" min="0" max="1" step="0.01" value={patch.secondary.level} {disabled} oninput={(event) => update((next) => { next.secondary.level = Number(event.currentTarget.value); })} /><output>{Math.round(patch.secondary.level * 100)}%</output></label>
				</div>
				<p class="component-readout"><span>A {previewFundamental.toFixed(1)} Hz</span><span>B {secondaryFrequency.toFixed(1)} Hz</span><span>Difference {differenceFrequency.toFixed(1)} Hz</span><span>Sum {sumFrequency.toFixed(1)} Hz</span></p>
			</section>
		</div>

		<section class="signal-centre" aria-labelledby="signal-heading">
			<header>
				<div><span>SIGNAL FLOW</span><h3 id="signal-heading">Inputs → output</h3></div>
				<output>{outputFormula} · window peak {outputPeak.toFixed(3)}</output>
			</header>

			<div class="scope-card input-scope">
				<div class="scope-heading"><span>INPUT WAVES</span><output>A ±{sourceAPeak.toFixed(3)}{patch.secondary.mode !== 'primary_only' ? ` · B ±${sourceBPeak.toFixed(3)}` : ''}</output></div>
				<svg class="signal-preview" viewBox="0 0 734 128" role="img" aria-labelledby="input-scope-title input-scope-description">
					<title id="input-scope-title">Oscillator input waveforms</title>
					<desc id="input-scope-description">A two-cycle 48 kilohertz reference render of oscillator A{patch.secondary.mode === 'primary_only' ? '' : ' and oscillator B'} at 220 hertz.</desc>
					<path class="signal-grid" d="M30 18 H704 M30 64 H704 M30 110 H704 M30 18 V110 M198.5 18 V110 M367 18 V110 M535.5 18 V110 M704 18 V110" />
					<text x="4" y="21">+{signalScale.toFixed(2)}</text><text x="14" y="67">0</text><text x="4" y="113">−{signalScale.toFixed(2)}</text>
					<path class="source-a" d={sourceAPath} data-peak={sourceAPeak.toFixed(3)} />
					{#if patch.secondary.mode !== 'primary_only'}<path class="source-b" d={sourceBPath} data-peak={sourceBPeak.toFixed(3)} />{/if}
				</svg>
				<div class="scope-legend"><span>A harmonic source</span>{#if patch.secondary.mode !== 'primary_only'}<span>B sine operator</span>{/if}</div>
			</div>

			<div class="scope-card output-scope">
				<div class="scope-heading"><span>FINAL OSCILLATOR OUTPUT</span><output>{outputFormula} · ±{outputPeak.toFixed(3)}</output></div>
				<svg class="signal-preview" viewBox="0 0 734 128" role="img" aria-labelledby="output-scope-title output-scope-description" data-peak={outputPeak.toFixed(3)} data-formula={outputFormula}>
					<title id="output-scope-title">Final oscillator output waveform</title>
					<desc id="output-scope-description">The settled {outputFormula} operator samples from the same 48 kilohertz reference render, before articulation and gain.</desc>
					<path class="signal-grid" d="M30 18 H704 M30 64 H704 M30 110 H704 M30 18 V110 M198.5 18 V110 M367 18 V110 M535.5 18 V110 M704 18 V110" />
					<text x="4" y="21">+{signalScale.toFixed(2)}</text><text x="14" y="67">0</text><text x="4" y="113">−{signalScale.toFixed(2)}</text>
					<path class="final-output" d={outputPath} />
				</svg>
				<div class="scope-legend"><span>settled operator result</span><span>shared amplitude scale</span></div>
			</div>

			<div class="spectrum-card">
				<div class="scope-heading"><span>OUTPUT COMPONENTS</span><output>0–{Math.round(spectrumFrequencyMaximum)} Hz</output></div>
				<svg class="spectrum-preview" viewBox="0 0 734 108" role="img" aria-label="One-sided spectrum of the final oscillator output">
					<path class="signal-grid" d="M30 88 H704 M30 18 V88 M198.5 18 V88 M367 18 V88 M535.5 18 V88 M704 18 V88" />
					{#each spectrum as line}
						{@const x = 30 + Math.min(1, line.frequency / spectrumFrequencyMaximum) * 674}
						{@const top = 88 - line.amplitude / spectrumMaximum * 58}
						<line class="spectral-line" x1={x} y1="88" x2={x} y2={top} />
						<text class="frequency" x={x} y={Math.max(15, top - 5)} text-anchor="middle">{Math.round(line.frequency)}</text>
					{/each}
				</svg>
			</div>
		</section>
	</div>

	<section class="trajectory-grid">
		<div class="trajectory" aria-labelledby="amplitude-heading">
			<header><span>AMPLITUDE</span><h3 id="amplitude-heading">Articulation</h3></header>
			<svg class="trajectory-preview" viewBox="0 0 400 100" role="img" aria-label="Current attack, decay, sustain level, and release trajectory"><path class="trajectory-axis" d="M0 90 H400 M0 50 H400" /><path class="amplitude-trajectory" d={envelopePath} /></svg>
			<div class="control-grid envelope-controls">
				<label><span>Attack</span><input type="range" min="0" max="5" step="0.005" value={patch.envelope.attackSecs} {disabled} oninput={(event) => update((next) => { next.envelope.attackSecs = Number(event.currentTarget.value); })} /><output>{patch.envelope.attackSecs.toFixed(3)} s</output></label>
				<label><span>Decay</span><input type="range" min="0" max="5" step="0.01" value={patch.envelope.decaySecs} {disabled} oninput={(event) => update((next) => { next.envelope.decaySecs = Number(event.currentTarget.value); })} /><output>{patch.envelope.decaySecs.toFixed(2)} s</output></label>
				<label><span>Sustain level</span><input type="range" min="0" max="1" step="0.01" value={patch.envelope.sustainLevel} {disabled} oninput={(event) => update((next) => { next.envelope.sustainLevel = Number(event.currentTarget.value); })} /><output>{Math.round(patch.envelope.sustainLevel * 100)}%</output></label>
				<label><span>Release</span><input type="range" min="0" max="10" step="0.01" value={patch.envelope.releaseSecs} {disabled} oninput={(event) => update((next) => { next.envelope.releaseSecs = Number(event.currentTarget.value); })} /><output>{patch.envelope.releaseSecs.toFixed(2)} s</output></label>
				<label><span>Velocity</span><input type="range" min="0" max="1" step="0.01" value={patch.envelope.velocitySensitivity} {disabled} oninput={(event) => update((next) => { next.envelope.velocitySensitivity = Number(event.currentTarget.value); })} /><output>{Math.round(patch.envelope.velocitySensitivity * 100)}%</output></label>
				<label><span>Expression</span><input type="range" min="0" max="1" step="0.01" value={patch.envelope.expressionSensitivity} {disabled} oninput={(event) => update((next) => { next.envelope.expressionSensitivity = Number(event.currentTarget.value); })} /><output>{Math.round(patch.envelope.expressionSensitivity * 100)}%</output></label>
			</div>
		</div>
		<div class="trajectory" aria-labelledby="pitch-heading">
			<header><span>PITCH</span><h3 id="pitch-heading">Vibrato</h3></header>
			<svg class="trajectory-preview" viewBox="0 0 400 100" role="img" aria-label="Current one-second vibrato pitch trajectory"><path class="trajectory-axis" d="M0 50 H400" /><path class="pitch-trajectory" d={vibratoPath} /></svg>
			<div class="control-grid vibrato-controls">
				<label><span>Rate</span><input type="range" min="1" max="8" step="0.1" value={patch.vibrato.rateHz} {disabled} oninput={(event) => update((next) => { next.vibrato.rateHz = Number(event.currentTarget.value); })} /><output>{patch.vibrato.rateHz.toFixed(1)} Hz</output></label>
				<label><span>Depth</span><input type="range" min="0" max="50" step="1" value={patch.vibrato.depthCents} {disabled} oninput={(event) => update((next) => { next.vibrato.depthCents = Number(event.currentTarget.value); })} /><output>{patch.vibrato.depthCents.toFixed(0)} c</output></label>
				<label><span>Mod-wheel add</span><input type="range" min="0" max="50" step="1" value={patch.vibrato.modWheelDepthCents} {disabled} oninput={(event) => update((next) => { next.vibrato.modWheelDepthCents = Number(event.currentTarget.value); })} /><output>{patch.vibrato.modWheelDepthCents.toFixed(0)} c</output></label>
			</div>
			<p>Slide remains the shared glide path for internal audio and routed MIDI.</p>
		</div>
	</section>
</section>

<style>
	.oscillator { min-width: 0; display: grid; gap: 9px; }
	.panel-title, .interaction > header, .trajectory > header, .signal-centre > header { display: flex; align-items: end; justify-content: space-between; gap: 16px; }
	.panel-title { min-height: 44px; padding: 8px 10px; border: 1px solid #3b3b3b; background: #292929; }
	.panel-title div, .interaction header div, .trajectory header, .signal-centre header div { min-width: 0; }
	h2, h3 { margin: 1px 0 0; color: #ddd; font: 600 12px var(--font-ui); letter-spacing: .04em; }
	.panel-title span, .interaction header span, .trajectory header span, .signal-centre > header span, legend { color: #7f9eb2; font: 700 8px var(--font-code); letter-spacing: .11em; }
	.panel-title label { display: grid; grid-template-columns: auto minmax(150px, 210px); align-items: center; gap: 8px; }
	label > span, summary { color: #999; font: 9px var(--font-code); }
	select, input { accent-color: #8aaac0; }
	select { min-width: 0; height: 28px; border: 1px solid #4a4a4a; background: #1b1b1b; color: #ddd; font: 10px var(--font-ui); }
	.workbench { display: grid; grid-template-columns: minmax(310px, .78fr) minmax(430px, 1.22fr); gap: 9px; align-items: start; }
	.control-bank { min-width: 0; display: grid; gap: 9px; }
	.harmonics { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px 10px; padding: 10px; border: 1px solid #3b3b3b; background: #222; }
	.harmonics legend { padding: 0 5px; }
	.harmonics label { display: grid; min-width: 0; grid-template-columns: minmax(42px, .7fr) minmax(40px, 1fr) 34px; align-items: center; gap: 4px; }
	.phase-grid label, .control-grid label { display: grid; min-width: 0; grid-template-columns: minmax(66px, .8fr) minmax(62px, 1fr) 44px; align-items: center; gap: 6px; }
	.harmonics label span { display: grid; }
	.harmonics b { color: #ddd; font: 600 10px var(--font-ui); }
	.harmonics small { color: #777; font: 7px var(--font-code); }
	input[type='range'] { width: 100%; min-width: 0; }
	output { color: #aaa; font: 8px var(--font-code); text-align: right; }
	.phase-editor { border: 1px solid #353535; background: #1d1d1d; }
	.phase-editor summary { display: flex; justify-content: space-between; padding: 8px 10px; cursor: pointer; }
	.phase-editor summary span { color: #666; }
	.phase-grid { display: grid; grid-template-columns: 1fr; gap: 7px; padding: 8px 10px 10px; border-top: 1px solid #303030; }
	.interaction, .trajectory { padding: 10px; border: 1px solid #3b3b3b; background: #222; }
	.interaction > header { align-items: start; padding-bottom: 9px; border-bottom: 1px solid #333; }
	.interaction header p, .trajectory p { max-width: 280px; margin: 0; color: #858585; font: 9px/1.4 var(--font-ui); }
	.mode-switch { display: flex; gap: 4px; margin: 9px 0; }
	button { min-height: 27px; padding: 0 12px; border: 1px solid #454545; background: #292929; color: #999; font: 9px var(--font-ui); }
	button.active { border-color: #8aaac0; background: #26333b; color: #d7e4eb; }
	.control-grid { display: grid; grid-template-columns: 1fr; gap: 7px; }
	.component-readout { display: grid; grid-template-columns: repeat(2, 1fr); gap: 1px; margin: 9px 0 0; background: #383838; }
	.component-readout span { padding: 6px; background: #191919; color: #aaa; font: 8px var(--font-code); text-align: center; }
	.signal-centre { min-width: 0; display: grid; gap: 8px; padding: 10px; border: 1px solid #4a555a; background: linear-gradient(180deg, #1d2326 0%, #16191b 100%); box-shadow: inset 0 0 42px rgba(95, 143, 164, .045); }
	.signal-centre > header { min-height: 31px; padding: 0 2px 7px; border-bottom: 1px solid #354147; }
	.signal-centre > header output { color: #b1c7d0; font-size: 9px; }
	.scope-card, .spectrum-card { overflow: hidden; border: 1px solid #334047; background: #101416; }
	.output-scope { border-color: #466259; background: #0f1614; box-shadow: 0 0 18px rgba(115, 178, 151, .07); }
	.scope-heading { min-height: 24px; display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 0 8px; border-bottom: 1px solid #283238; }
	.scope-heading span { color: #8199a4; font: 700 7px var(--font-code); letter-spacing: .12em; }
	.output-scope .scope-heading span { color: #8fb6a5; }
	.scope-heading output { font-size: 7px; }
	svg { display: block; width: 100%; }
	svg text { fill: #667177; font: 7px var(--font-code); }
	.signal-preview { min-height: 132px; }
	.spectrum-preview { min-height: 100px; }
	.signal-grid { fill: none; stroke: #263036; stroke-width: 1; }
	.source-a, .source-b, .final-output { fill: none; stroke-width: 2; vector-effect: non-scaling-stroke; }
	.source-a { stroke: #8eb8cf; }
	.source-b { stroke: #d3a36f; }
	.final-output { stroke: #9dd1b9; stroke-width: 2.5; filter: drop-shadow(0 0 3px rgba(157, 209, 185, .32)); }
	.scope-legend { display: flex; gap: 15px; padding: 5px 8px 7px; border-top: 1px solid #222d32; color: #68757b; font: 7px var(--font-code); text-transform: uppercase; }
	.scope-legend span::before { content: ''; display: inline-block; width: 12px; height: 2px; margin: 0 5px 2px 0; background: #8eb8cf; }
	.input-scope .scope-legend span:nth-child(2)::before { background: #d3a36f; }
	.output-scope .scope-legend span::before { background: #9dd1b9; }
	.output-scope .scope-legend span:nth-child(2)::before { background: #53636a; }
	.spectral-line { stroke: #c49268; stroke-width: 3; vector-effect: non-scaling-stroke; }
	.frequency { fill: #c1a487; font-size: 7px; }
	.trajectory-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 9px; }
	.trajectory header { align-items: start; justify-content: start; margin-bottom: 9px; padding-bottom: 7px; border-bottom: 1px solid #333; }
	.trajectory header span { display: block; }
	.trajectory-preview { min-height: 76px; margin-bottom: 9px; border: 1px solid #303030; background: #171717; }
	.trajectory-axis { fill: none; stroke: #292929; stroke-width: 1; }
	.amplitude-trajectory, .pitch-trajectory { fill: none; stroke-width: 2; vector-effect: non-scaling-stroke; }
	.amplitude-trajectory { stroke: #7ca27a; }
	.pitch-trajectory { stroke: #8aaac0; }
	.envelope-controls, .vibrato-controls { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 7px 14px; }
	.vibrato-controls { margin-bottom: 10px; }
	input:disabled, select:disabled, button:disabled { opacity: .4; }
	@media (max-width: 1100px) {
		.workbench { grid-template-columns: 1fr; }
		.signal-centre { grid-row: 1; }
		.control-bank { grid-template-columns: 1fr 1fr; }
		.interaction { grid-column: 1 / -1; }
		.phase-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
	}
	@media (max-width: 760px) {
		.panel-title { align-items: stretch; flex-direction: column; }
		.panel-title label { grid-template-columns: 1fr; }
		.control-bank, .trajectory-grid, .harmonics, .phase-grid, .envelope-controls, .vibrato-controls { grid-template-columns: 1fr; }
		.interaction { grid-column: auto; }
		.signal-centre { padding: 7px; }
		.scope-heading output { max-width: 50%; }
	}
</style>
