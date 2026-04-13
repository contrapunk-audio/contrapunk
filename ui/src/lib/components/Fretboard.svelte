<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { suggestions, type SuggestionScore } from '$lib/stores/suggestion.svelte';

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
		// Display: high E at top (string 5 -> y=18), low E at bottom (string 0 -> y=108)
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

	/** Simple hand position estimation (mean of active frets). */
	function estimateHandPosition(): number {
		const activeNotes = engine.inputNotes;
		if (activeNotes.length === 0) return 5; // default position
		let sumFret = 0;
		let count = 0;
		for (const note of activeNotes) {
			const positions = midiToPositions(note);
			if (positions.length > 0) {
				// Pick position with lowest fret as most likely
				const best = positions.reduce((a, b) => (a.fret < b.fret ? a : b));
				sumFret += best.fret;
				count++;
			}
		}
		return count > 0 ? sumFret / count : 5;
	}

	/** Score a position's ergonomic distance from hand position. */
	function positionScore(fret: number, handPos: number): number {
		const dist = Math.abs(fret - handPos);
		const stretch = Math.max(0, dist - 4);
		const cost = dist + 3 * stretch + 0.3 * Math.max(0, fret - 12) / 12;
		return Math.exp(-cost / 4);
	}

	/**
	 * Build fretboard markers from suggestions.
	 * Returns an array of marker objects for rendering.
	 */
	function buildMarkers(): {
		x: number;
		y: number;
		color: string;
		opacity: number;
		label: string;
	}[] {
		if (!suggestions.enabled || suggestions.suggestions.length === 0) return [];

		const handPos = estimateHandPosition();
		const markers: {
			x: number;
			y: number;
			color: string;
			opacity: number;
			label: string;
		}[] = [];

		const top3 = suggestions.suggestions.slice(0, 3);
		const next5 = suggestions.suggestions.slice(3, 8);

		function addMarkers(items: SuggestionScore[], color: string) {
			for (const { note } of items) {
				const positions = midiToPositions(note);
				if (positions.length === 0) continue;

				// Score each position
				const scored = positions.map((p) => ({
					...p,
					score: positionScore(p.fret, handPos)
				}));
				scored.sort((a, b) => b.score - a.score);

				// Primary position: full opacity
				const primary = scored[0];
				markers.push({
					x: fretCenterX(primary.fret),
					y: stringY(primary.string),
					color,
					opacity: 1.0,
					label: noteNameShort(note)
				});

				// Ghost positions (score > 0.3): 30% opacity
				for (let i = 1; i < scored.length; i++) {
					if (scored[i].score > 0.3) {
						markers.push({
							x: fretCenterX(scored[i].fret),
							y: stringY(scored[i].string),
							color,
							opacity: 0.3,
							label: ''
						});
					}
				}
			}
		}

		addMarkers(top3, '#00e436'); // green for top-3
		addMarkers(next5, '#ffdd44'); // yellow for next-5

		return markers;
	}

	/** Short note name from MIDI number. */
	function noteNameShort(midi: number): string {
		const names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
		return names[midi % 12];
	}

	/** Get markers for currently active (input) notes. */
	function buildActiveMarkers(): { x: number; y: number }[] {
		const markers: { x: number; y: number }[] = [];
		for (const note of engine.inputNotes) {
			const positions = midiToPositions(note);
			if (positions.length === 0) continue;
			// Show the most likely position (lowest fret)
			const best = positions.reduce((a, b) => (a.fret < b.fret ? a : b));
			markers.push({
				x: fretCenterX(best.fret),
				y: stringY(best.string)
			});
		}
		return markers;
	}

	let suggestionMarkers = $derived(buildMarkers());
	let activeMarkers = $derived(buildActiveMarkers());
</script>

{#if suggestions.enabled}
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

			<!-- Suggestion markers -->
			{#each suggestionMarkers as marker}
				<circle
					cx={marker.x}
					cy={marker.y}
					r="6"
					fill={marker.color}
					opacity={marker.opacity}
				/>
				{#if marker.label}
					<text
						x={marker.x}
						y={marker.y + 3}
						fill="#0a0a1a"
						font-size="5"
						font-family="'Press Start 2P', monospace"
						text-anchor="middle"
					>
						{marker.label}
					</text>
				{/if}
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
{/if}

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
