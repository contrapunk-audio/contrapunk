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

	/**
	 * HLD-style afterimage: when a note releases, the cell stays as a
	 * fading ghost — opacity decays, saturation drops (washes toward
	 * white), and the glow expands then fades. Gives a sense of the
	 * note "echoing" off the fretboard. Applied via `out:ghostFade` on
	 * the cell-fill rect.
	 */
	function ghostFade(_node: Element, { duration = 520 }: { duration?: number } = {}) {
		return {
			duration,
			css: (t: number, u: number) => {
				// t goes 1 → 0 (visible → invisible), u goes 0 → 1
				const opacity = t;
				const brightness = 1 + u * 0.5;
				const saturate = 0.2 + t * 0.8;
				const glow = 6 + u * 18;
				return `opacity: ${opacity};
					filter: brightness(${brightness}) saturate(${saturate})
						drop-shadow(0 0 ${glow}px currentColor);`;
			}
		};
	}


	let {
		inputNotes,
		harmonyNotes,
		borrowedNotes = [],
		onNoteOn,
		onNoteOff,
		frets: fretsProp,
		interactive = true,
		showLabels = true,
		lingering = true
	}: {
		inputNotes: number[];
		harmonyNotes: number[];
		borrowedNotes?: number[];
		onNoteOn?: (midi: number) => void;
		onNoteOff?: (midi: number) => void;
		/** When omitted, fret count adapts to viewport width:
		 *  desktop = 24, tablet = 17, phone = 12. Pass an explicit
		 *  number to override (the contrapunk app overrides to 24 in
		 *  its full-width performance view). */
		frets?: number;
		interactive?: boolean;
		showLabels?: boolean;
		/** Hyper Light Drifter–style afterimage on note release.
		 *  When false, cells unmount instantly. */
		lingering?: boolean;
	} = $props();

	// Viewport-driven default fret count. 12 frets = first hand position
	// for guitarists; 24 = full neck. Tap targets stay legible by
	// dropping fret count rather than shrinking cells.
	let viewportWidth = $state(typeof window !== 'undefined' ? window.innerWidth : 1200);

	$effect(() => {
		if (typeof window === 'undefined') return;
		const onResize = () => {
			viewportWidth = window.innerWidth;
		};
		window.addEventListener('resize', onResize);
		return () => window.removeEventListener('resize', onResize);
	});

	const frets = $derived(
		fretsProp !== undefined ? fretsProp : viewportWidth < 600 ? 6 : viewportWidth < 900 ? 12 : 24
	);

	// Duration to feed the ghostFade out-transition. Setting it to 0
	// makes Svelte unmount the rect/text immediately, no fade.
	let ghostDuration = $derived(lingering ? 520 : 0);

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
							out:ghostFade={{ duration: ghostDuration }}
						/>
						{#if showLabels}
							<text
								class="cell-label"
								x={cx}
								y={y + 2.5}
								text-anchor="middle"
								fill={state === 'input' ? '#0a0612' : '#f5e9c9'}
								out:ghostFade={{ duration: ghostDuration }}
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
	.fretboard-wrapper {
		width: 100%;
		/* Bulletproof aspect-lock via padding-bottom percentage trick.
		   padding-bottom: % is calculated against the PARENT's WIDTH per
		   CSS spec, so this gives a width-derived height regardless of
		   children. Works in every browser including older WebKit, and
		   doesn't depend on `aspect-ratio` (Safari 15.4+) or `contain`
		   (which WebKit may not honor for SVG layout). The SVG is
		   absolutely positioned to fill, so it can't push the wrapper.
		   14.4231% = 150 / 1040 (matches viewBox aspect). */
		padding: 0 0 14.4231% 0;
		position: relative;
		overflow: hidden;
		/* Belt-and-suspenders: contain still helps in browsers that
		   honor it; harmless in those that don't. */
		contain: layout paint;
	}
	.fretboard-svg {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		display: block;
		touch-action: none;
	}
	.fret-hit { -webkit-tap-highlight-color: transparent; }
	.fret-hit.interactive { cursor: pointer; }

	.cell-fill {
		animation: fret-glitch-in 480ms steps(6, end);
		filter: drop-shadow(0 0 6px currentColor);
		pointer-events: none;
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
		animation: fret-glitch-in 480ms steps(6, end);
	}

	/* In-place Hyper Light Drifter style press flash — cell snaps into
	   place at its final position from frame 0, then a hot near-white
	   core + wide colored bloom decays out over ~280ms. No translate or
	   scale, so cells never jump. steps(6, end) keeps the discrete
	   pixel-art glitch character (each step looks like one frame of a
	   stop-motion neon flash). */
	@keyframes fret-glitch-in {
		0%   { opacity: 1; filter: brightness(4.5) saturate(0.3)
		         drop-shadow(0 0 18px #ffffff)
		         drop-shadow(0 0 36px currentColor); }
		15%  { opacity: 1; filter: brightness(3.2) saturate(0.7)
		         drop-shadow(0 0 14px #ffffff)
		         drop-shadow(0 0 28px currentColor); }
		30%  { opacity: 1; filter: brightness(2.2)
		         drop-shadow(0 0 10px currentColor)
		         drop-shadow(0 0 22px currentColor); }
		55%  { opacity: 1; filter: brightness(1.5)
		         drop-shadow(0 0 8px  currentColor)
		         drop-shadow(0 0 16px currentColor); }
		80%  { opacity: 1; filter: brightness(1.15)
		         drop-shadow(0 0 7px  currentColor); }
		100% { opacity: 1; filter: brightness(1)
		         drop-shadow(0 0 6px  currentColor); }
	}

	@media (prefers-reduced-motion: reduce) {
		.cell-fill, .cell-label { animation: none; }
	}
</style>
