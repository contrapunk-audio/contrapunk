<script lang="ts">
	/**
	 * Fretboard — canonical embed component. State-library-agnostic.
	 *
	 * Data comes in via props; click events go out via callbacks.
	 * Imports zero stores so the same file works in the contrapunk
	 * app (wrapped with engine.svelte) and the website embed (wrapped
	 * with nanostores). Per contrapunk-audio/contrapunk#66.
	 *
	 * Mirror-copied to website/src/lib/contrapunk/embed/.
	 */
	import {
		COLOR_INPUT,
		COLOR_HARMONY,
		COLOR_BORROWED,
		STANDARD_TUNING,
		STANDARD_TUNING_LABELS,
		midiToName
	} from './music-utils';

	let {
		inputNotes,
		harmonyNotes,
		borrowedNotes = [],
		onNoteOn,
		onNoteOff,
		frets = 24,
		interactive = true,
		showLabels = true
	}: {
		inputNotes: number[];
		harmonyNotes: number[];
		borrowedNotes?: number[];
		onNoteOn?: (midi: number) => void;
		onNoteOff?: (midi: number) => void;
		frets?: number;
		interactive?: boolean;
		showLabels?: boolean;
	} = $props();

	const FRET_MARKERS = [3, 5, 7, 9, 12, 15, 17, 19, 21, 24];
	const DOUBLE_MARKERS = [12, 24];

	const SVG_WIDTH = 1040;
	const SVG_HEIGHT = 150;
	const NUT_X = 42;
	const FRET_AREA_WIDTH = SVG_WIDTH - NUT_X - 12;
	const STRING_Y_START = 20;
	const STRING_SPACING = 20;

	function fretX(fret: number): number {
		if (fret === 0) return NUT_X;
		return NUT_X + (fret / frets) * FRET_AREA_WIDTH;
	}
	function fretCenterX(fret: number): number {
		if (fret === 0) return NUT_X - 10;
		return (fretX(fret - 1) + fretX(fret)) / 2;
	}
	function fretCellWidth(fret: number): number {
		if (fret === 0) return 18;
		return fretX(fret) - fretX(fret - 1);
	}
	function stringY(stringIdx: number): number {
		return STRING_Y_START + (5 - stringIdx) * STRING_SPACING;
	}

	function cellState(midi: number): 'input' | 'harmony' | 'borrowed' | null {
		if (inputNotes.includes(midi)) return 'input';
		if (harmonyNotes.includes(midi)) return 'harmony';
		if (borrowedNotes.includes(midi)) return 'borrowed';
		return null;
	}
	function fillFor(state: 'input' | 'harmony' | 'borrowed'): string {
		return state === 'input' ? COLOR_INPUT : state === 'harmony' ? COLOR_HARMONY : COLOR_BORROWED;
	}

	const pressedByPointer = new Map<number, { string: number; fret: number; midi: number }>();

	function handleFretDown(s: number, f: number, e: PointerEvent) {
		if (!interactive) return;
		const midi = STANDARD_TUNING[s] + f;
		const target = e.currentTarget as SVGElement;
		try {
			target.setPointerCapture(e.pointerId);
		} catch {
			// pointer capture can throw on some touch devices — safe to ignore
		}
		pressedByPointer.set(e.pointerId, { string: s, fret: f, midi });
		onNoteOn?.(midi);
	}
	function handleFretUp(e: PointerEvent) {
		if (!interactive) return;
		const entry = pressedByPointer.get(e.pointerId);
		if (!entry) return;
		onNoteOff?.(entry.midi);
		pressedByPointer.delete(e.pointerId);
	}

	const fretIndices = $derived(Array.from({ length: frets + 1 }, (_, i) => i));
	const fretLineIndices = $derived(Array.from({ length: frets }, (_, i) => i + 1));
	const visibleMarkers = $derived(FRET_MARKERS.filter((m) => m <= frets));
</script>

