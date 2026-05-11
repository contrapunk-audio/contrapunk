<script lang="ts">
	/**
	 * Chord mini app (#12) — standalone chord detection.
	 *
	 * A focused page that surfaces only the chord-readout slice of
	 * Contrapunk: click piano keys, see the detected chord name.
	 * Backed by the same chord engine the main app uses
	 * (`contrapunk::chord::chord_display_with_analysis`, exposed via
	 * the existing WASM `get_note_state()`).
	 *
	 * Routing-aware: this page does NOT start the full router thread.
	 * It boots the WASM engine, drives input via `note_on` /
	 * `note_off` calls, and reads chord state via the engine store's
	 * existing `inputNotes` + `chordName` plumbing.
	 *
	 * No marketing copy here per the project's
	 * marketing-lives-in-the-website-repo convention.
	 */
	import { onMount } from 'svelte';
	import Piano from '$lib/embed/Piano.svelte';
	import ChordReadout from '$lib/embed/ChordReadout.svelte';
	import { adapter } from '$lib/adapter';
	import { engine } from '$lib/stores/engine.svelte';

	let initError = $state<string | null>(null);

	onMount(async () => {
		try {
			await adapter.init();
		} catch (err) {
			initError = err instanceof Error ? err.message : String(err);
		}
	});

	async function onNoteOn(midi: number) {
		try {
			await adapter.noteOn(midi, 100);
		} catch (err) {
			// Swallow — the page should keep rendering even if a single
			// click fails. Real errors surface via initError above.
			console.warn('[chord-app] noteOn failed', err);
		}
	}

	async function onNoteOff(midi: number) {
		try {
			await adapter.noteOff(midi);
		} catch (err) {
			console.warn('[chord-app] noteOff failed', err);
		}
	}
</script>

<svelte:head>
	<title>Chord Detector — Contrapunk</title>
	<meta name="description" content="Standalone chord detection. Click piano keys, see the chord name." />
</svelte:head>

<main class="chord-app">
	<header>
		<h1>Chord Detector</h1>
		<p class="sub">Click piano keys. The detected chord name appears below.</p>
	</header>

	{#if initError}
		<div class="error" data-testid="chord-init-error">
			Couldn't load the chord engine: {initError}
		</div>
	{:else}
		<section class="readout">
			<div data-testid="chord-name">
					<ChordReadout chordName={engine.chordName} variant="display" />
				</div>
			<div class="active-notes" data-testid="active-notes">
				{#if engine.inputNotes.length === 0}
					<span class="hint">(no notes held)</span>
				{:else}
					{engine.inputNotes.length} note{engine.inputNotes.length === 1 ? '' : 's'} held
				{/if}
			</div>
		</section>

		<section class="piano-host" data-testid="chord-piano">
			<Piano
				inputNotes={engine.inputNotes}
				harmonyNotes={[]}
				borrowedNotes={[]}
				{onNoteOn}
				{onNoteOff}
				interactive={true}
				showLabels={true}
			/>
		</section>
	{/if}
</main>

<style>
	.chord-app {
		max-width: 960px;
		margin: 0 auto;
		padding: 24px 16px;
		color: var(--color-text-primary);
		font-family: var(--font-ui), sans-serif;
	}

	header {
		text-align: center;
		margin-bottom: 24px;
	}

	h1 {
		margin: 0 0 4px;
		font-size: 1.5rem;
		color: var(--color-accent-gold);
	}

	.sub {
		margin: 0;
		font-size: 0.9rem;
		color: var(--color-text-secondary);
	}

	.readout {
		text-align: center;
		padding: 24px 16px;
		margin-bottom: 24px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
	}

	/* .chord-name styles moved into embed/ChordReadout.svelte
	   variant="display". Same font-size, color, font-family. */

	.active-notes {
		font-size: 0.9rem;
		color: var(--color-text-secondary);
		margin-top: 8px;
	}

	.hint {
		opacity: 0.6;
	}

	.piano-host {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		padding: 12px;
	}

	.error {
		padding: 16px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-accent-red, #c33);
		color: var(--color-accent-red, #c33);
		text-align: center;
	}
</style>
