<script lang="ts">
	import { onMount } from 'svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { adapter, platformName } from '$lib/adapter';
	import type { CalibrationStatus } from '$lib/adapter/types';
	import PixelSelect from './PixelSelect.svelte';
	import GuitarInputPanel from './GuitarInputPanel.svelte';

	// Virtual input sentinel values — must match
	// src-tauri/src/commands/engine.rs and MidiDevices.svelte.
	const VIRTUAL_COMPUTER_KEYBOARD = 999_998;
	const VIRTUAL_GUITAR_AUDIO = 999_997;

	type Source = 'midi' | 'guitar' | 'voice';

	const sourceFromSelection = (sel: number | null): Source =>
		sel === VIRTUAL_GUITAR_AUDIO ? 'guitar' : 'midi';

	let source = $derived<Source>(sourceFromSelection(midi.selectedInput));

	// Browser-only permission card. Tauri / Plugin paths report
	// `'granted'` from the adapter so this whole block disappears.
	let enabling = $state(false);
	const showPermissionCard = $derived(
		platformName === 'browser' && midi.permissionState !== 'granted'
	);

	async function onEnableMidi() {
		if (enabling) return;
		enabling = true;
		try {
			await midi.requestPermission();
		} finally {
			enabling = false;
		}
	}

	function selectSource(next: Source) {
		if (next === 'voice') return; // Disabled in v1.3
		if (next === 'guitar') {
			midi.selectVirtualInput(VIRTUAL_GUITAR_AUDIO);
			return;
		}
		// 'midi' — if we were on guitar, fall back to Computer Keyboard
		// so the user is never left without a selection.
		if (midi.selectedInput === VIRTUAL_GUITAR_AUDIO) {
			midi.selectVirtualInput(VIRTUAL_COMPUTER_KEYBOARD);
		}
	}

	// Calibration profile status (Tauri-only — gated by capability flag).
	let calibrationStatus = $state<CalibrationStatus | null>(null);
	let calibrationError = $state<string | null>(null);
	let calibrationBusy = $state(false);

	async function refreshCalibrationStatus() {
		if (!adapter.capabilities.calibrationFlow) return;
		try {
			calibrationStatus = await adapter.getCalibrationStatus();
		} catch (e) {
			calibrationError = `Status check failed: ${e}`;
		}
	}

	async function reloadCalibrationProfile() {
		if (calibrationBusy) return;
		calibrationBusy = true;
		calibrationError = null;
		try {
			calibrationStatus = await adapter.loadCalibrationProfile();
		} catch (e) {
			calibrationError = `Reload failed: ${e}`;
		} finally {
			calibrationBusy = false;
		}
	}

	const totalSamples = $derived(
		calibrationStatus
			? calibrationStatus.sampleCounts.reduce((a, b) => a + b, 0)
			: 0
	);

	onMount(() => {
		if (midi.inputs.length === 0 && !midi.isLoading) {
			midi.refresh();
		}
		refreshCalibrationStatus();
	});

	let isComputerKeyboard = $derived(midi.selectedInput === VIRTUAL_COMPUTER_KEYBOARD);

	// MIDI source — only "real" MIDI devices + Computer Keyboard.
	// Guitar Audio is hoisted out into the top-level source radio.
	let midiInputOptions = $derived([
		...midi.inputs.map((d) => ({ value: String(d.index), label: d.name })),
		{ value: String(VIRTUAL_COMPUTER_KEYBOARD), label: 'Computer Keyboard' }
	]);

	function handleMidiInputChange(value: string) {
		if (value === '') {
			midi.clearInput();
			return;
		}
		const idx = parseInt(value, 10);
		if (idx === VIRTUAL_COMPUTER_KEYBOARD) {
			midi.selectVirtualInput(idx);
		} else {
			midi.selectInput(idx);
		}
	}

	function voiceLabel(index: number, count: number): string {
		const names = ['Soprano', 'Alto', 'Tenor', 'Bass'];
		if (count <= 4) return `${names[index] || 'Voice'} (${index + 1})`;
		return `Voice (${index + 1})`;
	}
	const voicePositionOptions = $derived(
		Array.from({ length: engine.voiceCount }, (_, i) => ({
			value: String(i),
			label: voiceLabel(i, engine.voiceCount)
		}))
	);
	function onVoicePositionChange(val: string) {
		engine.setVoicePosition(parseInt(val, 10));
	}
