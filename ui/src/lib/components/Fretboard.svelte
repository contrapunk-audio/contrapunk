<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { adapter } from '$lib/adapter';

	// Standard tuning open string MIDI values [E2, A2, D3, G3, B3, E4]
	const STANDARD_TUNING = [40, 45, 50, 55, 59, 64];
	const STRING_LABELS = ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'];

	// Extended range: 24 frets to match a full guitar neck.
	const NUM_FRETS = 24;
	const FRET_MARKERS = [3, 5, 7, 9, 12, 15, 17, 19, 21, 24];
	const DOUBLE_MARKERS = [12, 24];

	// SVG dimensions — grow horizontally with fret count to keep cells readable.
	const SVG_WIDTH = 1040;
	const SVG_HEIGHT = 150;
	const NUT_X = 42;
	const FRET_AREA_WIDTH = SVG_WIDTH - NUT_X - 12;
	const STRING_Y_START = 20;
	const STRING_SPACING = 20;

	// Palette — matches the redesigned contrapunk.com.
	const COLOR_INPUT = '#4fe8c3';     // teal — melody in
	const COLOR_HARMONY = '#ff2e88';   // magenta — harmony out
	const COLOR_BORROWED = '#8a5cff';  // violet — borrowed / interchange
	const COLOR_PRESS = '#f5e9c9';     // cream — momentary press pulse

	/** X position for a fret. Fret 0 = nut. */
	function fretX(fret: number): number {
		if (fret === 0) return NUT_X;
		return NUT_X + (fret / NUM_FRETS) * FRET_AREA_WIDTH;
	}

	/** X center of a fret cell (between fret-1 and fret). */
	function fretCenterX(fret: number): number {
		if (fret === 0) return NUT_X - 10;
		return (fretX(fret - 1) + fretX(fret)) / 2;
	}

	/** Width of a single fret cell at this fret index. */
	function fretCellWidth(fret: number): number {
		if (fret === 0) return 18;
		return fretX(fret) - fretX(fret - 1);
	}

	/** Y position for a string (0 = low E at bottom, 5 = high E at top). */
	function stringY(stringIdx: number): number {
		return STRING_Y_START + (5 - stringIdx) * STRING_SPACING;
	}

	interface Pos { string: number; fret: number; x: number; y: number; }

	/** All (string, fret) positions on the board that produce the given MIDI note. */
	function midiToPositions(note: number): Pos[] {
		const out: Pos[] = [];
		for (let s = 0; s < 6; s++) {
			const fret = note - STANDARD_TUNING[s];
			if (fret >= 0 && fret <= NUM_FRETS) {
				out.push({ string: s, fret, x: fretCenterX(fret), y: stringY(s) });
			}
		}
		return out;
	}

	/** All positions for the current input / harmony / borrowed notes. */
	let inputMarkers = $derived(
		engine.inputNotes.flatMap((n) =>
			midiToPositions(n).map((p) => ({ ...p, midi: n }))
		)
	);
	let harmonyMarkers = $derived(
		engine.harmonyNotes.flatMap((n) =>
			midiToPositions(n).map((p) => ({ ...p, midi: n }))
		)
	);
	let borrowedMarkers = $derived(
		engine.borrowedNotes.flatMap((n) =>
			midiToPositions(n).map((p) => ({ ...p, midi: n }))
		)
	);

	// =========================================================
	// Click-to-play + press-flash
	// =========================================================

	const pressedByPointer = new Map<number, { string: number; fret: number; midi: number }>();
	let pressed = $state(new Set<string>()); // keys "s:f"

	function posKey(s: number, f: number): string { return `${s}:${f}`; }

	function handleFretDown(s: number, f: number, e: PointerEvent) {
		const midi = STANDARD_TUNING[s] + f;
		const target = e.currentTarget as SVGElement;
		try {
			target.setPointerCapture(e.pointerId);
		} catch {
			// pointer capture not supported — safe to ignore
		}
		pressedByPointer.set(e.pointerId, { string: s, fret: f, midi });
		pressed = new Set(pressed).add(posKey(s, f));
		adapter.injectNoteOn(midi, 100);
	}

	function handleFretUp(e: PointerEvent) {
		const entry = pressedByPointer.get(e.pointerId);
		if (!entry) return;
		adapter.injectNoteOff(entry.midi);
		pressedByPointer.delete(e.pointerId);
		const next = new Set(pressed);
		next.delete(posKey(entry.string, entry.fret));
		pressed = next;
	}

	// =========================================================
	// Flash on new note arrival (matches Piano.svelte treatment)
	// =========================================================

	let flashingInput = $state(new Set<number>());
	let flashingHarmony = $state(new Set<number>());
	let prevInput: Set<number> = new Set();
	let prevHarmony: Set<number> = new Set();

	function diffAndFlash(
		current: number[],
		prev: Set<number>,
		flashing: Set<number>,
		setFlashing: (s: Set<number>) => void
	): Set<number> {
		const curr = new Set(current);
		const newlyArrived: number[] = [];
		for (const m of curr) if (!prev.has(m)) newlyArrived.push(m);
		if (newlyArrived.length === 0) return curr;
		const next = new Set(flashing);
		for (const m of newlyArrived) next.add(m);
		setFlashing(next);
		for (const m of newlyArrived) {
			setTimeout(() => {
				// Read the freshest flashing set via the setter's closure
				setFlashing((() => {
					const later = new Set(flashing);
					later.delete(m);
					return later;
				})());
			}, 260);
		}
		return curr;
	}

	$effect(() => {
		prevInput = diffAndFlash(
			engine.inputNotes,
			prevInput,
			flashingInput,
			(s) => (flashingInput = s)
		);
	});

	$effect(() => {
		prevHarmony = diffAndFlash(
			engine.harmonyNotes,
			prevHarmony,
			flashingHarmony,
			(s) => (flashingHarmony = s)
		);
	});
