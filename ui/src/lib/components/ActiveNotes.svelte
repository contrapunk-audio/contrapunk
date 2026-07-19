<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { midiToName, formatMusicalString } from '$lib/embed/music-utils';

	/** Sort MIDI notes by pitch (low to high) and convert to names. */
	function notesToNames(notes: number[]): string[] {
		return [...notes].sort((a, b) => a - b).map(midiToName);
	}

	// Derived reactive state
	let inputNames = $derived(notesToNames(engine.inputNotes));
	let ensembleNotes = $derived([
		...new Set([...engine.harmonyNotes, ...engine.canonNotes, ...engine.counterpointNotes])
	]);
	let harmonyNames = $derived(notesToNames(ensembleNotes));
	let hasInput = $derived(engine.inputNotes.length > 0);
	let hasHarmony = $derived(ensembleNotes.length > 0);
	// Strip the trailing "(IVmaj7 in C)" roman-numeral analysis suffix
	// the Rust chord_display_with_analysis appends. Find the LAST " ("
	// (not the first — that would mangle partial chord names like
	// "Cmaj7(no 5)") and only strip it when the suffix matches the
	// analysis pattern "(... in <Key>)". Leaves chord names without
	// analysis untouched.
	function stripAnalysis(name: string): string {
		const idx = name.lastIndexOf(' (');
		if (idx < 0) return name;
		const suffix = name.slice(idx);
		return /in [A-G][b#]?\)$/.test(suffix) ? name.slice(0, idx) : name;
	}
	let chordDisplay = $derived(formatMusicalString(stripAnalysis(engine.chordName)));
	let borrowedDisplay = $derived(formatMusicalString(engine.lastBorrowedFrom));
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
			<span class="chord-name font-code">{chordDisplay}</span>
		{:else}
			<span class="chord-name dim font-code">—</span>
		{/if}
		<!-- Always-rendered borrowed-from line. visibility:hidden when
		     empty preserves the line-box height so .middle (and the
		     parent active-notes-strip) doesn't grow ~21px on first
		     borrowed-note arrival, which would otherwise force the
		     1fr content-area row to shrink in the implicit grid. -->
		<span class="borrowed-from font-code" class:empty={!engine.lastBorrowedFrom}>
			{engine.lastBorrowedFrom ? `from ${borrowedDisplay}` : ' '}
		</span>
	</div>

	<div class="side harmony-side">
		<span class="section-label font-ui">ENSEMBLE</span>
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
		/* Three equal columns. The previous `1fr auto 1fr` made the middle
		   column auto-size to the chord-name text width, so every chord
		   change (e.g. C → Cmaj7add9/E) resized the middle and shifted
		   the input/harmony columns horizontally. Equal columns lock the
		   layout and let the chord-name center inside its fixed slot. */
		grid-template-columns: 1fr 1fr 1fr;
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
		/* min-width: 0 so flex children with overflow:hidden actually clip
		   instead of growing the grid track on narrow viewports. */
		min-width: 0;
		overflow: hidden;
	}

	.chord-name {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-lg);
		font-weight: 600;
		letter-spacing: 1px;
		text-shadow: 0 0 8px rgba(51, 221, 255, 0.55), 0 0 18px rgba(51, 221, 255, 0.25);
		/* Overflow guard: clip without "..." since the dots leak into
		   the chord name visually (e.g. "Cmaj7add9..." reads wrong).
		   Long chord names just hard-clip at the column boundary. */
		max-width: 100%;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: clip;
	}
	.chord-name.dim {
		color: var(--color-text-dim);
		text-shadow: none;
	}

	.borrowed-from {
		color: var(--color-piano-borrowed);
		font-size: var(--font-size-xs);
	}

	/* Empty placeholder keeps the line-box height so .middle stays a
	   fixed 2-line column whether or not modal interchange is active. */
	.borrowed-from.empty {
		visibility: hidden;
	}

	.section-label {
		color: var(--color-text-dim);
		letter-spacing: 1px;
		flex-shrink: 0;
	}

	.note-list {
		letter-spacing: 0.5px;
		/* `<span>` is inline by default — overflow:hidden + text-overflow
		   are no-ops on inline. Force a block-level box so ellipsis
		   actually clips long note-name strings instead of forcing the
		   parent flex/grid track wider than its 1fr allotment. */
		display: inline-block;
		min-width: 0;
		max-width: 100%;
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
