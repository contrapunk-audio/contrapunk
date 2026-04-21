<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';

	// Standard tuning open string MIDI values [E2, A2, D3, G3, B3, E4]
	const STANDARD_TUNING = [40, 45, 50, 55, 59, 64];
	const STRING_LABELS = ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'];
	const NUM_FRETS = 15; // Show frets 0-15 for readability
	const FRET_MARKERS = [3, 5, 7, 9, 12, 15];
	const DOUBLE_MARKERS = [12];

	// SVG dimensions
	const SVG_WIDTH = 700;
	const SVG_HEIGHT = 130;
	const NUT_X = 40;
	const FRET_AREA_WIDTH = SVG_WIDTH - NUT_X - 10;
	const STRING_Y_START = 18;
	const STRING_SPACING = 18;

	/** Get X position for a fret. Fret 0 = nut. Uses equal spacing for simplicity. */
	function fretX(fret: number): number {
		if (fret === 0) return NUT_X;
		return NUT_X + (fret / NUM_FRETS) * FRET_AREA_WIDTH;
	}

	/** Get X center of a fret position (between fret-1 and fret). */
	function fretCenterX(fret: number): number {
		if (fret === 0) return NUT_X - 8;
		return (fretX(fret - 1) + fretX(fret)) / 2;
	}

	/** Get Y position for a string (0 = low E at bottom, 5 = high E at top). */
	function stringY(stringIdx: number): number {
		return STRING_Y_START + (5 - stringIdx) * STRING_SPACING;
	}

	/** Convert MIDI note to all valid (string, fret) positions. */
	function midiToPositions(note: number): { string: number; fret: number }[] {
		const positions: { string: number; fret: number }[] = [];
		for (let s = 0; s < 6; s++) {
			const fret = note - STANDARD_TUNING[s];
			if (fret >= 0 && fret <= NUM_FRETS) {
				positions.push({ string: s, fret });
			}
		}
		return positions;
	}

	/** Get markers for currently active (input) notes. */
	function buildActiveMarkers(): { x: number; y: number }[] {
		const markers: { x: number; y: number }[] = [];
		for (const note of engine.inputNotes) {
			const positions = midiToPositions(note);
			if (positions.length === 0) continue;
			const best = positions.reduce((a, b) => (a.fret < b.fret ? a : b));
			markers.push({
				x: fretCenterX(best.fret),
				y: stringY(best.string)
			});
		}
		return markers;
	}

	let activeMarkers = $derived(buildActiveMarkers());
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
		<rect x={NUT_X} y="10" width={FRET_AREA_WIDTH} height={STRING_SPACING * 5 + 16} fill="#1a1520" rx="1" />

		<!-- Nut -->
		<rect x={NUT_X - 3} y="10" width="4" height={STRING_SPACING * 5 + 16} fill="#555577" />

		<!-- Fret markers (dots) -->
		{#each FRET_MARKERS as fret}
			{@const cx = fretCenterX(fret)}
			{@const cy = STRING_Y_START + 2.5 * STRING_SPACING}
			{#if DOUBLE_MARKERS.includes(fret)}
				<circle cx={cx} cy={cy - 12} r="3" fill="#25223d" />
				<circle cx={cx} cy={cy + 12} r="3" fill="#25223d" />
			{:else}
				<circle cx={cx} cy={cy} r="3" fill="#25223d" />
			{/if}
		{/each}

		<!-- Fret lines -->
		{#each Array.from({ length: NUM_FRETS }, (_, i) => i + 1) as fret}
			<line
				x1={fretX(fret)}
				y1="10"
				x2={fretX(fret)}
				y2={STRING_Y_START + 5 * STRING_SPACING + 8}
				stroke="#333355"
				stroke-width="1"
			/>
		{/each}

		<!-- Strings -->
		{#each STANDARD_TUNING as _, i}
			{@const y = stringY(i)}
			<line
				x1={NUT_X}
				y1={y}
				x2={SVG_WIDTH - 10}
				y2={y}
				stroke="#888899"
				stroke-width={1.5 - i * 0.15}
			/>
		{/each}

		<!-- String labels -->
		{#each STRING_LABELS as label, i}
			<text
				x="6"
				y={stringY(i) + 3}
				fill="#555577"
				font-size="8"
				font-family="'Press Start 2P', monospace"
			>
				{label}
			</text>
		{/each}

		<!-- Fret numbers -->
		{#each FRET_MARKERS as fret}
			<text
				x={fretCenterX(fret)}
				y={SVG_HEIGHT - 4}
				fill="#444466"
				font-size="7"
				font-family="'Press Start 2P', monospace"
				text-anchor="middle"
			>
				{fret}
			</text>
		{/each}

		<!-- Active note markers (current input) -->
		{#each activeMarkers as marker}
			<circle
				cx={marker.x}
				cy={marker.y}
				r="6"
				fill="#00e436"
				stroke="#00e436"
				stroke-width="2"
				opacity="0.9"
			/>
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
	}
</style>
