<script lang="ts">
	/**
	 * Serum-style rotary knob.
	 *
	 * Drag vertically to change the value. Holding Shift gives a finer
	 * drag (10× smaller ratio). Double-click resets to `defaultValue`
	 * if provided. Scroll wheel fine-steps by `step`.
	 *
	 * Internally, the knob sweeps 270° from 7:30 to 4:30 clockwise,
	 * with an amber/gold arc tracking the value and a bright radial
	 * indicator line on the inner disc.
	 */

	let {
		value,
		min = 0,
		max = 1,
		step = 0.01,
		defaultValue,
		label = '',
		format,
		size = 72,
		accent = 'var(--color-accent-cyan)',
		onchange
	}: {
		value: number;
		min?: number;
		max?: number;
		step?: number;
		defaultValue?: number;
		label?: string;
		format?: (v: number) => string;
		size?: number;
		accent?: string;
		onchange: (value: number) => void;
	} = $props();

	const ARC_START = -135; // degrees from top (clockwise); 7:30 position
	const ARC_RANGE = 270; // sweep

	let dragging = $state(false);
	let hovering = $state(false);
	let startY = 0;
	let startValue = 0;

	let percent = $derived(Math.max(0, Math.min(1, (value - min) / (max - min))));
	let currentAngle = $derived(ARC_START + percent * ARC_RANGE);

	function polar(rDeg: number, radius: number): { x: number; y: number } {
		const rad = ((rDeg - 90) * Math.PI) / 180;
		return {
			x: 50 + radius * Math.cos(rad),
			y: 50 + radius * Math.sin(rad)
		};
	}

	let arcPath = $derived(buildArcPath(ARC_START, currentAngle, 38));

	function buildArcPath(startDeg: number, endDeg: number, r: number): string {
		const a = polar(startDeg, r);
		const b = polar(endDeg, r);
		const sweepDeg = endDeg - startDeg;
		const large = sweepDeg > 180 ? 1 : 0;
		return `M ${a.x} ${a.y} A ${r} ${r} 0 ${large} 1 ${b.x} ${b.y}`;
	}

	let tick = $derived(polar(currentAngle, 28));

	function displayValue(v: number): string {
		if (format) return format(v);
		if (Number.isInteger(v) || max - min > 20) return Math.round(v).toString();
		return v.toFixed(2);
	}

	function clamp(v: number): number {
		return Math.max(min, Math.min(max, v));
	}

	function snap(v: number): number {
		if (step <= 0) return v;
		return Math.round((v - min) / step) * step + min;
	}

	function onPointerDown(e: PointerEvent) {
		(e.currentTarget as Element).setPointerCapture(e.pointerId);
		dragging = true;
		startY = e.clientY;
		startValue = value;
		e.preventDefault();
	}

	function onPointerMove(e: PointerEvent) {
		if (!dragging) return;
		const sensitivity = e.shiftKey ? 1500 : 200; // px for full range
		const deltaPx = startY - e.clientY;
		const deltaVal = (deltaPx / sensitivity) * (max - min);
		onchange(clamp(snap(startValue + deltaVal)));
	}

	function onPointerUp(e: PointerEvent) {
		dragging = false;
		(e.currentTarget as Element).releasePointerCapture(e.pointerId);
	}

	function onWheel(e: WheelEvent) {
		e.preventDefault();
		const dir = e.deltaY < 0 ? 1 : -1;
		const mul = e.shiftKey ? 0.1 : 1;
		onchange(clamp(snap(value + dir * step * mul)));
	}

	function onDoubleClick() {
		if (defaultValue !== undefined) {
			onchange(clamp(snap(defaultValue)));
		}
	}
</script>

