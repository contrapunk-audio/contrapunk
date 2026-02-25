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

<div class="card">
	<div class="card-header font-pixel">Notes</div>

	<!-- Chord name (prominent) -->
	{#if engine.chordName}
		<div class="chord-name font-pixel">{engine.chordName}</div>
	{/if}

	<!-- Interchange source -->
	{#if engine.lastBorrowedFrom}
		<div class="borrowed-from font-pixel">from {engine.lastBorrowedFrom}</div>
	{/if}

	<!-- Input notes -->
	<div class="note-section">
		<span class="section-label font-pixel">INPUT</span>
		{#if hasInput}
			<span class="note-list input-notes font-pixel">
				{inputNames.join('  ')}
			</span>
		{:else}
			<span class="note-none font-pixel">(none)</span>
		{/if}
	</div>

	<!-- Harmony notes -->
	<div class="note-section">
		<span class="section-label font-pixel">HARMONY</span>
		{#if hasHarmony}
			<span class="note-list harmony-notes font-pixel">
				{harmonyNames.join('  ')}
			</span>
		{:else}
			<span class="note-none font-pixel">(none)</span>
		{/if}
	</div>
</div>

<style>
	.card {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		padding: 6px;
		margin-bottom: 4px;
		border-radius: 0;
	}

	.card-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		margin-bottom: 4px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.chord-name {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-sm);
		text-align: center;
		padding: 2px 0;
		margin-bottom: 4px;
		text-shadow: 0 0 6px rgba(51, 221, 255, 0.5);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.borrowed-from {
		color: var(--color-piano-borrowed);
		font-size: 6px;
		text-align: center;
		margin-bottom: 4px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.note-section {
		display: flex;
		align-items: baseline;
		gap: 6px;
		padding: 2px 0;
	}

	.section-label {
		color: var(--color-text-dim);
		font-size: 6px;
		min-width: 36px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.note-list {
		font-size: 7px;
		letter-spacing: 0.5px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
		word-break: break-word;
	}

	.input-notes {
		color: var(--color-piano-input);
	}

	.harmony-notes {
		color: var(--color-piano-harmony);
	}

	.note-none {
		color: var(--color-text-dim);
		font-size: 6px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}
</style>
