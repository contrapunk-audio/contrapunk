<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';

	type GuitarConfig = {
		buffer_size: number;
		hop_size: number;
		sample_rate: number;
		onset_threshold: number;
		string_confidence_min: number;
		bends_enabled: boolean;
		legato_enabled: boolean;
		slides_enabled: boolean;
		vibrato_detection: boolean;
		vibrato_passthrough: boolean;
		filter_enabled: boolean;
		min_clarity: number;
		cooldown_samples: number;
		n_harmonics: number;
		input_gain: number;
		flux_threshold: number;
		per_string_channels: boolean;
		pitch_bend_range: number;
		pressure_enabled: boolean;
		pressure_hold: number;
		brightness_enabled: boolean;
	};

	let config = $state<GuitarConfig | null>(null);
	let error = $state<string>('');
	let dirty = $state(false);
	let saving = $state(false);

	async function fetchConfig() {
		try {
			config = await invoke<GuitarConfig>('get_full_guitar_config');
			dirty = false;
		} catch (e) {
			error = `Failed to load: ${e}`;
		}
	}

	async function saveConfig() {
		if (!config) return;
		saving = true;
		try {
			await invoke('set_full_guitar_config', { config });
			dirty = false;
			error = '';
		} catch (e) {
			error = `Failed to save: ${e}`;
		} finally {
			saving = false;
		}
	}

	function markDirty() {
		dirty = true;
	}

	onMount(fetchConfig);
</script>

<svelte:head>
	<title>Contrapunk — Guitar Pipeline Debug</title>
</svelte:head>

<main>
	<header>
		<h1>Guitar Pipeline Debug</h1>
		<p class="subtitle">
			Live <code>GuitarInputConfig</code> editor. <strong>Note:</strong> the routing thread reads
			the config once at start, so changes here apply on next routing start, not mid-session.
		</p>
	</header>

	{#if error}
		<div class="error">{error}</div>
	{/if}

	{#if config === null}
		<p>Loading…</p>
	{:else}
		<section class="controls">
			<h2>Highlighted controls</h2>

			<div class="control-row">
				<label>
					<input
						type="checkbox"
						bind:checked={config.bends_enabled}
						onchange={markDirty}
					/>
					<span class="label-text">bends_enabled</span>
				</label>
				<span class="hint">
					Gates pitch bend output. Currently also gates the per-note initial bend that fires on every
					NoteOn — see issue #79 for the wobble bug.
				</span>
			</div>

			<div class="control-row">
				<label>
					<span class="label-text">pitch_bend_range</span>
					<input
						type="number"
						min="1"
						max="48"
						bind:value={config.pitch_bend_range}
						onchange={markDirty}
					/>
				</label>
				<span class="hint">Semitones of pitch-bend coverage. <code>2</code> = legacy MIDI; <code>48</code> = MPE.</span>
			</div>

			<div class="control-row">
				<label>
					<span class="label-text">cooldown_samples</span>
					<input
						type="number"
						min="0"
						bind:value={config.cooldown_samples}
						onchange={markDirty}
					/>
				</label>
				<span class="hint">
					Samples after a NoteOn before another can fire. <code>960</code> = 20ms @ 48kHz (HEAD default).
					<code>2400</code> = 50ms (v1.0.0 default).
				</span>
			</div>

			<div class="control-row">
				<label>
					<span class="label-text">onset_threshold</span>
					<input
						type="number"
						step="0.005"
						min="0"
						max="1"
						bind:value={config.onset_threshold}
						onchange={markDirty}
					/>
				</label>
				<span class="hint">RMS-style flux threshold to fire an onset. Higher = harder to trigger.</span>
			</div>
		</section>

		<section class="json-dump">
			<h2>Full config (read-only JSON)</h2>
			<pre>{JSON.stringify(config, null, 2)}</pre>
		</section>

		<section class="actions">
			<button onclick={saveConfig} disabled={!dirty || saving}>
				{saving ? 'Saving…' : dirty ? 'Save' : 'Saved'}
			</button>
			<button onclick={fetchConfig} disabled={saving}>Reload</button>
		</section>
	{/if}
</main>

<style>
	main {
		font-family: system-ui, -apple-system, sans-serif;
		max-width: 800px;
		margin: 0 auto;
		padding: 24px;
		color: #222;
		background: #f8f8f6;
		min-height: 100vh;
	}

	header h1 {
		margin: 0 0 8px;
		font-size: 22px;
	}

	.subtitle {
		margin: 0 0 24px;
		font-size: 13px;
		color: #555;
		line-height: 1.45;
	}

	.subtitle code {
		font-size: 12px;
		background: #ececec;
		padding: 1px 5px;
		border-radius: 3px;
	}

	.error {
		background: #ffe9e9;
		border: 1px solid #d33;
		color: #8a0000;
		padding: 8px 12px;
		border-radius: 4px;
		margin-bottom: 16px;
		font-size: 13px;
	}

	section {
		background: #fff;
		border: 1px solid #d8d8d4;
		border-radius: 6px;
		padding: 16px 18px;
		margin-bottom: 16px;
	}

	section h2 {
		margin: 0 0 12px;
		font-size: 14px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: #555;
	}

	.control-row {
		display: grid;
		grid-template-columns: minmax(220px, auto) 1fr;
		gap: 8px 16px;
		align-items: start;
		padding: 10px 0;
		border-bottom: 1px solid #f0efea;
	}

	.control-row:last-child {
		border-bottom: none;
	}

	.control-row label {
		display: flex;
		gap: 8px;
		align-items: center;
		font-family: ui-monospace, monospace;
		font-size: 13px;
	}

	.label-text {
		font-family: ui-monospace, monospace;
	}

	input[type='number'] {
		width: 80px;
		padding: 4px 6px;
		font-family: ui-monospace, monospace;
		font-size: 13px;
	}

	.hint {
		font-size: 12px;
		color: #777;
		line-height: 1.4;
	}

	.hint code {
		font-size: 11px;
		background: #ececec;
		padding: 1px 4px;
		border-radius: 3px;
	}

	.json-dump pre {
		font-family: ui-monospace, monospace;
		font-size: 12px;
		background: #f0efea;
		padding: 12px;
		border-radius: 4px;
		overflow-x: auto;
		margin: 0;
	}

	.actions {
		display: flex;
		gap: 8px;
	}

	.actions button {
		font-family: system-ui, sans-serif;
		font-size: 13px;
		padding: 6px 14px;
		border: 1px solid #444;
		background: #fff;
		color: #222;
		cursor: pointer;
		border-radius: 4px;
	}

	.actions button:disabled {
		opacity: 0.5;
		cursor: default;
	}
</style>