<div class="fretboard-wrapper">
	<svg
		viewBox="0 0 {SVG_WIDTH} {SVG_HEIGHT}"
		class="fretboard-svg"
		xmlns="http://www.w3.org/2000/svg"
	>
		<rect x="0" y="0" width={SVG_WIDTH} height={SVG_HEIGHT} fill="#0f0e1a" rx="2" />
		<rect
			x={NUT_X}
			y="12"
			width={FRET_AREA_WIDTH}
			height={STRING_SPACING * 5 + 18}
			fill="#1a1520"
			rx="1"
		/>
		<rect x={NUT_X - 3} y="12" width="4" height={STRING_SPACING * 5 + 18} fill="#6a5b86" />

		{#each visibleMarkers as fret}
			{@const cx = fretCenterX(fret)}
			{@const cy = STRING_Y_START + 2.5 * STRING_SPACING}
			{#if DOUBLE_MARKERS.includes(fret)}
				<circle cx={cx} cy={cy - 14} r="3.2" fill="#3a2580" />
				<circle cx={cx} cy={cy + 14} r="3.2" fill="#3a2580" />
			{:else}
				<circle cx={cx} cy={cy} r="3.2" fill="#3a2580" />
			{/if}
		{/each}

		{#each fretLineIndices as fret}
			<line
				x1={fretX(fret)}
				y1="12"
				x2={fretX(fret)}
				y2={STRING_Y_START + 5 * STRING_SPACING + 10}
				stroke="#2a1648"
				stroke-width="1"
			/>
		{/each}

		{#each STANDARD_TUNING as _, i}
			{@const y = stringY(i)}
			<line
				x1={NUT_X}
				y1={y}
				x2={SVG_WIDTH - 12}
				y2={y}
				stroke="#b79cea"
				stroke-opacity="0.6"
				stroke-width={1.6 - i * 0.15}
			/>
		{/each}

		{#each STANDARD_TUNING_LABELS as label, i}
			<text x="6" y={stringY(i) + 3} fill="#6a5b86" font-size="9" font-family="var(--font-code, ui-monospace)">
				{label}
			</text>
		{/each}

		{#each visibleMarkers as fret}
			<text
				x={fretCenterX(fret)}
				y={SVG_HEIGHT - 5}
				fill="#6a5b86"
				font-size="8"
				font-family="var(--font-code, ui-monospace)"
				text-anchor="middle"
			>
				{fret}
			</text>
		{/each}

		{#each STANDARD_TUNING as _, s}
			{#each fretIndices as f}
				{@const midi = STANDARD_TUNING[s] + f}
				{@const cx = fretCenterX(f)}
				{@const w = fretCellWidth(f)}
				{@const y = stringY(s)}
				{@const state = cellState(midi)}
				<g class="fret-cell">
					<rect
						class="fret-hit"
						class:interactive
						x={cx - w / 2}
						y={y - STRING_SPACING / 2}
						width={w}
						height={STRING_SPACING}
						fill="transparent"
						onpointerdown={(e) => handleFretDown(s, f, e)}
						onpointerup={handleFretUp}
						onpointercancel={handleFretUp}
						onpointerleave={handleFretUp}
						role="button"
						tabindex="-1"
						aria-label={`string ${s + 1} fret ${f}`}
					/>
					{#if state}
						<rect
							class="cell-fill {state}"
							x={cx - w / 2 + 1}
							y={y - STRING_SPACING / 2 + 1}
							width={w - 2}
							height={STRING_SPACING - 2}
							fill={fillFor(state)}
							rx="1"
						/>
						{#if showLabels}
							<text
								class="cell-label"
								x={cx}
								y={y + 2.5}
								text-anchor="middle"
								fill={state === 'input' ? '#0a0612' : '#f5e9c9'}
							>
								{midiToName(midi)}
							</text>
						{/if}
					{/if}
				</g>
			{/each}
		{/each}
	</svg>
</div>

<style>
	.fretboard-wrapper { width: 100%; padding: 0 0 2px 0; }
	.fretboard-svg { width: 100%; height: auto; display: block; touch-action: none; }
	.fret-hit { -webkit-tap-highlight-color: transparent; }
	.fret-hit.interactive { cursor: pointer; }

	.cell-fill {
		animation: fret-glitch-in 220ms steps(6, end);
		filter: drop-shadow(0 0 6px currentColor);
		pointer-events: none;
		transform-box: fill-box;
		transform-origin: center;
	}
	.cell-fill.input { color: #4fe8c3; }
	.cell-fill.harmony { color: #ff2e88; }
	.cell-fill.borrowed { color: #8a5cff; }

	.cell-label {
		font-family: var(--font-code, ui-monospace, monospace);
		font-size: 8.5px;
		font-weight: 600;
		letter-spacing: 0;
		pointer-events: none;
		animation: fret-glitch-in 220ms steps(6, end);
	}

	@keyframes fret-glitch-in {
		0%   { opacity: 1; transform: translateX(-4px) scaleX(1.2); filter: brightness(2.4) hue-rotate(24deg) drop-shadow(0 0 10px currentColor); }
		20%  { opacity: 1; transform: translateX( 3px) scaleX(0.9); filter: brightness(1.8) hue-rotate(-18deg) drop-shadow(0 0 8px currentColor); }
		40%  { opacity: 1; transform: translateX(-2px) scaleX(1.1); filter: brightness(1.5) hue-rotate(10deg) drop-shadow(0 0 6px currentColor); }
		60%  { opacity: 0.85; transform: translateX( 1px) scaleX(1);   filter: brightness(1.2) hue-rotate(-4deg) drop-shadow(0 0 6px currentColor); }
		80%  { opacity: 1; transform: translateX( 0)     scaleX(1);   filter: brightness(1.1) drop-shadow(0 0 6px currentColor); }
		100% { opacity: 1; transform: translateX( 0)     scaleX(1);   filter: brightness(1) drop-shadow(0 0 6px currentColor); }
	}

	@media (prefers-reduced-motion: reduce) {
		.cell-fill, .cell-label { animation: none; }
	}
</style>
