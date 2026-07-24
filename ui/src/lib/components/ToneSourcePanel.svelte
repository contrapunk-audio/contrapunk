<script lang="ts">
	import { onDestroy } from 'svelte';
	import { adapter } from '$lib/adapter';

	const noteNames = ['C', 'C♯', 'D', 'E♭', 'E', 'F', 'F♯', 'G', 'A♭', 'A', 'B♭', 'B'];
	let pitchClass = $state(0);
	let octave = $state(4);
	let velocity = $state(100);
	let sounding = $state<number | null>(null);
	let operation = Promise.resolve();

	let midiNote = $derived(Math.max(0, Math.min(127, (octave + 1) * 12 + pitchClass)));
	let frequency = $derived(440 * 2 ** ((midiNote - 69) / 12));

	function updateGate(next: number | null) {
		operation = operation
			.then(async () => {
				const previous = sounding;
				if (next !== null && next !== previous) {
					await adapter.injectNoteOn(next, velocity);
				}
				if (previous !== null && previous !== next) {
					await adapter.injectNoteOff(previous);
				}
				sounding = next;
			})
			.catch(async (error) => {
				console.error('Tone source failed', error);
				if (sounding !== null) {
					try { await adapter.injectNoteOff(sounding); } catch { /* panic remains available */ }
				}
				sounding = null;
			});
	}

	function retarget() {
		if (sounding !== null) updateGate(midiNote);
	}

	onDestroy(() => {
		if (sounding !== null) updateGate(null);
	});
</script>

<div class="tone-source pixel-card">
	<div class="section-header font-ui">MONOPHONIC TONE</div>
	<p>One known-pitch source drives the same harmony, tuning, and Slide paths as a performed note.</p>
	<div class="controls">
		<label><span>Note</span><select bind:value={pitchClass} onchange={retarget}>{#each noteNames as name, index}<option value={index}>{name}</option>{/each}</select></label>
		<label><span>Octave</span><select bind:value={octave} onchange={retarget}>{#each [0, 1, 2, 3, 4, 5, 6, 7] as value}<option value={value}>{value}</option>{/each}</select></label>
		<label><span>Velocity</span><input type="number" min="1" max="127" bind:value={velocity} /></label>
		<div class="readout"><span>Frequency</span><strong>{frequency.toFixed(2)} Hz</strong></div>
		<button class:active={sounding !== null} type="button" aria-pressed={sounding !== null} onclick={() => updateGate(sounding === null ? midiNote : null)}>
			{sounding === null ? 'START TONE' : 'STOP TONE'}
		</button>
	</div>
	<div class="hint">Change Note or Octave while running to create a legato Slide.</div>
</div>

<style>
	.tone-source { padding: 10px; }
	.section-header { margin-bottom: 5px; color: var(--color-accent-gold); font-size: var(--font-size-xs); letter-spacing: 1.5px; }
	p, .hint { margin: 0 0 9px; color: var(--color-text-secondary); font-size: 11px; }
	.controls { display: grid; grid-template-columns: .8fr .65fr .8fr 1fr 1.15fr; align-items: end; gap: 7px; }
	label, .readout { display: grid; gap: 4px; }
	label span, .readout span { color: var(--color-text-dim); font: 9px var(--font-code); letter-spacing: .08em; text-transform: uppercase; }
	select, input, button { height: 32px; border: 1px solid var(--color-border); background: var(--color-bg-tertiary); color: var(--color-text-primary); }
	.readout strong { height: 30px; padding: 7px; border: 1px solid var(--color-border); font: 11px var(--font-code); }
	button { font: 700 10px var(--font-ui); }
	button.active { border-color: #4fe8c3; color: #4fe8c3; }
	.hint { margin: 8px 0 0; font: 9px var(--font-code); }
	@media (max-width: 760px) { .controls { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
</style>
