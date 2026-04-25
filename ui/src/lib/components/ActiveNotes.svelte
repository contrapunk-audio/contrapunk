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

<!-- Three-column horizontal strip: INPUT notes left, chord name
     prominent in the middle, HARMONY notes right. The chord is the
     visual centerpiece — it's what the engine produces, the per-voice
     pitches are supporting detail. -->
<div class="strip">
	<div class="side input-side">
		<span class="section-label font-ui">INPUT</span>
		{#if hasInput}
			<span class="note-list input-notes font-code">{inputNames.join(' ')}</span>
		{:else}
			<span class="note-none font-code">—</span>
		{/if}
	</div>

	<div class="middle">
		{#if engine.chordName}
			<span class="chord-name font-code">{engine.chordName}</span>
		{:else}
			<span class="chord-name dim font-code">—</span>
		{/if}
		{#if engine.lastBorrowedFrom}
			<span class="borrowed-from font-code">from {engine.lastBorrowedFrom}</span>
		{/if}
	</div>

	<div class="side harmony-side">
		<span class="section-label font-ui">HARMONY</span>
		{#if hasHarmony}
			<span class="note-list harmony-notes font-code">{harmonyNames.join(' ')}</span>
		{:else}
			<span class="note-none font-code">—</span>
		{/if}
	</div>
</div>

<style>
	.strip {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		align-items: center;
		gap: 12px;
		font-size: var(--font-size-xs);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.side {
		display: flex;
		align-items: baseline;
		gap: 8px;
		min-width: 0;
	}
	.input-side {
		justify-content: flex-start;
	}
	.harmony-side {
		justify-content: flex-end;
	}

	.middle {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
	}

	.chord-name {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-lg);
		font-weight: 600;
		letter-spacing: 1px;
		text-shadow: 0 0 8px rgba(51, 221, 255, 0.55), 0 0 18px rgba(51, 221, 255, 0.25);
	}
	.chord-name.dim {
		color: var(--color-text-dim);
		text-shadow: none;
	}

	.borrowed-from {
		color: var(--color-piano-borrowed);
		font-size: var(--font-size-xs);
	}

	.section-label {
		color: var(--color-text-dim);
		letter-spacing: 1px;
		flex-shrink: 0;
	}

	.note-list {
		letter-spacing: 0.5px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
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