</script>

<div class="input-panel">
	<!-- Source radio: MIDI / Guitar Audio / Voice (disabled). Replaces
	     the inline Guitar Audio option in the MIDI input dropdown so the
	     source choice is a top-level concern. Voice is reserved for the
	     v1.4 vocal pipeline (PSOLA harmonizer). -->
	<div class="source-radio pixel-card">
		<div class="section-header font-ui">SOURCE</div>
		<div class="radio-row">
			<button
				class="source-btn pixel-btn font-ui"
				class:active={source === 'midi'}
				onclick={() => selectSource('midi')}
				title="MIDI keyboard or computer-keyboard input"
			>
				MIDI
			</button>
			<button
				class="source-btn pixel-btn font-ui"
				class:active={source === 'guitar'}
				onclick={() => selectSource('guitar')}
				title="Live guitar audio in via cpal (pitch-detected)"
			>
				Guitar
			</button>
			<button
				class="source-btn pixel-btn font-ui source-disabled"
				disabled
				aria-disabled="true"
				title="Voice input (PSOLA harmonizer) lands in v1.4"
			>
				Voice
			</button>
		</div>
	</div>

	{#if showPermissionCard}
		<!-- Web MIDI permission card. Browsers gate
		     navigator.requestMIDIAccess() behind a user gesture. -->
		<div class="midi-section pixel-card midi-permission-card">
			<div class="section-header font-ui">MIDI</div>
			{#if midi.permissionState === 'idle'}
				<p class="permission-text font-ui">
					Connect external MIDI keyboards, controllers, and synths.
				</p>
				<button
					class="permission-btn pixel-btn font-ui"
					onclick={onEnableMidi}
					disabled={enabling}
				>
					{enabling ? 'REQUESTING…' : 'ENABLE MIDI'}
				</button>
			{:else if midi.permissionState === 'denied'}
				<p class="permission-text error font-ui">
					MIDI permission denied. Open your browser's site settings
					to allow it, then try again.
				</p>
				<button
					class="permission-btn pixel-btn font-ui"
					onclick={onEnableMidi}
					disabled={enabling}
				>
					{enabling ? 'TRYING…' : 'TRY AGAIN'}
				</button>
			{:else if midi.permissionState === 'unsupported'}
				<p class="permission-text font-ui">
					Your browser doesn't support Web MIDI. Use Chrome, Edge, or
					Opera — or run Contrapunk natively on the desktop.
				</p>
			{/if}
		</div>
	{/if}

	{#if source === 'midi'}
		<div class="midi-section pixel-card">
			<div class="section-header font-ui">INPUT</div>
			<div class="input-row">
				<PixelSelect
					options={midiInputOptions}
					value={midi.selectedInput !== null ? String(midi.selectedInput) : ''}
					placeholder="Select..."
					onchange={handleMidiInputChange}
				/>
				<button
					class="refresh-btn pixel-btn"
					onclick={() => midi.refresh()}
					disabled={midi.isLoading}
					title="Refresh MIDI devices"
				>
					{midi.isLoading ? '...' : 'R'}
				</button>
			</div>

			{#if isComputerKeyboard}
				<div class="keyboard-hint font-code">
					Z-M: C3-B3, Q-U: C4-C5
				</div>
			{/if}

			{#if midi.error}
				<div class="error-text font-ui">{midi.error}</div>
			{/if}

			{#if engine.voiceCount > 1}
				<div class="you-play-row">
					<span class="you-play-label font-ui">You play</span>
					<PixelSelect
						options={voicePositionOptions}
						value={String(engine.voicePosition)}
						placeholder="Voice"
						small={true}
						onchange={onVoicePositionChange}
					/>
				</div>
			{/if}
		</div>
	{:else if source === 'guitar'}
		<GuitarInputPanel />

		{#if adapter.capabilities.calibrationFlow}
			<div class="midi-section pixel-card calibration-section">
				<div class="section-header font-ui">CALIBRATION</div>
				{#if calibrationStatus}
					<div class="calibration-row">
						<span
							class="cal-badge font-ui"
							class:cal-loaded={calibrationStatus.existsOnDisk}
						>
							{calibrationStatus.existsOnDisk ? 'Loaded' : 'Default'}
						</span>
						<span class="cal-stat font-code">
							{totalSamples} samples ·
							v{calibrationStatus.version}
						</span>
					</div>
					<div class="cal-path font-code" title={calibrationStatus.path}>
						{calibrationStatus.path || '—'}
					</div>
				{/if}
				{#if calibrationError}
					<div class="error-text font-ui">{calibrationError}</div>
				{/if}
				<div class="cal-actions">
					<button
						class="permission-btn pixel-btn font-ui"
						onclick={reloadCalibrationProfile}
						disabled={calibrationBusy}
						title="Re-read guitar_calibration_profile.json from app_data_dir"
					>
						{calibrationBusy ? 'LOADING…' : 'RELOAD PROFILE'}
					</button>
				</div>
				<p class="cal-note font-ui">
					Use TUNE + CALIBRATE above for in-app tuning. Drop a
					CLI-generated <code>guitar_calibration_profile.json</code> into
					the path above and RELOAD to apply on next routing start.
				</p>
			</div>
		{/if}
	{/if}
</div>

<style>
	.input-panel {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 8px 10px 12px;
	}

	.source-radio {
		padding: 6px 8px;
	}

	.section-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		margin-bottom: 6px;
		letter-spacing: 1.5px;
		text-transform: uppercase;
	}

	.radio-row {
		display: flex;
		gap: 4px;
	}

	.source-btn {
		flex: 1;
		font-size: var(--font-size-xs) !important;
		padding: 5px 4px !important;
		text-align: center;
		color: var(--color-text-dim);
		background: var(--color-widget-inactive);
		border-color: var(--color-border);
	}

	.source-btn.active {
		color: var(--color-accent-cyan);
		border-color: var(--color-accent-cyan);
		background: var(--color-bg-panel);
		box-shadow: 0 0 4px var(--color-accent-cyan);
	}

	.source-btn.source-disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.midi-section {
		padding: 4px 6px;
	}

	.input-row {
		display: flex;
		gap: 4px;
		align-items: center;
	}

	.refresh-btn {
		padding: 3px 6px !important;
		font-size: var(--font-size-xs) !important;
		min-width: 20px;
		text-align: center;
	}

	.keyboard-hint {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-xs);
		margin-top: 4px;
		padding: 2px 4px;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
	}

	.error-text {
		color: #ff4466;
		font-size: var(--font-size-xs);
		margin-top: 3px;
	}

	.you-play-row {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: 4px;
	}

	.you-play-label {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		white-space: nowrap;
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
	}

	.permission-btn[disabled] {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.calibration-section {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-top: 4px;
	}

	.calibration-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.cal-badge {
		font-size: var(--font-size-xs);
		padding: 1px 6px;
		border: 1px solid var(--color-border);
		background: var(--color-widget-inactive);
		color: var(--color-text-dim);
		letter-spacing: 1px;
		text-transform: uppercase;
	}

	.cal-badge.cal-loaded {
		color: var(--color-accent-teal, #6affb8);
		border-color: var(--color-accent-teal, #6affb8);
	}

	.cal-stat {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
	}

	.cal-path {
		font-size: var(--font-size-xs);
		color: var(--color-text-dim);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.cal-actions {
		display: flex;
		gap: 6px;
	}

	.cal-note {
		font-size: var(--font-size-xs);
		color: var(--color-text-dim);
		margin: 0;
		line-height: 1.4;
	}

	.cal-note code {
		color: var(--color-text-secondary);
		background: var(--color-bg-deep);
		padding: 0 3px;
		border: 1px solid var(--color-border);
	}
</style>
