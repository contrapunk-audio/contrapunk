<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { adapter } from '$lib/adapter';
	import { ui } from '$lib/stores/ui.svelte';
	import { getPianoKeyColor } from '$lib/theme/colors';
	import { formatMusicalString } from '$lib/embed/music-utils';

	/** Contrast text color for a key based on which state is driving its fill.
	 *  Input (teal) + default-cream fills are light → dark label.
	 *  Harmony (magenta) + borrowed (violet) fills are mid-dark → light label.
	 *  Un-lit black keys stay with the default pale-violet text. */
	function labelColorFor(midi: number, isBlack: boolean): string {
		if (engine.inputNotes.includes(midi)) return '#0a0612';    // dark on teal
		if (engine.harmonyNotes.includes(midi)) return '#f5e9c9';  // cream on magenta
		if (engine.borrowedNotes.includes(midi)) return '#f5e9c9'; // cream on violet
		return isBlack ? 'var(--color-text-primary)' : 'var(--color-bg-deep)';
	}

	// MIDI range for standard 88-key piano: A0 (21) to C8 (108)
	const PIANO_START = 21;
	const PIANO_END = 108;

	/** Returns true if a MIDI note is a black key. */
	function isBlackKey(midi: number): boolean {
		return [1, 3, 6, 8, 10].includes(midi % 12);
	}

	/** Convert MIDI note number to note name (e.g., 60 -> "C4"). Uses
	 *  Unicode ♯ for sharps so the rendered key labels read as music
	 *  notation rather than ASCII identifiers. */
	function midiToName(midi: number): string {
		const names = ['C', 'C♯', 'D', 'D♯', 'E', 'F', 'F♯', 'G', 'G♯', 'A', 'A♯', 'B'];
		const octave = Math.floor(midi / 12) - 1;
		return names[midi % 12] + octave;
	}

	// Precompute key arrays
	const allMidi = Array.from({ length: PIANO_END - PIANO_START + 1 }, (_, i) => i + PIANO_START);
	const whiteKeys = allMidi.filter((n) => !isBlackKey(n));
	const blackKeys = allMidi.filter((n) => isBlackKey(n));

	// Build white key position map: midi -> index in white key row
	const whiteKeyIndex = new Map<number, number>();
	whiteKeys.forEach((midi, i) => {
		whiteKeyIndex.set(midi, i);
	});

	// For each black key, find the white key just below it and compute x offset
	function getBlackKeyPosition(midi: number, whiteKeyWidthPx: number): number {
		const prevWhite = midi - 1;
		const wIndex = whiteKeyIndex.get(prevWhite);
		if (wIndex === undefined) return 0;
		return wIndex * whiteKeyWidthPx + whiteKeyWidthPx - (whiteKeyWidthPx * 0.6) / 2;
	}

	/** Get CSS color for a key based on current note state. */
	function keyColor(midi: number): string {
		return getPianoKeyColor(
			midi,
			engine.inputNotes,
			engine.harmonyNotes,
			engine.borrowedNotes,
			engine.inScaleNotes
		);
	}

	/** Get box-shadow for active keys (neon glow). */
	function keyGlow(midi: number): string {
		if (engine.inputNotes.includes(midi)) {
			return '0 0 6px #4fe8c3, 0 0 14px #4fe8c366';
		}
		if (engine.harmonyNotes.includes(midi)) {
			return '0 0 6px #ff2e88, 0 0 14px #ff2e8866';
		}
		if (engine.borrowedNotes.includes(midi)) {
			return '0 0 6px #8a5cff, 0 0 14px #8a5cff66';
		}
		return 'none';
	}

	/** Check if a note is currently active (any state). */
	function isActive(midi: number): boolean {
		return (
			engine.inputNotes.includes(midi) ||
			engine.harmonyNotes.includes(midi) ||
			engine.borrowedNotes.includes(midi)
		);
	}

	/** Check if a note is in the current scale. */
	function isInScale(midi: number): boolean {
		return engine.inScaleNotes.includes(midi);
	}

	const NUM_WHITE_KEYS = whiteKeys.length; // 52

	// =========================================================
	// Click-to-play wiring + press-flash animation (issue #38)
	// =========================================================

	// Tracks which pointer ids hold which midi notes so we can release
	// correctly on pointerup, pointercancel, or pointerleave.
	const pressedByPointer = new Map<number, number>();

	// Midi notes currently showing the press-flash class.
	let pressing = $state(new Set<number>());

	function handlePointerDown(midi: number, e: PointerEvent) {
		const el = e.currentTarget as HTMLElement;
		try {
			el.setPointerCapture(e.pointerId);
		} catch {
			// setPointerCapture can throw on some touch devices — safe to ignore
		}
		pressedByPointer.set(e.pointerId, midi);
		pressing = new Set(pressing).add(midi);
		adapter.injectNoteOn(midi, 100);
	}

	function releasePointer(midi: number, pointerId: number) {
		if (pressedByPointer.get(pointerId) === midi) {
			adapter.injectNoteOff(midi);
			pressedByPointer.delete(pointerId);
		}
		const next = new Set(pressing);
		next.delete(midi);
		pressing = next;
	}

	function handlePointerUp(midi: number, e: PointerEvent) {
		releasePointer(midi, e.pointerId);
	}

	function handlePointerCancel(midi: number, e: PointerEvent) {
		releasePointer(midi, e.pointerId);
	}

	// =========================================================
	// Harmony / input flash on new note arrival (issue #37, perf #53)
	// Each arrival mounts one overlay element keyed by id; animationend
	// removes it. No Set-mutation, no setTimeout-driven second update.
	// =========================================================

	type Flash = { id: number; midi: number; kind: 'harmony' | 'input' };
	type Ghost = { id: number; midi: number; kind: 'harmony' | 'input' };
	let flashes = $state<Flash[]>([]);
	let ghosts = $state<Ghost[]>([]);
	let nextFlashId = 0;
	let nextGhostId = 0;

	let prevHarmony: Set<number> = new Set();
	let prevInput: Set<number> = new Set();

	// Combined arrival + release tracking. Newly-arrived notes spawn a
	// press flash; newly-released notes spawn a Hyper Light Drifter
	// afterimage ghost that fades over ~520ms. Both overlays mount per
	// key and self-remove via `onanimationend`.
	$effect(() => {
		const current = new Set(engine.harmonyNotes);
		const newFlashes: Flash[] = [];
		const newGhosts: Ghost[] = [];
		for (const m of current) {
			if (!prevHarmony.has(m)) newFlashes.push({ id: nextFlashId++, midi: m, kind: 'harmony' });
		}
		// Only spawn ghosts when the user has enabled the lingering trail
		// in Settings. prevHarmony still gets updated either way so that
		// toggling the setting doesn't produce a flood of ghosts on the
		// next state change.
		if (ui.noteLingering) {
			for (const m of prevHarmony) {
				if (!current.has(m)) newGhosts.push({ id: nextGhostId++, midi: m, kind: 'harmony' });
			}
		}
		prevHarmony = current;
		if (newFlashes.length) flashes = [...flashes, ...newFlashes];
		if (newGhosts.length) ghosts = [...ghosts, ...newGhosts];
	});

	$effect(() => {
		const current = new Set(engine.inputNotes);
		const newFlashes: Flash[] = [];
		const newGhosts: Ghost[] = [];
		for (const m of current) {
			if (!prevInput.has(m)) newFlashes.push({ id: nextFlashId++, midi: m, kind: 'input' });
		}
		if (ui.noteLingering) {
			for (const m of prevInput) {
				if (!current.has(m)) newGhosts.push({ id: nextGhostId++, midi: m, kind: 'input' });
			}
		}
		prevInput = current;
		if (newFlashes.length) flashes = [...flashes, ...newFlashes];
		if (newGhosts.length) ghosts = [...ghosts, ...newGhosts];
	});

	function removeFlash(id: number) {
		flashes = flashes.filter((f) => f.id !== id);
	}
	function removeGhost(id: number) {
		ghosts = ghosts.filter((g) => g.id !== id);
	}
