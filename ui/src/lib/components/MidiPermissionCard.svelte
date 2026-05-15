<script lang="ts">
	import { midi } from '$lib/stores/midi.svelte';
	import { platformName } from '$lib/adapter';

	let enabling = $state(false);

	/** Browser-only — Tauri and Plugin adapters report 'granted' immediately,
	 *  so the card stays hidden on those surfaces. */
	const show = $derived(
		platformName === 'browser' && midi.permissionState !== 'granted'
	);

	async function onEnable() {
		if (enabling) return;
		enabling = true;
		try {
			await midi.requestPermission();
		} finally {
			enabling = false;
		}
	}
</script>

{#if show}
	<!--
	  Web MIDI permission card. Browsers gate `navigator.requestMIDIAccess()`
	  behind a user gesture, so on first visit (or any denied/unsupported
	  state) the user needs an explicit affordance to trigger the prompt.

	  Single source of truth — InputPanel + MidiDevices both render this so
	  the prior dual-card bug (two cards, copy drift on the unsupported
	  branch's "DOWNLOAD DESKTOP APP" link) can't reoccur.
	-->
	<div class="midi-section pixel-card midi-permission-card">
		<div class="section-header font-ui">MIDI</div>

		{#if midi.permissionState === 'idle'}
			<p class="permission-text font-ui">
				Connect external MIDI keyboards, controllers, and synths.
			</p>
			<button
				class="permission-btn pixel-btn font-ui"
				onclick={onEnable}
				disabled={enabling}
			>
				{enabling ? 'REQUESTING…' : 'ENABLE MIDI'}
			</button>
		{:else if midi.permissionState === 'denied'}
			<p class="permission-text error font-ui">
				MIDI permission denied. Open your browser's site settings to
				allow it, then try again.
			</p>
			<button
				class="permission-btn pixel-btn font-ui"
				onclick={onEnable}
				disabled={enabling}
			>
				{enabling ? 'TRYING…' : 'TRY AGAIN'}
			</button>
		{:else if midi.permissionState === 'unsupported'}
			<p class="permission-text font-ui">
				Your browser doesn't support Web MIDI. Use Chrome, Edge, or
				Opera — or run Contrapunk natively on the desktop:
			</p>
			<a
				class="permission-btn pixel-btn font-ui"
				href="https://contrapunk.com"
				target="_blank"
				rel="noopener noreferrer"
			>
				DOWNLOAD DESKTOP APP →
			</a>
		{/if}
	</div>
{/if}

<style>
	.midi-section {
		padding: 4px 6px;
		margin-bottom: 2px;
	}

	.section-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		margin-bottom: 4px;
	}

	.midi-permission-card {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.permission-text {
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		line-height: 1.5;
		margin: 0;
	}

	.permission-text.error {
		color: #ff4466;
	}

	.permission-btn {
		display: inline-block;
		text-align: center;
		padding: 6px 10px;
		font-size: var(--font-size-xs);
		text-decoration: none;
		color: var(--color-accent-cyan);
		cursor: pointer;
	}

	.permission-btn[disabled] {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