<div class="knob" style:width="{size}px">
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div
		class="knob-dial"
		class:dragging
		class:hovering
		style:width="{size}px"
		style:height="{size}px"
		onpointerdown={onPointerDown}
		onpointermove={onPointerMove}
		onpointerup={onPointerUp}
		onpointercancel={onPointerUp}
		onpointerenter={() => (hovering = true)}
		onpointerleave={() => (hovering = false)}
		onwheel={onWheel}
		ondblclick={onDoubleClick}
		role="slider"
		tabindex="0"
		aria-valuemin={min}
		aria-valuemax={max}
		aria-valuenow={value}
		aria-label={label}
	>
		{#if (hovering || dragging) && label}
			<div class="knob-tooltip font-ui">
				{label}<span class="tooltip-sep">:</span><span class="tooltip-val">{displayValue(value)}</span>
			</div>
		{/if}
		<svg viewBox="0 0 100 100" style:display="block">
			<!-- Track ring -->
			<circle cx="50" cy="50" r="38" fill="none" stroke="#2a2848" stroke-width="6" />
			<!-- Active arc -->
			<path d={arcPath} fill="none" stroke={accent} stroke-width="6" stroke-linecap="round" />
			<!-- Inner disc with radial gradient -->
			<defs>
				<radialGradient id="knob-disc" cx="50%" cy="40%" r="60%">
					<stop offset="0%" stop-color="#25223d" />
					<stop offset="100%" stop-color="#0f0e1a" />
				</radialGradient>
			</defs>
			<circle cx="50" cy="50" r="28" fill="url(#knob-disc)" stroke="#11101c" stroke-width="1" />
			<!-- Indicator line -->
			<line
				x1="50"
				y1="50"
				x2={tick.x}
				y2={tick.y}
				stroke={accent}
				stroke-width="3"
				stroke-linecap="round"
			/>
			<!-- Tick dots at 0 / max -->
			{#each [ARC_START, ARC_START + ARC_RANGE] as deg}
				{@const p = polar(deg, 44)}
				<circle cx={p.x} cy={p.y} r="1.2" fill="#555577" />
			{/each}
		</svg>
		<div class="knob-value font-code" class:dragging>{displayValue(value)}</div>
	</div>

	{#if label}
		<div class="knob-label font-ui">{label}</div>
	{/if}
</div>

<style>
	.knob {
		display: inline-flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		user-select: none;
	}

	.knob-dial {
		position: relative;
		cursor: ns-resize;
		filter: drop-shadow(0 0 6px rgba(77, 221, 221, 0.2));
		transition: filter 120ms;
	}
	.knob-dial:hover,
	.knob-dial.dragging {
		filter: drop-shadow(0 0 10px rgba(77, 221, 221, 0.5));
	}
	.knob-dial:focus {
		outline: none;
	}

	.knob-value {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		font-size: var(--font-size-xs);
		color: var(--color-accent-cyan);
		pointer-events: none;
		text-align: center;
		line-height: 1;
		white-space: nowrap;
	}
	.knob-value.dragging {
		color: #fff;
		text-shadow: 0 0 4px var(--color-accent-cyan);
	}

	.knob-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		letter-spacing: 1px;
		text-transform: uppercase;
	}

	/* Serum-style hover tooltip — dark bubble above the knob with
	   "Label : Value" while hovering or dragging. Positioned above
	   the dial so the cursor doesn't occlude it on drag. */
	.knob-tooltip {
		position: absolute;
		bottom: 100%;
		left: 50%;
		transform: translate(-50%, -4px);
		padding: 3px 8px;
		font-size: var(--font-size-xs);
		background: rgba(15, 14, 26, 0.95);
		border: 1px solid var(--color-accent-cyan);
		color: var(--color-text);
		white-space: nowrap;
		pointer-events: none;
		z-index: 10;
		box-shadow: 0 0 8px rgba(77, 221, 221, 0.3);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}
	.tooltip-sep {
		color: var(--color-text-dim);
		margin: 0 4px;
	}
	.tooltip-val {
		color: var(--color-accent-cyan);
	}
</style>
