<script lang="ts">
	import { onDestroy } from 'svelte';
	import { adapter, type HarmonicLimit, type TuningStyle } from '$lib/adapter';
	import { engine } from '$lib/stores/engine.svelte';

	const styles: Array<{ value: TuningStyle; label: string; description: string }> = [
		{ value: 'standard', label: 'Standard', description: 'Familiar piano and synthesizer tuning.' },
		{ value: 'pure', label: 'Pure', description: 'Generated voices adjust so chords blend more cleanly.' }
	];

	function setStyle(style: TuningStyle) {
		void engine.setTuningStyle(style);
	}

	function setDepth(event: Event) {
		void engine.setTuningDepth(Number((event.currentTarget as HTMLInputElement).value));
	}

	function setLimit(event: Event) {
		void engine.setHarmonicLimit((event.currentTarget as HTMLSelectElement).value as HarmonicLimit);
	}

	let comparing = $state(false);
	let compareRequests = Promise.resolve();
	function compare(enabled: boolean) {
		if (comparing === enabled) return;
		comparing = enabled;
		compareRequests = compareRequests
			.then(() => adapter.setTuningCompare(enabled))
			.catch((error) => console.error('Could not compare tuning', error));
	}

	$effect(() => {
		if (engine.tuningStyle !== 'pure' && comparing) compare(false);
	});

	onDestroy(() => {
		if (comparing) compare(false);
	});
</script>

<section class="tuning" aria-labelledby="tuning-heading">
	<div class="heading">
		<div>
			<h3 id="tuning-heading">Tuning</h3>
			<p>{styles.find((style) => style.value === engine.tuningStyle)?.description}</p>
		</div>
		<div class="style-switch" role="group" aria-label="Tuning style">
			{#each styles as style}
				<button
					type="button"
					class:active={engine.tuningStyle === style.value}
					aria-pressed={engine.tuningStyle === style.value}
					onclick={() => setStyle(style.value)}
				>{style.label}</button>
			{/each}
		</div>
	</div>

	{#if engine.tuningStyle === 'pure'}
		<label class="depth">
			<span>Depth <output>{Math.round(engine.tuningDepth * 100)}%</output></span>
			<small>Familiar</small>
			<input
				type="range"
				min="0"
				max="1"
				step="0.01"
				value={engine.tuningDepth}
				onchange={setDepth}
			/>
			<small>Full</small>
		</label>

		<button
			type="button"
			class="compare"
			class:active={comparing}
			aria-pressed={comparing}
			onpointerdown={(event) => {
				event.currentTarget.setPointerCapture(event.pointerId);
				compare(true);
			}}
			onpointerup={() => compare(false)}
			onpointercancel={() => compare(false)}
			onlostpointercapture={() => compare(false)}
			onblur={() => compare(false)}
			onkeydown={(event) => {
				if (!event.repeat && (event.key === ' ' || event.key === 'Enter')) compare(true);
			}}
			onkeyup={(event) => {
				if (event.key === ' ' || event.key === 'Enter') compare(false);
			}}
		>Hold to compare Standard</button>

		<details>
			<summary>Advanced</summary>
			<label class="limit">
				<span>Harmonic character</span>
				<select value={engine.harmonicLimit} onchange={setLimit}>
					<option value="five">Clean thirds and fifths</option>
					<option value="seven">Wider harmonic color</option>
				</select>
			</label>
		</details>
	{/if}
</section>

<style>
	.tuning {
		margin-top: 12px;
		padding: 12px;
		border: 1px solid var(--color-border, #3a3a3a);
		background: var(--color-surface, #242424);
	}
	.heading { display: flex; align-items: center; justify-content: space-between; gap: 16px; }
	h3 { margin: 0; color: var(--color-text, #ddd); font: 600 12px var(--font-ui); text-transform: uppercase; letter-spacing: .06em; }
	p { margin: 4px 0 0; color: var(--color-text-muted, #999); font: 11px var(--font-ui); }
	.style-switch { display: flex; gap: 4px; }
	button, select {
		height: 30px;
		border: 1px solid var(--color-border, #484848);
		background: var(--color-surface-raised, #303030);
		color: var(--color-text-muted, #aaa);
		font: 11px var(--font-ui);
	}
	button { padding: 0 12px; }
	button.active { border-color: #668563; color: #c1d8bd; background: #2d372d; }
	button.compare { margin-top: 12px; width: 100%; }
	button.compare.active { border-color: #7c9eb8; color: #d1e3ee; background: #29343b; }
	.depth { margin-top: 14px; display: grid; grid-template-columns: auto minmax(140px, 1fr) auto; align-items: center; gap: 8px; }
	.depth > span { grid-column: 1 / -1; display: flex; justify-content: space-between; color: var(--color-text, #ccc); font: 11px var(--font-ui); }
	.depth output { color: var(--color-text-muted, #999); font-family: var(--font-code); }
	.depth input { width: 100%; accent-color: #7c9eb8; }
	small { color: var(--color-text-muted, #888); font: 10px var(--font-ui); }
	details { margin-top: 10px; color: var(--color-text-muted, #999); font: 10px var(--font-ui); }
	summary { cursor: pointer; }
	.limit { margin-top: 10px; display: flex; align-items: center; justify-content: space-between; gap: 12px; }
	.limit select { min-width: 190px; padding: 0 8px; }
	@media (max-width: 620px) {
		.heading { align-items: flex-start; flex-direction: column; }
	}
</style>
