<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { adapter } from '$lib/adapter';
	import { getPianoKeyColor } from '$lib/theme/colors';

	// MIDI range for standard 88-key piano: A0 (21) to C8 (108)
	const PIANO_START = 21;
	const PIANO_END = 108;

	/** Returns true if a MIDI note is a black key. */
	function isBlackKey(midi: number): boolean {
		return [1, 3, 6, 8, 10].includes(midi % 12);
	}

	/** Convert MIDI note number to note name (e.g., 60 -> "C4"). */
	function midiToName(midi: number): string {
		const names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
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
	// Harmony / input flash on new note arrival (issue #37)
	// Fires a brief bloom animation whenever the engine emits a new note.
	// =========================================================

	// Notes currently showing the harmony-flash class.
	let flashingHarmony = $state(new Set<number>());
	let flashingInput = $state(new Set<number>());

	// Previous sets, tracked outside $effect so we diff against the last snapshot.
	let prevHarmony: Set<number> = new Set();
	let prevInput: Set<number> = new Set();

	$effect(() => {
		const currentHarmony = new Set(engine.harmonyNotes);
		const newlyArrived: number[] = [];
		for (const m of currentHarmony) {
			if (!prevHarmony.has(m)) newlyArrived.push(m);
		}
		prevHarmony = currentHarmony;
		if (newlyArrived.length === 0) return;

		const next = new Set(flashingHarmony);
		for (const m of newlyArrived) next.add(m);
		flashingHarmony = next;

		for (const m of newlyArrived) {
			setTimeout(() => {
				const later = new Set(flashingHarmony);
				later.delete(m);
				flashingHarmony = later;
			}, 220);
		}
	});

	$effect(() => {
		const currentInput = new Set(engine.inputNotes);
		const newlyArrived: number[] = [];
		for (const m of currentInput) {
			if (!prevInput.has(m)) newlyArrived.push(m);
		}
		prevInput = currentInput;
		if (newlyArrived.length === 0) return;

		const next = new Set(flashingInput);
		for (const m of newlyArrived) next.add(m);
		flashingInput = next;

		for (const m of newlyArrived) {
			setTimeout(() => {
				const later = new Set(flashingInput);
				later.delete(m);
				flashingInput = later;
			}, 220);
		}
	});
</script>

<div class="piano-wrapper">
	{#if engine.chordName}
		<div class="chord-display font-pixel">{engine.chordName}</div>
	{/if}
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
				class:flash-harmony={flashingHarmony.has(midi)}
				class:flash-input={flashingInput.has(midi)}
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
				{#if active}
					<span class="key-label font-pixel">{midiToName(midi)}</span>
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
				class:flash-harmony={flashingHarmony.has(midi)}
				class:flash-input={flashingInput.has(midi)}
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
				{#if active}
					<span class="key-label font-pixel">{midiToName(midi)}</span>
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

	.white-key.flash-input,
	.black-key.flash-input {
		animation: cp-flash-input 220ms ease-out;
	}

	.white-key.flash-harmony,
	.black-key.flash-harmony {
		animation: cp-flash-harmony 220ms ease-out;
	}

	@keyframes cp-flash-input {
		0%   { box-shadow: 0 0 24px #4fe8c3, 0 0 48px #4fe8c3aa, 0 0 0 2px #4fe8c3; filter: brightness(1.6); }
		50%  { box-shadow: 0 0 14px #4fe8c3, 0 0 28px #4fe8c388; filter: brightness(1.3); }
		100% { box-shadow: 0 0 6px #4fe8c3, 0 0 14px #4fe8c366; filter: brightness(1);   }
	}

	@keyframes cp-flash-harmony {
		0%   { box-shadow: 0 0 24px #ff2e88, 0 0 48px #ff2e88aa, 0 0 0 2px #ff2e88; filter: brightness(1.6); }
		50%  { box-shadow: 0 0 14px #ff2e88, 0 0 28px #ff2e8888; filter: brightness(1.3); }
		100% { box-shadow: 0 0 6px #ff2e88, 0 0 14px #ff2e8866; filter: brightness(1);   }
	}

	@media (prefers-reduced-motion: reduce) {
		.white-key, .black-key {
			transition: none;
		}
		.white-key.pressed, .black-key.pressed,
		.white-key.flash-input, .black-key.flash-input,
		.white-key.flash-harmony, .black-key.flash-harmony {
			animation: none;
		}
	}
</style>
