<script lang="ts">
	/**
	 * PianoEmbed — thin adapter around the canonical embed Piano.
	 *
	 * Wires engine state + adapter click handlers into
	 * `$lib/embed/Piano.svelte` for use inside the main app's
	 * micro-views (canonical embed surface). Mirror-pairs with
	 * components/Fretboard.svelte, which wraps the canonical embed
	 * Fretboard the same way. Per #66 wave 3.
	 *
	 * Distinct from components/Piano.svelte:
	 *   - components/Piano.svelte = full app keyboard with harmony
	 *     flashes, ghosts, chord-readout, scale overlay.
	 *   - PianoEmbed = the small canonical keyboard, no harmony-
	 *     overlay legend, no flash/ghost animation. Just shows
	 *     coloured keys for whatever the engine is doing.
	 *
	 * Engine state passes through unchanged; the embed component
	 * already knows how to color input/harmony/borrowed notes.
	 */
	import EmbedPiano from '$lib/embed/Piano.svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { adapter } from '$lib/adapter';
	import { ui } from '$lib/stores/ui.svelte';

	let {
		interactive = true,
		keyRange
	}: {
		/** When false, the keyboard renders the engine state but ignores
		 *  pointer presses — useful for read-only micro-views. */
		interactive?: boolean;
		/** Override the viewport-derived key range. Forwarded directly
		 *  to the embed component. */
		keyRange?: { start: number; end: number };
	} = $props();
</script>

<EmbedPiano
	inputNotes={engine.inputNotes}
	harmonyNotes={engine.harmonyNotes}
	borrowedNotes={engine.borrowedNotes}
	onNoteOn={(midi) => adapter.injectNoteOn(midi, 100)}
	onNoteOff={(midi) => adapter.injectNoteOff(midi)}
	{interactive}
	showLabels={ui.showNoteLabels}
	{keyRange}
/>