</script>

<div class="fretboard-wrapper">
	<svg
		viewBox="0 0 {SVG_WIDTH} {SVG_HEIGHT}"
		class="fretboard-svg"
		xmlns="http://www.w3.org/2000/svg"
	>
		<!-- Background -->
		<rect x="0" y="0" width={SVG_WIDTH} height={SVG_HEIGHT} fill="#0f0e1a" rx="2" />

		<!-- Fretboard wood area -->
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

		<!-- Invisible click/touch targets per fret cell.
		     6 strings × 25 frets (0..24) = 150 hit targets. -->
		{#each STANDARD_TUNING as _, s}
			{#each Array.from({ length: NUM_FRETS + 1 }, (_, f) => f) as f}
				{@const cx = fretCenterX(f)}
				{@const w = fretCellWidth(f)}
				{@const y = stringY(s)}
				<rect
					class="fret-hit"
					class:pressed={pressed.has(posKey(s, f))}
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
			{/each}
		{/each}

		<!-- Harmony markers (magenta) -->
		{#each harmonyMarkers as marker}
			<g class="marker" class:flash={flashingHarmony.has(marker.midi)}>
				<circle cx={marker.x} cy={marker.y} r="7.5" fill={COLOR_HARMONY} fill-opacity="0.22" />
				<circle cx={marker.x} cy={marker.y} r="5" fill={COLOR_HARMONY} stroke={COLOR_HARMONY} stroke-width="1.5" />
			</g>
		{/each}

		<!-- Borrowed markers (violet) -->
		{#each borrowedMarkers as marker}
			<g class="marker">
				<circle cx={marker.x} cy={marker.y} r="7.5" fill={COLOR_BORROWED} fill-opacity="0.22" />
				<circle cx={marker.x} cy={marker.y} r="5" fill={COLOR_BORROWED} stroke={COLOR_BORROWED} stroke-width="1.5" />
			</g>
		{/each}

		<!-- Input markers (teal) — drawn last so they sit on top -->
		{#each inputMarkers as marker}
			<g class="marker" class:flash-input={flashingInput.has(marker.midi)}>
				<circle cx={marker.x} cy={marker.y} r="7.5" fill={COLOR_INPUT} fill-opacity="0.28" />
				<circle cx={marker.x} cy={marker.y} r="5" fill={COLOR_INPUT} stroke={COLOR_INPUT} stroke-width="1.5" />
			</g>
		{/each}

		<!-- Press-flash overlays — cream pulse at the clicked fret cell -->
		{#each [...pressed] as key}
			{@const [s, f] = key.split(':').map(Number)}
			{@const cx = fretCenterX(f)}
			{@const y = stringY(s)}
			<circle class="press-pulse" cx={cx} cy={y} r="9" fill={COLOR_PRESS} fill-opacity="0.3" />
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

	.marker {
		filter: drop-shadow(0 0 6px currentColor);
	}

	.marker.flash-input circle {
		animation: cp-fret-flash-input 260ms ease-out;
	}

	.marker.flash circle {
		animation: cp-fret-flash-harmony 260ms ease-out;
	}

	.press-pulse {
		animation: cp-fret-press 220ms ease-out;
		pointer-events: none;
	}

	@keyframes cp-fret-flash-input {
		0%   { filter: brightness(1.8) drop-shadow(0 0 14px #4fe8c3); transform: scale(1.25); transform-origin: center; }
		50%  { filter: brightness(1.3) drop-shadow(0 0 8px  #4fe8c3); transform: scale(1.1); }
		100% { filter: brightness(1)   drop-shadow(0 0 4px  #4fe8c3); transform: scale(1); }
	}

	@keyframes cp-fret-flash-harmony {
		0%   { filter: brightness(1.8) drop-shadow(0 0 14px #ff2e88); transform: scale(1.25); transform-origin: center; }
		50%  { filter: brightness(1.3) drop-shadow(0 0 8px  #ff2e88); transform: scale(1.1); }
		100% { filter: brightness(1)   drop-shadow(0 0 4px  #ff2e88); transform: scale(1); }
	}

	@keyframes cp-fret-press {
		0%   { r: 12; fill-opacity: 0.5; }
		100% { r: 18; fill-opacity: 0;   }
	}

	@media (prefers-reduced-motion: reduce) {
		.marker.flash circle,
		.marker.flash-input circle,
		.press-pulse {
			animation: none;
		}
	}
</style>
