<script lang="ts">
	let {
		grid = [],
		counts = undefined,
		onCellClick = undefined,
	}: {
		grid: number[][];
		counts?: number[][];
		onCellClick?: (stringIdx: number, fret: number) => void;
	} = $props();

	const stringLabels = ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'];

	let tooltip = $state<{ x: number; y: number; text: string } | null>(null);

	function cellColor(v: number): string {
		// red -> amber -> cyan -> green based on accuracy
		if (v >= 0.95) return '#00ccaa';
		if (v >= 0.90) return '#33ddff';
		if (v >= 0.85) return '#ffaa33';
		if (v >= 0.70) return '#cc6622';
		if (v >= 0.50) return '#ff3388';
		return '#881144';
	}

	function cellOpacity(v: number): number {
		// Scale opacity so lower accuracy cells are dimmer
		return 0.4 + v * 0.6;
	}

	// Layout constants
	const labelW = 30;
	const headerH = 20;
	const cellW = 24;
	const cellH = 22;
	const gap = 1;

	function totalW(): number {
		const numFrets = grid.length > 0 ? grid[0].length : 23;
		return labelW + numFrets * (cellW + gap);
	}

	function totalH(): number {
		return headerH + grid.length * (cellH + gap);
	}

	function handleMouseEnter(sIdx: number, fret: number, event: MouseEvent) {
		const v = grid[sIdx][fret];
		const count = counts?.[sIdx]?.[fret];
		let text = `${stringLabels[sIdx]} fret ${fret}: ${(v * 100).toFixed(0)}%`;
		if (count !== undefined) text += ` (${count} samples)`;

		const target = event.currentTarget as SVGElement;
		const svg = target.closest('svg');
		if (!svg) return;
		const rect = svg.getBoundingClientRect();
		const eRect = target.getBoundingClientRect();
		tooltip = {
			x: eRect.left - rect.left + cellW / 2,
			y: eRect.top - rect.top - 6,
			text,
		};
	}

	function handleMouseLeave() {
		tooltip = null;
	}

	function handleClick(sIdx: number, fret: number) {
		onCellClick?.(sIdx, fret);
	}
</script>

<div class="heatmap-wrapper" style="position: relative;">
	<svg
		viewBox="0 0 {totalW()} {totalH()}"
		width="100%"
		style="display: block;"
		role="img"
		aria-label="Fret accuracy heatmap showing accuracy per string and fret"
	>
		<!-- Fret number headers -->
		{#each Array(grid[0]?.length ?? 23) as _, f}
			<text
				x={labelW + f * (cellW + gap) + cellW / 2}
				y={headerH - 5}
				fill="#555577"
				font-size="8"
				font-family="system-ui, sans-serif"
				text-anchor="middle"
			>{f}</text>
		{/each}

		<!-- String labels + cells -->
		{#each grid as row, sIdx}
			<!-- String label -->
			<text
				x={labelW - 5}
				y={headerH + sIdx * (cellH + gap) + cellH / 2}
				fill="#e8e0f0"
				font-size="10"
				font-family="system-ui, sans-serif"
				text-anchor="end"
				dominant-baseline="central"
			>{stringLabels[sIdx]}</text>

			{#each row as value, fret}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<rect
					x={labelW + fret * (cellW + gap)}
					y={headerH + sIdx * (cellH + gap)}
					width={cellW}
					height={cellH}
					rx="2"
					fill={cellColor(value)}
					opacity={cellOpacity(value)}
					style="cursor: {onCellClick ? 'pointer' : 'default'};"
					onmouseenter={(e) => handleMouseEnter(sIdx, fret, e)}
					onmouseleave={handleMouseLeave}
					onclick={() => handleClick(sIdx, fret)}
				/>
			{/each}
		{/each}
	</svg>

	<!-- Tooltip overlay -->
	{#if tooltip}
		<div
			class="heatmap-tooltip"
			style="left: {tooltip.x}px; top: {tooltip.y}px;"
		>
			{tooltip.text}
		</div>
	{/if}
</div>

<style>
	.heatmap-wrapper {
		overflow-x: auto;
	}

	.heatmap-tooltip {
		position: absolute;
		transform: translate(-50%, -100%);
		background: #151428;
		border: 1px solid #2a2848;
		color: #e8e0f0;
		font-size: 11px;
		font-family: system-ui, sans-serif;
		padding: 4px 8px;
		white-space: nowrap;
		pointer-events: none;
		z-index: 10;
	}
</style>
