<script lang="ts">
	let {
		matrix = [],
	}: {
		matrix: number[][];
	} = $props();

	const stringLabels = ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'];

	let tooltip = $state<{ x: number; y: number; text: string } | null>(null);

	// Find max off-diagonal value for scaling
	function maxOffDiag(): number {
		let max = 1;
		for (let r = 0; r < matrix.length; r++) {
			for (let c = 0; c < matrix[r].length; c++) {
				if (r !== c && matrix[r][c] > max) max = matrix[r][c];
			}
		}
		return max;
	}

	function cellColor(row: number, col: number, value: number): string {
		if (row === col) {
			// Diagonal = correct predictions, teal
			return '#00ccaa';
		}
		// Off-diagonal = errors, magenta intensity scaled by count
		if (value === 0) return '#0a0a1a';
		const intensity = Math.min(1, value / maxOffDiag());
		const r = Math.round(10 + intensity * 245);   // 10..255
		const g = Math.round(10 + intensity * 41);     // 10..51
		const b = Math.round(26 + intensity * 110);    // 26..136
		return `rgb(${r},${g},${b})`;
	}

	function cellOpacity(row: number, col: number, value: number): number {
		if (row === col) {
			// Diagonal: brighter for larger counts
			const maxDiag = Math.max(...matrix.map((r, i) => r[i]));
			return 0.4 + 0.6 * (maxDiag > 0 ? value / maxDiag : 0);
		}
		if (value === 0) return 0.15;
		return 0.3 + 0.7 * Math.min(1, value / maxOffDiag());
	}

	// Layout
	const labelW = 30;
	const headerH = 26;
	const cellSize = 44;
	const gap = 2;
	const w = labelW + 6 * (cellSize + gap);
	const h = headerH + 6 * (cellSize + gap);

	function handleMouseEnter(row: number, col: number, event: MouseEvent) {
		const value = matrix[row][col];
		const text = row === col
			? `${stringLabels[row]} correct: ${value}`
			: `${stringLabels[row]} predicted as ${stringLabels[col]}: ${value}`;

		const target = event.currentTarget as SVGElement;
		const svg = target.closest('svg');
		if (!svg) return;
		const rect = svg.getBoundingClientRect();
		const eRect = target.getBoundingClientRect();
		tooltip = {
			x: eRect.left - rect.left + cellSize / 2,
			y: eRect.top - rect.top - 6,
			text,
		};
	}

	function handleMouseLeave() {
		tooltip = null;
	}
</script>

<div class="confusion-wrapper" style="position: relative;">
	<svg
		viewBox="0 0 {w} {h}"
		width="100%"
		style="display: block; max-width: 400px;"
		role="img"
		aria-label="String confusion matrix showing which strings get confused"
	>
		<!-- Header label -->
		<text
			x={labelW + 3 * (cellSize + gap)}
			y={10}
			fill="#555577"
			font-size="9"
			font-family="system-ui, sans-serif"
			text-anchor="middle"
		>Predicted</text>

		<!-- Column headers -->
		{#each stringLabels as label, c}
			<text
				x={labelW + c * (cellSize + gap) + cellSize / 2}
				y={headerH - 4}
				fill="#e8e0f0"
				font-size="10"
				font-family="system-ui, sans-serif"
				text-anchor="middle"
			>{label}</text>
		{/each}

		<!-- Rows -->
		{#each matrix as row, r}
			<!-- Row label -->
			<text
				x={labelW - 5}
				y={headerH + r * (cellSize + gap) + cellSize / 2}
				fill="#e8e0f0"
				font-size="10"
				font-family="system-ui, sans-serif"
				text-anchor="end"
				dominant-baseline="central"
			>{stringLabels[r]}</text>

			{#each row as value, c}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<rect
					x={labelW + c * (cellSize + gap)}
					y={headerH + r * (cellSize + gap)}
					width={cellSize}
					height={cellSize}
					rx="3"
					fill={cellColor(r, c, value)}
					opacity={cellOpacity(r, c, value)}
					onmouseenter={(e) => handleMouseEnter(r, c, e)}
					onmouseleave={handleMouseLeave}
				/>

				<!-- Count number inside cell -->
				<text
					x={labelW + c * (cellSize + gap) + cellSize / 2}
					y={headerH + r * (cellSize + gap) + cellSize / 2}
					fill={value === 0 ? '#333355' : '#e8e0f0'}
					font-size={r === c ? '12' : '10'}
					font-weight={r === c ? '700' : '400'}
					font-family="system-ui, sans-serif"
					text-anchor="middle"
					dominant-baseline="central"
					pointer-events="none"
				>{value}</text>
			{/each}
		{/each}

		<!-- "Actual" label on left side, rotated -->
		<text
			x={8}
			y={headerH + 3 * (cellSize + gap)}
			fill="#555577"
			font-size="9"
			font-family="system-ui, sans-serif"
			text-anchor="middle"
			transform="rotate(-90, 8, {headerH + 3 * (cellSize + gap)})"
		>Actual</text>
	</svg>

	<!-- Tooltip overlay -->
	{#if tooltip}
		<div
			class="confusion-tooltip"
			style="left: {tooltip.x}px; top: {tooltip.y}px;"
		>
			{tooltip.text}
		</div>
	{/if}
</div>

<style>
	.confusion-wrapper {
		display: inline-block;
	}

	.confusion-tooltip {
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
