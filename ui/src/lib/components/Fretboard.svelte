<script lang="ts">
	/**
	 * Fretboard — mirrors the interactive fretboard on the redesigned
	 * contrapunk.com. Cell-based highlights (not circle markers):
	 *
	 *   - Click any fret cell → plays note + harmony via the real engine.
	 *   - Every (string, fret) whose MIDI is in engine.inputNotes fills teal.
	 *   - Every (string, fret) whose MIDI is in engine.harmonyNotes fills magenta.
	 *   - Borrowed notes fill violet.
	 *   - Active cells bloom in on arrival via a single CSS animation.
	 *   - Note name centered inside the active cell (toggle via ui.showNoteLabels).
	 */
	import { engine } from '$lib/stores/engine.svelte';
	import { adapter } from '$lib/adapter';
	import { ui } from '$lib/stores/ui.svelte';

	// Standard tuning open string MIDI values [E2, A2, D3, G3, B3, E4]
	const STANDARD_TUNING = [40, 45, 50, 55, 59, 64];
	const STRING_LABELS = ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'];

	const NUM_FRETS = 24;
	const FRET_MARKERS = [3, 5, 7, 9, 12, 15, 17, 19, 21, 24];
	const DOUBLE_MARKERS = [12, 24];

	const SVG_WIDTH = 1040;
	const SVG_HEIGHT = 150;
	const NUT_X = 42;
	const FRET_AREA_WIDTH = SVG_WIDTH - NUT_X - 12;
	const STRING_Y_START = 20;
	const STRING_SPACING = 20;

	// Palette — matches contrapunk.com
	const COLOR_INPUT = '#4fe8c3';
	const COLOR_HARMONY = '#ff2e88';
	const COLOR_BORROWED = '#8a5cff';

	function fretX(fret: number): number {
		if (fret === 0) return NUT_X;
		return NUT_X + (fret / NUM_FRETS) * FRET_AREA_WIDTH;
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

	function midiName(m: number): string {
		const names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
		return names[((m % 12) + 12) % 12] + (Math.floor(m / 12) - 1);
	}

	/** Classify a midi note against the current engine state.
	 *  Input wins over harmony wins over borrowed (matches Piano priority). */
	function cellState(midi: number): 'input' | 'harmony' | 'borrowed' | null {
		if (engine.inputNotes.includes(midi)) return 'input';
		if (engine.harmonyNotes.includes(midi)) return 'harmony';
		if (engine.borrowedNotes.includes(midi)) return 'borrowed';
		return null;
	}

	function fillFor(state: 'input' | 'harmony' | 'borrowed'): string {
		return state === 'input' ? COLOR_INPUT : state === 'harmony' ? COLOR_HARMONY : COLOR_BORROWED;
	}

	// =========================================================
	// Click-to-play
	// =========================================================

	const pressedByPointer = new Map<number, { string: number; fret: number; midi: number }>();

	function handleFretDown(s: number, f: number, e: PointerEvent) {
		const midi = STANDARD_TUNING[s] + f;
		const target = e.currentTarget as SVGElement;
		try {
			target.setPointerCapture(e.pointerId);
		} catch {
			// pointer capture can throw on some touch devices — safe to ignore
		}
		pressedByPointer.set(e.pointerId, { string: s, fret: f, midi });
		adapter.injectNoteOn(midi, 100);
	}

	function handleFretUp(e: PointerEvent) {
		const entry = pressedByPointer.get(e.pointerId);
		if (!entry) return;
		adapter.injectNoteOff(entry.midi);
		pressedByPointer.delete(e.pointerId);
	}
</script>

<div class="fretboard-wrapper">
	<svg
		viewBox="0 0 {SVG_WIDTH} {SVG_HEIGHT}"
		class="fretboard-svg"
		xmlns="http://www.w3.org/2000/svg"
	>
		<!-- Background + wood -->
		<rect x="0" y="0" width={SVG_WIDTH} height={SVG_HEIGHT} fill="#0f0e1a" rx="2" />
		<rect x={NUT_X} y="12" width={FRET_AREA_WIDTH} height={STRING_SPACING * 5 + 18} fill="#1a1520" rx="1" />

		<!-- Nut -->
		<rect x={NUT_X - 3} y="12" width="4" height={STRING_SPACING * 5 + 18} fill="#6a5b86" />

		<!-- Fret markers (dots) -->
		{#each FRET_MARKERS as fret}
			{@const cx = fretCenterX(fret)}
			{@const cy = STRING_Y_START + 2.5 * STRING_SPACING}
			{#if DOUBLE_MARKERS.includes(fret)}
				<circle cx={cx} cy={cy - 14} r="3.2" fill="#3a2580" />
				<circle cx={cx} cy={cy + 14} r="3.2" fill="#3a2580" />
			{:else}
				<circle cx={cx} cy={cy} r="3.2" fill="#3a2580" />
			{/if}
		{/each}

		<!-- Fret lines -->
		{#each Array.from({ length: NUM_FRETS }, (_, i) => i + 1) as fret}
			<line
				x1={fretX(fret)}
				y1="12"
				x2={fretX(fret)}
				y2={STRING_Y_START + 5 * STRING_SPACING + 10}
				stroke="#2a1648"
				stroke-width="1"
			/>
		{/each}

		<!-- Strings -->
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

		<!-- String labels -->
		{#each STRING_LABELS as label, i}
			<text
				x="6"
				y={stringY(i) + 3}
				fill="#6a5b86"
				font-size="9"
				font-family="var(--font-code)"
			>
				{label}
			</text>
		{/each}

		<!-- Fret numbers -->
		{#each FRET_MARKERS as fret}
			<text
				x={fretCenterX(fret)}
				y={SVG_HEIGHT - 5}
				fill="#6a5b86"
				font-size="8"
				font-family="var(--font-code)"
				text-anchor="middle"
			>
				{fret}
			</text>
		{/each}

		<!-- Cells: one per (string, fret). Visually invisible by default;
		     highlights + labels render only when the cell's midi matches the
		     engine's current input/harmony/borrowed state. 6 × 25 = 150. -->
		{#each STANDARD_TUNING as _, s}
			{#each Array.from({ length: NUM_FRETS + 1 }, (_, f) => f) as f}
				{@const midi = STANDARD_TUNING[s] + f}
				{@const cx = fretCenterX(f)}
				{@const w = fretCellWidth(f)}
				{@const y = stringY(s)}
				{@const state = cellState(midi)}
				<g class="fret-cell">
					<rect
						class="fret-hit"
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
						aria-label="string {s + 1} fret {f}"
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
						{#if ui.showNoteLabels}
							<text
								class="cell-label"
								x={cx}
								y={y + 2.5}
								text-anchor="middle"
								fill={state === 'input' ? '#0a0612' : '#f5e9c9'}
							>
								{midiName(midi)}
							</text>
						{/if}
					{/if}
				</g>
			{/each}
		{/each}
	</svg>
</div>

<style>
	.fretboard-wrapper {
		width: 100%;
		padding: 0 0 2px 0;
	}

	.fretboard-svg {
		width: 100%;
		height: auto;
		display: block;
		touch-action: none;
	}

	.fret-hit {
		cursor: pointer;
		-webkit-tap-highlight-color: transparent;
	}

	/* Simple fade-in — matches contrapunk.com's Fretboard.
	   No scale transform, no "dragging" motion — the cell just lights up
	   in place with a quick bloom of opacity + glow. */
	.cell-fill {
		animation: fret-fade-in 160ms ease-out;
		filter: drop-shadow(0 0 6px currentColor);
		pointer-events: none;
		transform-box: fill-box;
		transform-origin: center;
	}
	.cell-fill.input { color: #4fe8c3; }
	.cell-fill.harmony { color: #ff2e88; }
	.cell-fill.borrowed { color: #8a5cff; }

	.cell-label {
		font-family: var(--font-code);
		font-size: 8.5px;
		font-weight: 600;
		letter-spacing: 0;
		pointer-events: none;
		animation: fret-fade-in 160ms ease-out;
	}

	@keyframes fret-fade-in {
		0%   { opacity: 0; }
		100% { opacity: 1; }
	}

	@media (prefers-reduced-motion: reduce) {
		.cell-fill, .cell-label { animation: none; }
	}
</style>
