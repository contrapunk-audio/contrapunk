<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
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
		// The white key immediately below this black key
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
			return '0 0 6px #00e436, 0 0 14px #00e43666';
		}
		if (engine.harmonyNotes.includes(midi)) {
			return '0 0 6px #ff9933, 0 0 14px #ff993366';
		}
		if (engine.borrowedNotes.includes(midi)) {
			return '0 0 6px #ffaa33, 0 0 14px #ffaa3366';
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
				data-midi={midi}
				style:background={color || 'var(--color-text-primary)'}
				style:box-shadow={glow}
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
				data-midi={midi}
				style:left="calc({getBlackKeyPosition(midi, 1)} * (100% / {NUM_WHITE_KEYS}))"
				style:width="calc(0.6 * (100% / {NUM_WHITE_KEYS}))"
				style:background={color || '#111'}
				style:box-shadow={glow}
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
	}

	.key-label {
		font-size: 5px;
		color: var(--color-bg-deep);
		pointer-events: none;
		line-height: 1;
		text-align: center;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.black-key .key-label {
		color: var(--color-text-primary);
		font-size: 4px;
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
</style>
