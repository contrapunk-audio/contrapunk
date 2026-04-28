<!--
	BeatGrid — generic cell-strip primitive shared by TransportBar pips,
	PatternPanel programmer, and HistoryStrip's beat header. Each callsite
	passes its own visual variant via `variant`; the common structure
	(row of cells, current-playing highlight, optional click toggling,
	optional dividers) lives here.

	Variants:
	  - 'pip'        — small fixed pips (TransportBar / HistoryStrip header)
	  - 'cell'       — large clickable cells (PatternPanel)
	  - 'mini'       — tiny read-only cells (HistoryStrip pattern-mini)
-->

<script lang="ts">
	type Variant = 'pip' | 'cell' | 'mini';

	let {
		count,
		cellOn = () => false,
		currentIndex,
		onClick,
		isBeatDivider = () => false,
		isBarDivider = () => false,
		variant = 'cell',
		ariaLabel = ''
	}: {
		count: number;
		cellOn?: (i: number) => boolean;
		currentIndex?: number | null;
		onClick?: (i: number) => void;
		isBeatDivider?: (i: number) => boolean;
		isBarDivider?: (i: number) => boolean;
		variant?: Variant;
		ariaLabel?: string;
	} = $props();
</script>

<div class="beat-grid {variant}" role="group" aria-label={ariaLabel}>
	{#each Array(count).fill(0) as _, i (i)}
		<button
			type="button"
			class="cell"
			class:on={cellOn(i)}
			class:playing={currentIndex === i}
			class:beat-divider={isBeatDivider(i)}
			class:bar-divider={isBarDivider(i)}
			class:clickable={!!onClick}
			tabindex={onClick ? 0 : -1}
			disabled={!onClick}
			onclick={() => onClick?.(i)}
			aria-pressed={cellOn(i)}
			aria-label={`Cell ${i + 1}`}
		></button>
	{/each}
</div>

<style>
	.beat-grid {
		display: flex;
		gap: 2px;
		align-items: stretch;
	}

	.cell {
		background: transparent;
		border: 1px solid var(--color-border);
		padding: 0;
		cursor: default;
	}

	.cell.clickable {
		cursor: pointer;
	}

	.cell:disabled {
		cursor: default;
	}

	/* === pip variant — TransportBar / HistoryStrip header === */
	.beat-grid.pip {
		gap: 3px;
	}
	.beat-grid.pip .cell {
		width: 8px;
		height: 8px;
		background: var(--color-border);
	}
	.beat-grid.pip .cell.playing {
		background: var(--color-accent-cyan);
		box-shadow: 0 0 4px var(--color-accent-cyan);
	}

	/* === cell variant — PatternPanel programmer === */
	.beat-grid.cell {
		flex-wrap: wrap;
		gap: 2px;
	}
	.beat-grid.cell .cell {
		flex: 1 1 0;
		min-width: 16px;
		height: 28px;
		transition: background-color 0.05s linear, border-color 0.05s linear;
	}
	.beat-grid.cell .cell.clickable:hover {
		border-color: var(--color-accent-cyan);
	}
	.beat-grid.cell .cell.on {
		background: var(--color-accent-teal);
		border-color: var(--color-accent-cyan);
	}
	.beat-grid.cell .cell.playing {
		box-shadow: 0 0 8px var(--color-accent-amber);
		border-color: var(--color-accent-amber);
	}
	.beat-grid.cell .cell.on.playing {
		background: var(--color-accent-amber);
	}
	.beat-grid.cell .cell.beat-divider {
		margin-left: 4px;
	}
	.beat-grid.cell .cell.bar-divider {
		margin-left: 8px;
	}

	/* === mini variant — HistoryStrip pattern preview === */
	.beat-grid.mini {
		gap: 1px;
		flex: 1;
		min-width: 0;
		max-width: 600px;
	}
	.beat-grid.mini .cell {
		flex: 1 1 0;
		min-width: 4px;
		height: 8px;
		background: rgba(0, 0, 0, 0.4);
	}
	.beat-grid.mini .cell.on {
		background: var(--color-accent-teal);
	}
	.beat-grid.mini .cell.playing {
		box-shadow: 0 0 4px var(--color-accent-amber);
		border-color: var(--color-accent-amber);
	}
</style>
