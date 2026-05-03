<script lang="ts">
	/**
	 * Fretboard wrapper — feeds engine + adapter into the canonical
	 * embed Fretboard. The actual SVG / animation lives in
	 * $lib/embed/Fretboard.svelte; this file only binds engine state
	 * + click handlers. Per #66.
	 */
	import EmbedFretboard from '$lib/embed/Fretboard.svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { adapter } from '$lib/adapter';
	import { ui } from '$lib/stores/ui.svelte';
</script>

<EmbedFretboard
	inputNotes={engine.inputNotes}
	harmonyNotes={engine.harmonyNotes}
	borrowedNotes={engine.borrowedNotes}
	onNoteOn={(midi) => adapter.injectNoteOn(midi, 100)}
	onNoteOff={(midi) => adapter.injectNoteOff(midi)}
	showLabels={ui.showNoteLabels}
	lingering={ui.noteLingering}
/>
