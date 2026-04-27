<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';

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

	type GuitarSignal = {
		rms: number;
		frequency: number | null;
		clarity: number;
		note_state: number; // 0=Idle 1=Attack 2=Sustain 3=Decay
		note_name: string;
		midi_note: number;
	};

	const STATE_LABELS = ['Idle', 'Attack', 'Sustain', 'Decay'];

	let config = $state<GuitarConfig | null>(null);
	let error = $state<string>('');
	let dirty = $state(false);
	let saving = $state(false);
	let lastSaved = $state<string>('');

	// Live signal state — populated by `guitar-signal` events emitted by the
	// router thread when the active input is Guitar Audio.
	let signal = $state<GuitarSignal | null>(null);
	let signalLastTs = $state<number>(0);
	let signalSubscribed = $state(false);
	let unlistenSignal: UnlistenFn | null = null;

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
			lastSaved = new Date().toLocaleTimeString();
		} catch (e) {
			error = `Failed to save: ${e}`;
		} finally {
			saving = false;
		}
	}

	function markDirty() {
		dirty = true;
	}

	// Auto-save 250ms after the last edit so live tuning doesn't require
	// clicking save between every slider tweak.
	let saveTimer: ReturnType<typeof setTimeout> | null = null;
	function scheduleSave() {
		markDirty();
		if (saveTimer) clearTimeout(saveTimer);
		saveTimer = setTimeout(() => {
			saveConfig();
		}, 250);
	}

	async function subscribeSignal() {
		try {
			unlistenSignal = await listen<GuitarSignal>('guitar-signal', (event) => {
				signal = event.payload;
				signalLastTs = Date.now();
			});
			signalSubscribed = true;
		} catch (e) {
			console.warn('[debug] guitar-signal subscribe failed:', e);
		}
	}

	// Stale check — if no event for 500ms, the router probably stopped or
	// isn't in Guitar Audio mode. Surface that explicitly. `_staleTick`
	// is read inside the derived so the derivation re-runs every 250ms
	// even when no signal events arrive.
	let staleTimer: ReturnType<typeof setInterval> | null = null;
	let _staleTick = $state(0);
	let signalStale = $derived(
		_staleTick >= 0 && signalSubscribed && signalLastTs > 0 && Date.now() - signalLastTs > 500
	);
	function startStaleTicker() {
		staleTimer = setInterval(() => {
			_staleTick++;
		}, 250);
	}

	onMount(() => {
		fetchConfig();
		subscribeSignal();
		startStaleTicker();
	});

	onDestroy(() => {
		if (unlistenSignal) unlistenSignal();
		if (staleTimer) clearInterval(staleTimer);
		if (saveTimer) clearTimeout(saveTimer);
	});
</script>

<svelte:head>
	<title>Contrapunk — Guitar Pipeline Debug</title>
</svelte:head>

