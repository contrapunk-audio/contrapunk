<script lang="ts">
	import { phrase } from '$lib/stores/phrase.svelte';

	function setGap(value: number) {
		void phrase.setGapBeats(value);
	}
</script>

<section class="phrase-control" aria-labelledby="phrase-control-title">
	<div class="copy">
		<div class="eyebrow">PLAYER PHRASING</div>
		<h3 id="phrase-control-title">Phrase Gap</h3>
		<p>Silence after every released note and raised sustain pedal before the next phrase begins.</p>
	</div>
	<div class="control">
		<label>
			<span class="sr-only">Phrase gap in beats</span>
			<input
				type="range"
				min="0.5"
				max="16"
				step="0.25"
				value={phrase.gapBeats}
				disabled={!phrase.loaded}
				onchange={(event) => setGap(Number(event.currentTarget.value))}
			/>
		</label>
		<label class="number">
			<input
				aria-label="Phrase gap in beats"
				type="number"
				min="0.5"
				max="16"
				step="0.25"
				value={phrase.gapBeats}
				disabled={!phrase.loaded}
				onchange={(event) => setGap(Number(event.currentTarget.value))}
			/>
			<span>beats</span>
		</label>
	</div>
	<div class="status" aria-live="polite">
		<span class:live={phrase.phase !== 'idle'}></span>
		<strong>{phrase.statusLabel}</strong>
		<small>{phrase.id === null ? 'No active phrase' : `Phrase ${phrase.id} · ${phrase.state.attackCount} attacks`}</small>
	</div>
	{#if phrase.error}<p class="error" role="alert">{phrase.error}</p>{/if}
</section>

<style>
	.phrase-control { display: grid; grid-template-columns: minmax(220px, 1fr) minmax(260px, 1.4fr) minmax(150px, .7fr); align-items: center; gap: 18px; margin-bottom: 12px; padding: 14px; border: 1px solid var(--proto-line, var(--color-border)); background: var(--proto-surface, var(--color-widget-bg)); }
	.eyebrow { margin-bottom: 3px; color: var(--proto-dim, var(--color-text-tertiary)); font: 700 8px var(--font-code); letter-spacing: .13em; }
	h3 { margin: 0; font-size: 14px; font-weight: 650; }
	p { margin: 4px 0 0; color: var(--proto-muted, var(--color-text-secondary)); font-size: 9px; line-height: 1.4; }
	.control { display: grid; grid-template-columns: minmax(120px, 1fr) 94px; align-items: center; gap: 10px; }
	.control input[type='range'] { width: 100%; accent-color: var(--proto-text, var(--color-text-primary)); }
	.number { display: grid; grid-template-columns: 52px auto; align-items: center; gap: 5px; color: var(--proto-muted, var(--color-text-secondary)); font: 8px var(--font-code); }
	.number input { width: 100%; box-sizing: border-box; padding: 6px; border: 1px solid var(--proto-line-strong, var(--color-border)); background: #080808; color: var(--proto-text, var(--color-text-primary)); font: 10px var(--font-code); }
	.status { display: grid; grid-template-columns: 7px 1fr; grid-template-rows: auto auto; align-items: center; column-gap: 7px; }
	.status > span { grid-row: 1 / 3; width: 6px; height: 6px; border: 1px solid var(--proto-muted, var(--color-text-secondary)); border-radius: 50%; }
	.status > span.live { border-color: #4fe8c3; background: #4fe8c3; box-shadow: 0 0 7px #4fe8c3; }
	.status strong { font-size: 10px; }
	.status small { color: var(--proto-muted, var(--color-text-secondary)); font: 7px var(--font-code); }
	.error { grid-column: 1 / -1; color: #ff7a91; }
	.sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; }
	@media (max-width: 760px) { .phrase-control { grid-template-columns: 1fr; gap: 10px; } }
</style>
