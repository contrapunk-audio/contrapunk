<script lang="ts">
	let {
		data = [],
		labels = ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'],
		height = 200,
		animated = true,
	}: {
		data: number[];
		labels?: string[];
		height?: number;
		animated?: boolean;
	} = $props();

	let mounted = $state(false);

	$effect(() => {
		if (animated) {
			// Trigger animation after mount
			requestAnimationFrame(() => { mounted = true; });
		} else {
			mounted = true;
		}
	});

	function barColor(v: number): string {
		if (v >= 0.95) return '#00ccaa';
		if (v >= 0.90) return '#33ddff';
		if (v >= 0.85) return '#ffaa33';
		return '#ff3388';
	}

	function pct(v: number): string {
		return (v * 100).toFixed(1) + '%';
	}

	const labelWidth = 40;
	const valueWidth = 52;
	const padTop = 4;
	const padBottom = 4;
</script>

<svg
	viewBox="0 0 400 {height}"
	width="100%"
	style="display: block; max-width: 600px;"
	role="img"
	aria-label="Per-string accuracy bar chart"
>
	{#each data as value, i}
		{@const rowH = (height - padTop - padBottom) / data.length}
		{@const y = padTop + i * rowH}
		{@const barAreaW = 400 - labelWidth - valueWidth}
		{@const barW = mounted ? value * barAreaW : 0}
		{@const barH = rowH * 0.6}
		{@const barY = y + (rowH - barH) / 2}
		<!-- String label -->
		<text
			x={labelWidth - 6}
			y={y + rowH / 2}
			fill="#e8e0f0"
			font-size="12"
			font-family="system-ui, sans-serif"
			text-anchor="end"
			dominant-baseline="central"
		>{labels[i]}</text>

		<!-- Track background -->
		<rect
			x={labelWidth}
			y={barY}
			width={barAreaW}
			height={barH}
			fill="#0a0a1a"
			stroke="#2a2848"
			stroke-width="1"
		/>

		<!-- Filled bar -->
		<rect
			x={labelWidth}
			y={barY}
			width={barW}
			height={barH}
			fill={barColor(value)}
			opacity="0.85"
			style="transition: width 0.6s ease-out;"
		/>

		<!-- Percentage label -->
		<text
			x={400 - 4}
			y={y + rowH / 2}
			fill={barColor(value)}
			font-size="11"
			font-family="system-ui, sans-serif"
			text-anchor="end"
			dominant-baseline="central"
		>{pct(value)}</text>
	{/each}
</svg>
