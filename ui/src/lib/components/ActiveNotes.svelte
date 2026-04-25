<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';

	const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

	/** Convert MIDI note number to note name (e.g., 60 -> "C4"). */
	function midiToName(midi: number): string {
		const octave = Math.floor(midi / 12) - 1;
		return NOTE_NAMES[midi % 12] + octave;
	}

	/** Sort MIDI notes by pitch (low to high) and convert to names. */
	function notesToNames(notes: number[]): string[] {
		return [...notes].sort((a, b) => a - b).map(midiToName);
	}

	// Derived reactive state
	let inputNames = $derived(notesToNames(engine.inputNotes));
	let harmonyNames = $derived(notesToNames(engine.harmonyNotes));
	let hasInput = $derived(engine.inputNotes.length > 0);
	let hasHarmony = $derived(engine.harmonyNotes.length > 0);
</script>

<!-- Horizontal strip: header + chord + input + harmony on one row.
     Sized to slot directly above the piano area so the active pitches
     are right next to the keys lighting up. -->
<div class="strip">
	<span class="strip-header font-ui">NOTES</span>

	{#if engine.chordName}
		<span class="chord-name font-code">{engine.chordName}</span>
	{/if}

	{#if engine.lastBorrowedFrom}
		<span class="borrowed-from font-code">from {engine.lastBorrowedFrom}</span>
	{/if}

	<span class="divider" aria-hidden="true">·</span>

	<span class="note-section">
		<span class="section-label font-ui">INPUT</span>
		{#if hasInput}
			<span class="note-list input-notes font-code">{inputNames.join(' ')}</span>
		{:else}
			<span class="note-none font-code">—</span>
		{/if}
	</span>

	<span class="divider" aria-hidden="true">·</span>

	<span class="note-section">
		<span class="section-label font-ui">HARMONY</span>
		{#if hasHarmony}
			<span class="note-list harmony-notes font-code">{harmonyNames.join(' ')}</span>
		{:else}
			<span class="note-none font-code">—</span>
		{/if}
	</span>
</div>

<style>
	.strip {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 12px;
		font-size: var(--font-size-xs);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.strip-header {
		color: var(--color-accent-gold);
		letter-spacing: 2px;
	}

	.divider {
		color: var(--color-text-dim);
	}

	.chord-name {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-sm);
		text-shadow: 0 0 6px rgba(51, 221, 255, 0.5);
	}

	.borrowed-from {
		color: var(--color-piano-borrowed);
	}

	.note-section {
		display: inline-flex;
		align-items: baseline;
		gap: 6px;
	}

	.section-label {
		color: var(--color-text-dim);
		letter-spacing: 1px;
	}

	.note-list {
		letter-spacing: 0.5px;
	}

	.input-notes {
		color: var(--color-piano-input);
	}

	.harmony-notes {
		color: var(--color-piano-harmony);
	}

	.note-none {
		color: var(--color-text-dim);
	}
</style>
