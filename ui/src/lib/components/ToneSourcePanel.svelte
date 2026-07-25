<script lang="ts">
	import { tone } from '$lib/stores/tone.svelte';

	let { compact = false }: { compact?: boolean } = $props();
</script>

<div class="tone-source pixel-card" class:compact>
	{#if !compact}
		<div class="section-header font-ui">MONOPHONIC TONE</div>
		<p>One known-pitch source drives the same harmony, tuning, and Slide paths as a performed note.</p>
	{/if}
	<div class="controls">
		<label>
			<span>Note</span>
			<select value={tone.pitchClass} onchange={(event) => tone.setPitchClass(Number(event.currentTarget.value))}>
				{#each tone.noteNames as name, index}<option value={index}>{name}</option>{/each}
			</select>
		</label>
		<label>
			<span>Octave</span>
			<select value={tone.octave} onchange={(event) => tone.setOctave(Number(event.currentTarget.value))}>
				{#each [0, 1, 2, 3, 4, 5, 6, 7] as value}<option value={value}>{value}</option>{/each}
			</select>
		</label>
		<label>
			<span>Velocity</span>
			<input type="number" min="1" max="127" value={tone.velocity} onchange={(event) => tone.setVelocity(Number(event.currentTarget.value))} />
		</label>
		<div class="readout"><span>Frequency</span><strong>{tone.frequency.toFixed(2)} Hz</strong></div>
		<button class:active={tone.desired !== null} type="button" aria-pressed={tone.desired !== null} onclick={() => tone.toggle()}>
			{compact ? (tone.desired === null ? 'START' : 'STOP') : (tone.desired === null ? 'START TONE' : 'STOP TONE')}
		</button>
	</div>
	{#if !compact}<div class="hint">Change Note or Octave while running to create a legato Slide.</div>{/if}
	{#if tone.error}<div class="error" role="alert">{tone.error}</div>{/if}
</div>

<style>
	.tone-source { padding: 10px; }
	.section-header { margin-bottom: 5px; color: var(--color-accent-gold); font-size: var(--font-size-xs); letter-spacing: 1.5px; }
	p, .hint { margin: 0 0 9px; color: var(--color-text-secondary); font-size: 11px; }
	.controls { display: grid; grid-template-columns: .8fr .65fr .8fr 1fr 1.15fr; align-items: end; gap: 7px; }
	label, .readout { display: grid; min-width: 0; gap: 4px; }
	label span, .readout span { color: var(--color-text-dim); font: 9px var(--font-code); letter-spacing: .08em; text-transform: uppercase; }
	select, input, button { width: 100%; min-width: 0; height: 32px; border: 1px solid var(--color-border); background: var(--color-bg-tertiary, var(--proto-surface)); color: var(--color-text-primary); }
	.readout strong { box-sizing: border-box; height: 32px; overflow: hidden; padding: 7px; border: 1px solid var(--color-border); font: 11px var(--font-code); text-overflow: ellipsis; white-space: nowrap; }
	button { padding: 0 7px; font: 700 10px var(--font-ui); }
	button.active { border-color: #4fe8c3; color: #4fe8c3; }
	.hint { margin: 8px 0 0; font: 9px var(--font-code); }
	.error { margin-top: 5px; color: #ff7a91; font: 8px var(--font-code); }
	.compact { padding: 0; border: 0; background: transparent; }
	.compact .controls { grid-template-columns: minmax(44px, .8fr) minmax(40px, .65fr) minmax(52px, .8fr) minmax(58px, .9fr); gap: 5px; }
	.compact .readout { display: none; }
	.compact label span { font-size: 7px; letter-spacing: .1em; }
	.compact select, .compact input, .compact button { height: 27px; font-size: 9px; }
	@media (max-width: 760px) { .controls { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
</style>