<main>
	<header>
		<h1>Guitar Pipeline Debug</h1>
		<p class="subtitle">
			Live <code>GuitarInputConfig</code> editor. Edits auto-save 250ms after the last change and
			apply mid-routing — no need to stop/restart for most fields. Stream-affecting fields
			(<code>buffer_size</code>, <code>hop_size</code>, <code>sample_rate</code>) still need a restart.
		</p>
		<p class="status">
			{#if dirty || saving}<span class="dot saving"></span>Saving…
			{:else if lastSaved}<span class="dot ok"></span>Saved at {lastSaved}
			{:else}<span class="dot idle"></span>Loaded
			{/if}
		</p>
	</header>

	{#if error}
		<div class="error">{error}</div>
	{/if}

	<section class="live-signal" class:stale={signalStale} class:offline={!signalSubscribed}>
		<h2>
			Live signal
			<span class="signal-status">
				{#if !signalSubscribed}— subscribing…
				{:else if signalLastTs === 0}— waiting for events (start routing in Guitar Audio mode)
				{:else if signalStale}— stale
				{:else}— live
				{/if}
			</span>
		</h2>
		{#if signal !== null}
			<div class="signal-grid">
				<div class="metric">
					<span class="metric-label">Note</span>
					<span class="metric-value note">{signal.note_name || '—'}</span>
					<span class="metric-sub">midi {signal.midi_note}</span>
				</div>
				<div class="metric">
					<span class="metric-label">Frequency</span>
					<span class="metric-value">{signal.frequency != null ? signal.frequency.toFixed(1) : '—'}</span>
					<span class="metric-sub">Hz</span>
				</div>
				<div class="metric">
					<span class="metric-label">Clarity</span>
					<span class="metric-value">{(signal.clarity * 100).toFixed(0)}%</span>
					<span class="metric-sub">pitch detector confidence</span>
				</div>
				<div class="metric">
					<span class="metric-label">RMS</span>
					<span class="metric-value">{signal.rms.toFixed(4)}</span>
					<span class="metric-sub">audio level</span>
				</div>
				<div class="metric">
					<span class="metric-label">State</span>
					<span class="metric-value state-{signal.note_state}">{STATE_LABELS[signal.note_state] ?? signal.note_state}</span>
					<span class="metric-sub">state machine</span>
				</div>
				<div class="metric rms-bar-cell">
					<span class="metric-label">RMS bar</span>
					<div class="rms-bar"><div class="rms-bar-fill" style="width: {Math.min(100, signal.rms * 1000)}%"></div></div>
					<span class="metric-sub">×1000 for visibility</span>
				</div>
			</div>
		{:else}
			<p class="muted">No signal yet.</p>
		{/if}
	</section>

	{#if config === null}
		<p>Loading…</p>
	{:else}
		<section>
			<h2>Pipeline gating</h2>
			<p class="section-hint">
				Thresholds the input has to clear before any MIDI fires. Tighten to reject string scrapes
				and crosstalk; loosen if soft picks aren't triggering.
			</p>
			<div class="grid">
				<label>
					<span>onset_threshold</span>
					<input
						type="number"
						step="0.005"
						min="0"
						max="1"
						bind:value={config.onset_threshold}
						oninput={scheduleSave}
					/>
					<small>RMS-style flux threshold to fire an onset.</small>
				</label>
				<label>
					<span>flux_threshold</span>
					<input
						type="number"
						step="0.05"
						min="0"
						bind:value={config.flux_threshold}
						oninput={scheduleSave}
					/>
					<small>Spectral flux gate — how much spectral energy must change.</small>
				</label>
				<label>
					<span>min_clarity</span>
					<input
						type="number"
						step="0.05"
						min="0"
						max="1"
						bind:value={config.min_clarity}
						oninput={scheduleSave}
					/>
					<small>Pitch detector confidence floor. Below = no pitch reported.</small>
				</label>
				<label>
					<span>string_confidence_min</span>
					<input
						type="number"
						step="0.05"
						min="0"
						max="1"
						bind:value={config.string_confidence_min}
						oninput={scheduleSave}
					/>
					<small>String-ID confidence floor for MPE channel routing.</small>
				</label>
				<label>
					<span>cooldown_samples</span>
					<input
						type="number"
						min="0"
						bind:value={config.cooldown_samples}
						oninput={scheduleSave}
					/>
					<small>Samples after a NoteOn before another can fire (960 = 20ms @ 48kHz).</small>
				</label>
			</div>
		</section>

		<section>
			<h2>Output / MIDI</h2>
			<div class="grid">
				<label>
					<span>pitch_bend_range</span>
					<input
						type="number"
						min="1"
						max="48"
						bind:value={config.pitch_bend_range}
						oninput={scheduleSave}
					/>
					<small>Semitones. <code>2</code> = legacy MIDI; <code>48</code> = MPE.</small>
				</label>
				<label class="check">
					<input
						type="checkbox"
						bind:checked={config.per_string_channels}
						onchange={scheduleSave}
					/>
					<span>per_string_channels</span>
					<small>One MIDI channel per string (MPE-style).</small>
				</label>
			</div>
		</section>

		<section>
			<h2>Articulations</h2>
			<p class="section-hint">
				Per-stage feature toggles. Each one off means the corresponding event type isn't emitted.
			</p>
			<div class="grid">
				<label class="check">
					<input
						type="checkbox"
						bind:checked={config.bends_enabled}
						onchange={scheduleSave}
					/>
					<span>bends_enabled</span>
					<small>Pitch bend output (initial + continuous). #79 wobble bug context.</small>
				</label>
				<label class="check">
					<input
						type="checkbox"
						bind:checked={config.legato_enabled}
						onchange={scheduleSave}
					/>
					<span>legato_enabled</span>
				</label>
				<label class="check">
					<input
						type="checkbox"
						bind:checked={config.slides_enabled}
						onchange={scheduleSave}
					/>
					<span>slides_enabled</span>
				</label>
				<label class="check">
					<input
						type="checkbox"
						bind:checked={config.vibrato_detection}
						onchange={scheduleSave}
					/>
					<span>vibrato_detection</span>
				</label>
				<label class="check">
					<input
						type="checkbox"
						bind:checked={config.vibrato_passthrough}
						onchange={scheduleSave}
					/>
					<span>vibrato_passthrough</span>
				</label>
				<label class="check">
					<input
						type="checkbox"
						bind:checked={config.filter_enabled}
						onchange={scheduleSave}
					/>
					<span>filter_enabled</span>
				</label>
				<label class="check">
					<input
						type="checkbox"
						bind:checked={config.pressure_enabled}
						onchange={scheduleSave}
					/>
					<span>pressure_enabled</span>
				</label>
				<label class="check">
					<input
						type="checkbox"
						bind:checked={config.brightness_enabled}
						onchange={scheduleSave}
					/>
					<span>brightness_enabled</span>
				</label>
			</div>
		</section>

		<section>
			<h2>DSP</h2>
			<div class="grid">
				<label>
					<span>input_gain</span>
					<input
						type="number"
						step="0.05"
						min="0"
						bind:value={config.input_gain}
						oninput={scheduleSave}
					/>
					<small>Pre-DSP audio gain (1.0 = unity).</small>
				</label>
				<label>
					<span>n_harmonics</span>
					<input
						type="number"
						min="1"
						max="16"
						bind:value={config.n_harmonics}
						oninput={scheduleSave}
					/>
					<small>How many harmonics Goertzel measures for octave correction.</small>
				</label>
				<label>
					<span>pressure_hold</span>
					<input
						type="number"
						step="0.05"
						min="0"
						max="1"
						bind:value={config.pressure_hold}
						oninput={scheduleSave}
					/>
					<small>Pressure CC hold floor for sustained notes.</small>
				</label>
			</div>
		</section>

		<section class="restart-needed">
			<h2>Audio stream <span class="warn">— needs routing restart</span></h2>
			<p class="section-hint">
				These bind to the cpal stream config. Edits won't take effect until you stop and restart
				routing.
			</p>
			<div class="grid">
				<label>
					<span>buffer_size</span>
					<input
						type="number"
						min="64"
						max="4096"
						bind:value={config.buffer_size}
						oninput={scheduleSave}
					/>
					<small>Analysis window in samples. Power of 2, [256, 4096].</small>
				</label>
				<label>
					<span>hop_size</span>
					<input
						type="number"
						min="32"
						bind:value={config.hop_size}
						oninput={scheduleSave}
					/>
					<small>Block hop between analyses.</small>
				</label>
				<label>
					<span>sample_rate</span>
					<input
						type="number"
						min="8000"
						bind:value={config.sample_rate}
						oninput={scheduleSave}
					/>
					<small>cpal sets this from the device; editing here is informational only.</small>
				</label>
			</div>
		</section>

		<details class="json-dump">
			<summary>Full config (JSON)</summary>
			<pre>{JSON.stringify(config, null, 2)}</pre>
		</details>

		<section class="actions">
			<button onclick={saveConfig} disabled={saving}>
				{saving ? 'Saving…' : 'Save now'}
			</button>
			<button onclick={fetchConfig} disabled={saving}>Reload from backend</button>
		</section>
	{/if}
</main>

<style>
	main {
		font-family: system-ui, -apple-system, sans-serif;
		max-width: 900px;
		margin: 0 auto;
		padding: 24px 28px 60px;
		color: #222;
		background: #f8f8f6;
		height: 100vh;
		overflow-y: auto;
		box-sizing: border-box;
	}

	header h1 {
		margin: 0 0 6px;
		font-size: 22px;
	}

	.subtitle {
		margin: 0 0 4px;
		font-size: 13px;
		color: #555;
		line-height: 1.5;
	}

	.subtitle code {
		font-size: 12px;
		background: #ececec;
		padding: 1px 5px;
		border-radius: 3px;
	}

	.status {
		margin: 4px 0 16px;
		font-size: 12px;
		color: #555;
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		display: inline-block;
	}
	.dot.idle { background: #9aa; }
	.dot.saving { background: #f0ad4e; }
	.dot.ok { background: #5cb85c; }

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
		padding: 14px 18px 18px;
		margin-bottom: 14px;
	}

	section h2 {
		margin: 0 0 6px;
		font-size: 13px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: #555;
	}

	.warn {
		color: #b46300;
		text-transform: none;
		letter-spacing: 0;
		font-size: 12px;
		font-weight: normal;
	}

	.section-hint {
		margin: 0 0 12px;
		font-size: 12px;
		color: #777;
		line-height: 1.4;
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 10px 16px;
	}

	label {
		display: flex;
		flex-direction: column;
		gap: 3px;
		font-family: ui-monospace, monospace;
		font-size: 12px;
	}

	label > span {
		color: #333;
		font-weight: 500;
	}

	label.check {
		flex-direction: row;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}

	label.check > span {
		flex: 1 1 auto;
		min-width: 0;
	}

	label.check small {
		flex: 1 0 100%;
	}

	input[type='number'] {
		width: 100%;
		padding: 4px 6px;
		font-family: ui-monospace, monospace;
		font-size: 12px;
		box-sizing: border-box;
	}

	small {
		font-family: system-ui, sans-serif;
		color: #888;
		font-size: 11px;
		line-height: 1.35;
	}

	small code {
		background: #ececec;
		padding: 0 4px;
		border-radius: 2px;
		font-family: ui-monospace, monospace;
	}

	.restart-needed {
		border-color: #e8c787;
		background: #fff8e8;
	}

	.live-signal {
		border-color: #2a2a2a;
		background: #1c1f24;
		color: #e6e7ea;
	}

	.live-signal.stale,
	.live-signal.offline {
		opacity: 0.65;
	}

	.live-signal h2 {
		color: #aaa;
	}

	.signal-status {
		font-size: 11px;
		color: #ddd;
		text-transform: none;
		letter-spacing: 0;
		font-weight: normal;
		margin-left: 6px;
	}

	.live-signal .muted {
		color: #777;
		font-size: 12px;
		margin: 0;
	}

	.signal-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 10px;
		margin-top: 8px;
	}

	.metric {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 10px;
		background: #14171b;
		border-radius: 4px;
	}

	.metric-label {
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: #888;
	}

	.metric-value {
		font-family: ui-monospace, monospace;
		font-size: 18px;
		color: #e6e7ea;
		line-height: 1.1;
	}

	.metric-value.note {
		color: #6ad36a;
	}

	.metric-value.state-0 { color: #888; }
	.metric-value.state-1 { color: #ff9b30; }
	.metric-value.state-2 { color: #6ad36a; }
	.metric-value.state-3 { color: #db5757; }

	.metric-sub {
		font-family: system-ui, sans-serif;
		font-size: 10px;
		color: #777;
	}

	.rms-bar-cell {
		flex-direction: column;
	}

	.rms-bar {
		width: 100%;
		height: 14px;
		background: #14171b;
		border: 1px solid #333;
		border-radius: 3px;
		overflow: hidden;
	}

	.rms-bar-fill {
		height: 100%;
		background: linear-gradient(90deg, #6ad36a 0%, #f0ad4e 70%, #db5757 100%);
		transition: width 80ms linear;
	}

	.json-dump summary {
		font-family: ui-monospace, monospace;
		font-size: 12px;
		color: #555;
		cursor: pointer;
		padding: 6px 0;
	}

	.json-dump pre {
		font-family: ui-monospace, monospace;
		font-size: 11px;
		background: #f0efea;
		padding: 12px;
		border-radius: 4px;
		overflow-x: auto;
		margin: 8px 0 0;
	}

	.actions {
		display: flex;
		gap: 8px;
		background: transparent;
		border: none;
		padding: 8px 0;
	}

	.actions button {
		font-family: system-ui, sans-serif;
		font-size: 12px;
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