</script>

<div class="piano-wrapper">
	<!-- Always-rendered chord display. Content swaps between the chord
	     name and a non-breaking space so the line-box never collapses,
	     which would otherwise mount/unmount this div on every silence↔
	     playing boundary and shift .piano-wrapper height by ~24px. -->
	<div class="chord-display font-code" class:empty={!engine.chordName}>
		{engine.chordName ? formatMusicalString(engine.chordName) : ' '}
	</div>
	<div class="piano-container" style="--num-white-keys: {NUM_WHITE_KEYS};">
		<!-- White keys -->
		{#each whiteKeys as midi}
			{@const color = keyColor(midi)}
			{@const glow = keyGlow(midi)}
			{@const active = isActive(midi)}
			{@const inScale = isInScale(midi)}
			<div
				class="white-key"
				class:in-scale={inScale && !active}
				class:pressed={pressing.has(midi)}
				data-midi={midi}
				style:background={color || 'var(--color-text-primary)'}
				style:box-shadow={glow}
				onpointerdown={(e) => handlePointerDown(midi, e)}
				onpointerup={(e) => handlePointerUp(midi, e)}
				onpointercancel={(e) => handlePointerCancel(midi, e)}
				onpointerleave={(e) => handlePointerCancel(midi, e)}
				role="button"
				tabindex="-1"
				aria-label={midiToName(midi)}
			>
				{#if inScale && !active}
					<div class="scale-overlay"></div>
				{/if}
				{#each flashes.filter((f) => f.midi === midi) as f (f.id)}
					<span class="flash flash-{f.kind}" onanimationend={() => removeFlash(f.id)}></span>
				{/each}
				{#each ghosts.filter((g) => g.midi === midi) as g (g.id)}
					<span class="ghost ghost-{g.kind}" onanimationend={() => removeGhost(g.id)}></span>
				{/each}
				{#if active && ui.showNoteLabels}
					<span class="key-label font-code" style:color={labelColorFor(midi, isBlackKey(midi))}>{midiToName(midi)}</span>
				{/if}
			</div>
		{/each}
		<!-- Black keys positioned absolutely -->
		{#each blackKeys as midi}
			{@const color = keyColor(midi)}
			{@const glow = keyGlow(midi)}
			{@const active = isActive(midi)}
			{@const inScale = isInScale(midi)}
			<div
				class="black-key"
				class:in-scale={inScale && !active}
				class:pressed={pressing.has(midi)}
				data-midi={midi}
				style:left="calc({getBlackKeyPosition(midi, 1)} * (100% / {NUM_WHITE_KEYS}))"
				style:width="calc(0.6 * (100% / {NUM_WHITE_KEYS}))"
				style:background={color || '#111'}
				style:box-shadow={glow}
				onpointerdown={(e) => handlePointerDown(midi, e)}
				onpointerup={(e) => handlePointerUp(midi, e)}
				onpointercancel={(e) => handlePointerCancel(midi, e)}
				onpointerleave={(e) => handlePointerCancel(midi, e)}
				role="button"
				tabindex="-1"
				aria-label={midiToName(midi)}
			>
				{#if inScale && !active}
					<div class="scale-overlay"></div>
				{/if}
				{#each flashes.filter((f) => f.midi === midi) as f (f.id)}
					<span class="flash flash-{f.kind}" onanimationend={() => removeFlash(f.id)}></span>
				{/each}
				{#each ghosts.filter((g) => g.midi === midi) as g (g.id)}
					<span class="ghost ghost-{g.kind}" onanimationend={() => removeGhost(g.id)}></span>
				{/each}
				{#if active && ui.showNoteLabels}
					<span class="key-label font-code" style:color={labelColorFor(midi, isBlackKey(midi))}>{midiToName(midi)}</span>
				{/if}
			</div>
		{/each}
	</div>
</div>

<style>
	.piano-wrapper {
		width: 100%;
		position: relative;
	}

	.chord-display {
		text-align: center;
		color: var(--color-accent-cyan);
		font-size: var(--font-size-xs);
		padding: 2px 0;
		background: var(--color-bg-deep);
		border-bottom: 1px solid var(--color-border);
	}

	/* Empty state: keep the box (and therefore .piano-wrapper height)
	   stable but make it visually inert. visibility:hidden preserves
	   layout box, removes paint. */
	.chord-display.empty {
		visibility: hidden;
	}

	.piano-container {
		position: relative;
		width: 100%;
		height: 90px;
		display: flex;
		flex-direction: row;
		background: var(--color-bg-deep);
	}

	.white-key {
		flex: 1;
		height: 100%;
		border-right: 1px solid var(--color-border);
		border-radius: 0;
		position: relative;
		z-index: 1;
		display: flex;
		align-items: flex-end;
		justify-content: center;
		padding-bottom: 2px;
		cursor: pointer;
		user-select: none;
		-webkit-user-select: none;
		-webkit-tap-highlight-color: transparent;
		transition: transform 60ms ease-out;
	}

	.black-key {
		position: absolute;
		top: 0;
		height: 55%;
		z-index: 2;
		border-radius: 0;
		border-left: 1px solid #000;
		border-right: 1px solid #000;
		border-bottom: 1px solid #000;
		display: flex;
		align-items: flex-end;
		justify-content: center;
		padding-bottom: 2px;
		cursor: pointer;
		user-select: none;
		-webkit-user-select: none;
		-webkit-tap-highlight-color: transparent;
		transition: transform 60ms ease-out;
	}

	.key-label {
		font-size: var(--font-size-xs);
		color: var(--color-bg-deep);
		pointer-events: none;
		line-height: 1;
		text-align: center;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.black-key .key-label {
		color: var(--color-text-primary);
		font-size: var(--font-size-xs);
	}

	.scale-overlay {
		position: absolute;
		inset: 0;
		pointer-events: none;
		border-radius: inherit;
	}

	.white-key .scale-overlay {
		background: rgba(255, 253, 140, 0.3);
	}

	.black-key .scale-overlay {
		background: rgba(255, 253, 140, 0.2);
	}

	/* =========================================================
	   Press-flash (pointerdown) + harmony/input flash on new note
	   ========================================================= */

	.white-key.pressed,
	.black-key.pressed {
		animation: cp-key-press 160ms ease-out;
	}

	@keyframes cp-key-press {
		0%   { transform: scaleY(0.94); filter: brightness(1.4); }
		60%  { transform: scaleY(0.98); filter: brightness(1.2); }
		100% { transform: scaleY(1);    filter: brightness(1);   }
	}

	/* Overlay elements mounted on arrival (issue #53). animationend
	   removes them — no JS Set + setTimeout re-render cycle. */
	.flash {
		position: absolute;
		inset: 0;
		pointer-events: none;
		border-radius: inherit;
		z-index: 3;
	}

	.flash.flash-input {
		animation: cp-flash-input 220ms ease-out forwards;
	}

	.flash.flash-harmony {
		animation: cp-flash-harmony 220ms ease-out forwards;
	}

	@keyframes cp-flash-input {
		0%   { box-shadow: 0 0 24px #4fe8c3, 0 0 48px #4fe8c3aa, 0 0 0 2px #4fe8c3; background: rgba(255, 255, 255, 0.35); }
		50%  { box-shadow: 0 0 14px #4fe8c3, 0 0 28px #4fe8c388;                    background: rgba(255, 255, 255, 0.15); }
		100% { box-shadow: 0 0 6px  #4fe8c3, 0 0 14px #4fe8c366;                    background: transparent;               }
	}

	@keyframes cp-flash-harmony {
		0%   { box-shadow: 0 0 24px #ff2e88, 0 0 48px #ff2e88aa, 0 0 0 2px #ff2e88; background: rgba(255, 255, 255, 0.35); }
		50%  { box-shadow: 0 0 14px #ff2e88, 0 0 28px #ff2e8888;                    background: rgba(255, 255, 255, 0.15); }
		100% { box-shadow: 0 0 6px  #ff2e88, 0 0 14px #ff2e8866;                    background: transparent;               }
	}

	/* Hyper Light Drifter afterimage on note release. The cell holds the
	   active color briefly, then fades opacity → 0 while desaturating
	   and brightening — feels like the note is echoing off the key.
	   z-index: 4 puts it above .scale-overlay (auto z) and even above
	   .flash (z 3) so it's never hidden by either. */
	.ghost {
		position: absolute;
		inset: 0;
		pointer-events: none;
		border-radius: inherit;
		z-index: 4;
		animation: cp-ghost-fade 520ms linear forwards;
	}
	.ghost.ghost-input {
		background: #4fe8c3;
		box-shadow: 0 0 12px #4fe8c3, 0 0 24px #4fe8c3aa;
	}
	.ghost.ghost-harmony {
		background: #ff2e88;
		box-shadow: 0 0 12px #ff2e88, 0 0 24px #ff2e88aa;
	}

	@keyframes cp-ghost-fade {
		0%   { opacity: 1;    filter: saturate(1.2) brightness(1.4); }
		40%  { opacity: 0.7;  filter: saturate(1)   brightness(1.3); }
		70%  { opacity: 0.4;  filter: saturate(0.6) brightness(1.4); }
		100% { opacity: 0;    filter: saturate(0.2) brightness(1.5); }
	}

	@media (prefers-reduced-motion: reduce) {
		.white-key, .black-key {
			transition: none;
		}
		.white-key.pressed, .black-key.pressed,
		.flash, .ghost {
			animation: none;
		}
	}
</style>
