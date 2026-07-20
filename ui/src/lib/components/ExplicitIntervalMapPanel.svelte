<script lang="ts">
	import {
		DEFAULT_EXPLICIT_INTERVAL_MAP,
		engine,
		type ExplicitIntervalMapConfig
	} from '$lib/stores/engine.svelte';

	let saving = $state(false);
	let error = $state('');

	function parseOffsets(raw: string): number[] {
		if (!raw.trim()) return [];
		const offsets = raw.split(/[\s,]+/).filter(Boolean).map(Number);
		if (
			offsets.length > 7 ||
			offsets.some(
				(offset, index) =>
					!Number.isInteger(offset) ||
					offset === 0 ||
					offset < -48 ||
					offset > 48 ||
					offsets.indexOf(offset) !== index
			)
		) {
			throw new Error('Use up to seven unique nonzero semitone offsets from −48 to +48.');
		}
		return offsets;
	}

	async function update(entry: number | 'fallback', raw: string) {
		const next: ExplicitIntervalMapConfig = {
			degreeOffsets: engine.explicitIntervalMap.degreeOffsets.map((offsets) => [...offsets]),
			fallbackOffsets: [...engine.explicitIntervalMap.fallbackOffsets]
		};
		try {
			const offsets = parseOffsets(raw);
			if (entry === 'fallback') next.fallbackOffsets = offsets;
			else next.degreeOffsets[entry] = offsets;
			saving = true;
			error = '';
			await engine.setExplicitIntervalMap(next);
		} catch (cause) {
			error = String(cause instanceof Error ? cause.message : cause);
		} finally {
			saving = false;
		}
	}

	async function reset() {
		saving = true;
		error = '';
		try {
			await engine.setExplicitIntervalMap(DEFAULT_EXPLICIT_INTERVAL_MAP);
		} catch (cause) {
			error = String(cause);
		} finally {
			saving = false;
		}
	}
</script>

<div class="interval-map-panel">
	<header>
		<div><p>EXPLICIT INTERVAL MAP</p><h3>Scale degree → semitone stack</h3></div>
		<button disabled={saving} onclick={reset}>Reset to fifths</button>
	</header>
	<p class="explanation">Each row maps the played note’s scale degree to direct semitone offsets from that exact note. Separate offsets with commas. The Voices control caps total output voices.</p>
	{#if error}<p class="error" role="alert">{error}</p>{/if}
	<div class="map-grid">
		{#each engine.explicitIntervalMap.degreeOffsets as offsets, index}
			<label>
				<span>Degree {index + 1}</span>
				<input
					type="text"
					inputmode="text"
					value={offsets.join(', ')}
					disabled={saving}
					aria-label={`Semitone offsets for scale degree ${index + 1}`}
					onchange={(event) => update(index, event.currentTarget.value)}
				/>
				<small>{offsets.length ? offsets.map((offset) => `${offset > 0 ? '+' : ''}${offset}`).join(' · ') : 'source only'}</small>
			</label>
		{/each}
		<label class="fallback">
			<span>Chromatic fallback</span>
			<input
				type="text"
				inputmode="text"
				value={engine.explicitIntervalMap.fallbackOffsets.join(', ')}
				disabled={saving}
				aria-label="Semitone offsets for notes outside the selected scale"
				onchange={(event) => update('fallback', event.currentTarget.value)}
			/>
			<small>{engine.explicitIntervalMap.fallbackOffsets.length ? engine.explicitIntervalMap.fallbackOffsets.map((offset) => `${offset > 0 ? '+' : ''}${offset}`).join(' · ') : 'source only'}</small>
		</label>
	</div>
</div>

<style>
	.interval-map-panel {
		--map-line: var(--proto-line, var(--color-border));
		--map-line-strong: var(--proto-line-strong, var(--color-border));
		--map-panel: var(--proto-panel, var(--color-bg-panel));
		--map-surface: var(--proto-surface, var(--color-widget-bg));
		--map-text: var(--proto-text, var(--color-text-primary));
		--map-muted: var(--proto-muted, var(--color-text-secondary));
		--map-dim: var(--proto-dim, var(--color-text-dim));
		margin-top: 14px;
		border: 1px solid var(--map-line);
		background: var(--map-panel);
	}
	header { display: flex; min-height: 50px; align-items: center; justify-content: space-between; gap: 12px; padding: 8px 12px; border-bottom: 1px solid var(--map-line); }
	header p { margin: 0 0 3px; color: var(--map-muted); font: 700 8px var(--font-code); letter-spacing: .14em; }
	h3 { margin: 0; color: var(--map-text); font-size: 13px; }
	button { min-height: 27px; padding: 0 9px; border: 1px solid var(--map-line-strong); background: transparent; color: var(--map-muted); font: 700 9px var(--font-code); }
	button:hover { border-color: var(--map-text); color: var(--map-text); }
	button:disabled { opacity: .45; }
	.explanation { margin: 0; padding: 8px 12px; border-bottom: 1px solid var(--map-line); color: var(--map-dim); font: 9px/1.45 var(--font-code); }
	.error { margin: 0; padding: 7px 12px; border-bottom: 1px solid #7b3030; color: #ffaaaa; font: 9px var(--font-code); }
	.map-grid { display: grid; grid-template-columns: repeat(4, minmax(120px, 1fr)); gap: 8px; padding: 12px; }
	label { display: grid; min-width: 0; gap: 4px; }
	label > span { color: var(--map-muted); font: 700 8px var(--font-code); letter-spacing: .08em; }
	input { box-sizing: border-box; width: 100%; height: 28px; border: 1px solid var(--map-line-strong); background: var(--map-surface); color: var(--map-text); font: 10px var(--font-code); }
	input:focus { border-color: var(--map-text); outline: 0; }
	small { overflow: hidden; color: var(--map-dim); font: 8px var(--font-code); text-overflow: ellipsis; white-space: nowrap; }
	.fallback { grid-column: span 2; }
	@media (max-width: 900px) { .map-grid { grid-template-columns: repeat(2, 1fr); } }
</style>
