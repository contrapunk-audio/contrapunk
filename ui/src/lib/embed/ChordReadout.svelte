<script lang="ts">
	/**
	 * ChordReadout — canonical embed component. State-library-agnostic.
	 *
	 * Renders the "Fmaj7 (IVmaj7 in C)" display the chord engine
	 * produces. Sharps/flats are passed through `formatMusicalString`
	 * so the output reads as proper music notation (♯, ♭) rather than
	 * ASCII (#, b).
	 *
	 * Same props-only pattern as embed/Piano.svelte and embed/Fretboard
	 * — zero store imports. Mirror-copied to the website embed bundle.
	 * Per contrapunk-audio/contrapunk#66 wave 4.
	 */
	import { formatMusicalString } from './music-utils';

	let {
		chordName,
		variant = 'inline',
		emptyPlaceholder = ' '
	}: {
		/** Raw chord-name string from the engine, e.g. "Fmaj7 (IVmaj7 in C)". */
		chordName: string;
		/**
		 * `inline` — compact one-line readout used inside another panel
		 *           (e.g. above the Piano in the main app).
		 * `display` — large hero readout for standalone pages
		 *           (chord-detector micro-view).
		 */
		variant?: 'inline' | 'display';
		/** What to render when `chordName` is empty. Defaults to a non-
		 *  breaking space so the line-box height stays stable instead of
		 *  collapsing — same trick the main-app Piano uses. */
		emptyPlaceholder?: string;
	} = $props();

	const hasChord = $derived(!!chordName);
	const formatted = $derived(hasChord ? formatMusicalString(chordName) : emptyPlaceholder);
</script>

<div
	class="chord-readout {variant}"
	class:empty={!hasChord}
	data-testid="chord-readout"
>
	{formatted}
</div>

<style>
	.chord-readout {
		text-align: center;
		color: var(--color-accent-cyan, #33ddff);
		font-family: var(--font-code, ui-monospace, monospace);
		letter-spacing: 0.02em;
	}

	/* Inline variant — small, just keeps the line-box stable. Matches
	   the chord-display the full Piano used to render inline above
	   the keyboard. */
	.chord-readout.inline {
		font-size: var(--font-size-xs, 0.7rem);
		padding: 2px 0;
		background: var(--color-bg-deep, #0a0612);
		border-bottom: 1px solid var(--color-border, #2a1648);
	}

	/* Display variant — hero readout for standalone chord-detector. */
	.chord-readout.display {
		font-size: 2.5rem;
		font-weight: 600;
		min-height: 3rem;
	}

	/* Empty state — preserve layout box, hide paint. Same approach
	   as the main-app Piano used (visibility:hidden, not display:none)
	   so the parent .piano-wrapper height stays put. */
	.chord-readout.empty {
		visibility: hidden;
	}
</style>
