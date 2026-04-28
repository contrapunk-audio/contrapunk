<!--
	PatternPanel — beat-aligned chord trigger pattern programmer.

	Owns the UI for the pattern store: three pickers (Subdivision /
	Length / Input mode), Clear/Default buttons, and a clickable cell
	grid. Currently-playing cell pulses via transport.totalBeat.

	Lives below the harmony controls in either Simple or Advanced view —
	gated by `ui.panels.pattern`. Engine-level feature, not view-specific.
-->

<script lang="ts">
	import { pattern } from '$lib/stores/pattern.svelte';
	import { onMount, onDestroy } from 'svelte';
	import {
		SUBDIVISION_OPTIONS,
		LENGTH_OPTIONS,
		INPUT_MODE_OPTIONS,
		type Subdivision,
		type Length,
		type InputMode
	} from '$lib/stores/pattern.svelte';
	import { transport } from '$lib/stores/transport.svelte';
	import PixelSelect from './PixelSelect.svelte';
	import BeatGrid from './BeatGrid.svelte';

	// Pattern feature is enabled while this panel is mounted (the user
	// has the panel pip on). Dispatch master enable on mount, disable on
	// unmount — the router falls back to today's real-time path when off.
	onMount(() => {
		void pattern.setEnabled(true);
	});
	onDestroy(() => {
		void pattern.setEnabled(false);
	});

	// Currently-playing cell index, derived from transport. Re-evaluates
	// whenever totalBeat advances (driven by the audio-clock beat-update
	// events that the transport store applies).
	let currentCell = $derived(
		transport.running ? pattern.cellIndexAt(transport.totalBeat) : -1
	);

	function onSubdivisionChange(v: string) {
		pattern.setSubdivision(parseInt(v, 10) as Subdivision);
	}
	function onLengthChange(v: string) {
		pattern.setLength(parseInt(v, 10) as Length);
	}
	function onInputModeChange(v: string) {
		pattern.setInputMode(v as InputMode);
	}

	// Beat dividers: a vertical line every `subdivision` cells, marking
	// quarter-note boundaries within the loop.
	function isBeatDivider(idx: number): boolean {
		return idx > 0 && idx % pattern.subdivision === 0;
	}

	// Bar dividers: a stronger line every `beatsPerBar × subdivision`
	// cells, marking bar boundaries within multi-bar loops.
	function isBarDivider(idx: number): boolean {
		const cellsPerBar = pattern.beatsPerBar * pattern.subdivision;
		return idx > 0 && idx % cellsPerBar === 0;
	}

	const subdivOptions = SUBDIVISION_OPTIONS.map((o) => ({
		value: String(o.value),
		label: o.label
	}));
	const lengthOptions = LENGTH_OPTIONS.map((o) => ({
		value: String(o.value),
		label: o.label
	}));
	const inputOptions = INPUT_MODE_OPTIONS.map((o) => ({
		value: o.value,
		label: o.label
	}));
</script>

<div class="pattern-panel">
	<div class="pattern-header font-ui">
		<span class="title">Pattern</span>

		<label class="picker">
			<span class="picker-label">Subdiv</span>
			<PixelSelect
				options={subdivOptions}
				value={String(pattern.subdivision)}
				onchange={onSubdivisionChange}
				small
			/>
		</label>

		<label class="picker">
			<span class="picker-label">Length</span>
			<PixelSelect
				options={lengthOptions}
				value={String(pattern.length)}
				onchange={onLengthChange}
				small
			/>
		</label>

		<label class="picker">
			<span class="picker-label">Input</span>
			<PixelSelect
				options={inputOptions}
				value={pattern.inputMode}
				onchange={onInputModeChange}
				small
			/>
		</label>

		<div class="spacer"></div>

		<button
			type="button"
			class="action-btn pixel-btn"
			onclick={() => pattern.clear()}
			title="Turn all cells off"
		>
			Clear
		</button>
		<button
			type="button"
			class="action-btn pixel-btn"
			onclick={() => pattern.resetDefault()}
			title="All cells on (chord on every beat)"
		>
			Default
		</button>
	</div>

	<BeatGrid
		count={pattern.cells.length}
		cellOn={(i) => !!pattern.cells[i]}
		currentIndex={currentCell}
		onClick={(i) => pattern.toggleCell(i)}
		isBeatDivider={isBeatDivider}
		isBarDivider={isBarDivider}
		variant="cell"
		ariaLabel="Pattern cells"
	/>
</div>

<style>
	.pattern-panel {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 8px 10px;
		border: 1px solid var(--color-border);
		background: var(--color-bg-card, rgba(0, 0, 0, 0.2));
	}

	.pattern-header {
		display: flex;
		align-items: center;
		gap: 12px;
		font-size: var(--font-size-xs);
		flex-wrap: wrap;
	}

	.title {
		color: var(--color-accent-amber);
		letter-spacing: 0.1em;
	}

	.picker {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.picker-label {
		color: var(--color-text-dim);
	}

	.spacer {
		flex: 1;
	}

	.action-btn {
		padding: 3px 8px;
		font-size: var(--font-size-xs);
		background: transparent;
		border: 1px solid var(--color-border);
		color: var(--color-text-dim);
		cursor: pointer;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}
	.action-btn:hover {
		border-color: var(--color-accent-cyan);
		color: var(--color-accent-cyan);
	}

	.cell-grid {
		display: flex;
		flex-wrap: wrap;
		gap: 2px;
		align-items: stretch;
	}

	.cell {
		flex: 1 1 0;
		min-width: 16px;
		height: 28px;
		background: transparent;
		border: 1px solid var(--color-border);
		cursor: pointer;
		padding: 0;
		transition: background-color 0.05s linear, border-color 0.05s linear;
	}

	.cell:hover {
		border-color: var(--color-accent-cyan);
	}

	.cell.on {
		background: var(--color-accent-teal);
		border-color: var(--color-accent-cyan);
	}

	.cell.playing {
		box-shadow: 0 0 8px var(--color-accent-amber);
		border-color: var(--color-accent-amber);
	}

	.cell.on.playing {
		background: var(--color-accent-amber);
	}

	/* Beat dividers: thicker left edge to mark quarter-note groups */
	.cell.beat-divider {
		margin-left: 4px;
	}

	/* Bar dividers: even larger gap to mark bar boundaries */
	.cell.bar-divider {
		margin-left: 8px;
	}
</style>
