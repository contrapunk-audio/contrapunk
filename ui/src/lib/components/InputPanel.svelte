<script lang="ts">
	import { onMount } from 'svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { adapter } from '$lib/adapter';
	import type { CalibrationStatus } from '$lib/adapter/types';
	import PixelSelect from './PixelSelect.svelte';
	import GuitarInputPanel from './GuitarInputPanel.svelte';
	import MidiPermissionCard from './MidiPermissionCard.svelte';

	// Virtual input sentinel values — must match
	// src-tauri/src/commands/engine.rs.
	const VIRTUAL_COMPUTER_KEYBOARD = 999_998;
	const VIRTUAL_GUITAR_AUDIO = 999_997;

	type Source = 'midi' | 'keyboard' | 'guitar' | 'voice' | 'none';

	/** Keep hardware MIDI and computer typing distinct. Treating both as
	 *  one "MIDI" source made the virtual keyboard look like the only
	 *  available device while hardware discovery was still running. */
	const sourceFromSelection = (sel: number | null): Source => {
		if (sel === null) return 'none';
		if (sel === VIRTUAL_GUITAR_AUDIO) return 'guitar';
		if (sel === VIRTUAL_COMPUTER_KEYBOARD) return 'keyboard';
		return 'midi';
	};

	let requestedSource = $state<Source | null>(null);
	let source = $derived<Source>(requestedSource ?? sourceFromSelection(midi.selectedInput));

	// Remember the user's last PHYSICAL MIDI device so clicking the MIDI
	// radio after a guitar session restores it, instead of either
	// auto-selecting Computer Keyboard (hot keyboard mid-take) or
	// dropping into Computer-Keyboard as "last MIDI device" (brutal-
	// critic #6 — VCK is not a physical MIDI device).
	function isPhysicalMidi(sel: number | null): boolean {
		return (
			sel !== null &&
			sel !== VIRTUAL_GUITAR_AUDIO &&
			sel !== VIRTUAL_COMPUTER_KEYBOARD
		);
	}
	let lastMidiSelection = $state<number | null>(
		isPhysicalMidi(midi.selectedInput) ? midi.selectedInput : null
	);
	$effect(() => {
		if (isPhysicalMidi(midi.selectedInput)) {
			lastMidiSelection = midi.selectedInput;
		}
	});

	// Transient feedback when the user clicks the disabled Voice tile.
	// Clears on any other source click.
	let voiceUnavailableHint = $state(false);

	async function switchInput(select: () => void) {
		const restart = engine.isRunning;
		midi.error = null;
		try {
			if (restart) await engine.stop();
			select();
			if (restart && midi.selectedInput !== null) {
				await engine.start(midi.selectedInput, midi.selectedOutputs);
			}
		} catch (e) {
			midi.error = `Failed to switch input: ${e}`;
		}
	}

	async function selectSource(next: Source) {
		if (next === 'none') return;
		if (next === 'voice') {
			voiceUnavailableHint = true;
			return;
		}
		requestedSource = next;
		voiceUnavailableHint = false;
		if (next === 'guitar') {
			await switchInput(() => midi.selectVirtualInput(VIRTUAL_GUITAR_AUDIO));
			return;
		}
		if (next === 'keyboard') {
			await switchInput(() => midi.selectVirtualInput(VIRTUAL_COMPUTER_KEYBOARD));
			return;
		}
		await midi.refresh();
		if (source === 'midi' && isPhysicalMidi(midi.selectedInput)) return;
		await switchInput(() => {
			const restored =
				lastMidiSelection !== null &&
				midi.inputs.some((device) => device.index === lastMidiSelection);
			const device = restored
				? lastMidiSelection
				: (midi.inputs[0]?.index ?? null);
			if (device === null) midi.clearInput();
			else midi.selectInput(device);
		});
	}

	// Calibration profile status (Tauri-only — gated by capability flag).
	let calibrationStatus = $state<CalibrationStatus | null>(null);
	let calibrationError = $state<string | null>(null);
	let calibrationBusy = $state(false);

	async function refreshCalibrationStatus() {
		if (!adapter.capabilities.calibrationFlow) return;
		try {
			calibrationStatus = await adapter.getCalibrationStatus();
			calibrationError = null;
		} catch (e) {
			calibrationError = `Status check failed: ${e}`;
		}
	}

	async function reloadCalibrationProfile() {
		// Defense in depth: even though the UI gates the button on the
		// capability, route through the same check so any future call
		// site (programmatic, dev console, future surface) can't slip
		// past the capability contract.
		if (!adapter.capabilities.calibrationFlow) return;
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

	async function resetCalibrationToDefault() {
		if (!adapter.capabilities.calibrationFlow) return;
		if (calibrationBusy) return;
		// Inline confirm: a real bad calibration is destructive to undo
		// (the file is gone). Use a native confirm for now; a follow-up
		// commit can promote this to an inline toggle-confirm.
		const ok =
			typeof window !== 'undefined' &&
			window.confirm(
				'Delete the saved calibration profile and reset to defaults? This cannot be undone.'
			);
		if (!ok) return;
		calibrationBusy = true;
		calibrationError = null;
		try {
			calibrationStatus = await adapter.deleteCalibrationProfile();
		} catch (e) {
			calibrationError = `Reset failed: ${e}`;
		} finally {
			calibrationBusy = false;
		}
	}

	const totalSamples = $derived(
		calibrationStatus
			? calibrationStatus.sampleCounts.reduce((a, b) => a + b, 0)
			: 0
	);

	/** Per-string sample distribution display, e.g. "12 8 7 9 6 5".
	 *  Surfaces under the total so users see whether their calibration
	 *  sweep covered every string (brutal-critic #5). */
	const sampleDistribution = $derived(
		calibrationStatus ? calibrationStatus.sampleCounts.join(' ') : ''
	);

	onMount(() => {
		if (midi.inputs.length === 0 && !midi.isLoading) {
			midi.refresh();
		}
		refreshCalibrationStatus();
		// Re-check on window focus so a user who drops a profile file in
		// from another app and tabs back sees the badge flip immediately
		// (brutal-critic #5).
		if (typeof window !== 'undefined') {
			const onFocus = () => refreshCalibrationStatus();
			window.addEventListener('focus', onFocus);
			return () => window.removeEventListener('focus', onFocus);
		}
	});

	// Hardware inputs only. Computer Keys has its own top-level source
	// choice so users never mistake it for the only discovered device.
	let midiInputOptions = $derived(
		midi.inputs.map((device) => ({ value: String(device.index), label: device.name }))
	);

	async function handleMidiInputChange(value: string) {
		requestedSource = 'midi';
		await switchInput(() => {
			if (value === '') {
				midi.clearInput();
				return;
			}
			midi.selectInput(parseInt(value, 10));
		});
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
	<!-- Source selection separates hardware MIDI from computer typing.
	     Voice remains reserved for the later vocal pipeline. -->
	<div class="source-radio pixel-card">
		<div class="section-header font-ui">SOURCE</div>
		<div class="radio-row" role="radiogroup" aria-label="Input source">
			<button
				class="source-btn pixel-btn font-ui"
				class:active={source === 'midi'}
				role="radio"
				type="button"
				aria-checked={source === 'midi'}
				onclick={() => selectSource('midi')}
				title="Use a connected hardware MIDI controller"
			>
				MIDI Controller
			</button>
			<button
				class="source-btn pixel-btn font-ui"
				class:active={source === 'keyboard'}
				role="radio"
				type="button"
				aria-checked={source === 'keyboard'}
				onclick={() => selectSource('keyboard')}
				title="Play notes with the computer keyboard"
			>
				Computer Keys
			</button>
			<button
				class="source-btn pixel-btn font-ui"
				class:active={source === 'guitar'}
				role="radio"
				type="button"
				aria-checked={source === 'guitar'}
				onclick={() => selectSource('guitar')}
				title="Live guitar audio in via cpal (pitch-detected)"
			>
				Guitar
			</button>
			<button
				class="source-btn pixel-btn font-ui source-disabled"
				role="radio"
				type="button"
				aria-checked="false"
				aria-disabled="true"
				onclick={() => selectSource('voice')}
				title="Voice input (PSOLA harmonizer) lands in v1.4"
			>
				Voice
			</button>
		</div>
		{#if voiceUnavailableHint}
			<p class="voice-hint font-ui">
				Voice input (PSOLA harmonizer) lands in v1.4 — stay tuned.
			</p>
		{/if}
	</div>

	<!-- Web MIDI permission card. Browsers gate
	     navigator.requestMIDIAccess() behind a user gesture. The
	     component renders nothing on Tauri / Plugin paths and includes
	     a "DOWNLOAD DESKTOP APP" link for the unsupported-browser
	     branch (Safari, Firefox without flag) — previously missing
	     from InputPanel's inline copy. -->
	<MidiPermissionCard />

	{#if source === 'midi'}
		<div class="midi-section pixel-card">
			<div class="section-header font-ui">MIDI CONTROLLER</div>
			<div class="input-row">
				<PixelSelect
					options={midiInputOptions}
					value={isPhysicalMidi(midi.selectedInput) ? String(midi.selectedInput) : ''}
					placeholder={midi.isLoading ? 'Scanning MIDI devices…' : 'Select controller…'}
					label="MIDI controller"
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
			{#if !midi.isLoading && midi.inputs.length === 0}
				<div class="device-status warning font-ui">
					No hardware MIDI controller found. Connect one, then press R to scan again.
				</div>
			{:else if !midi.isLoading}
				<div class="device-status font-ui">{midi.inputs.length} controller{midi.inputs.length === 1 ? '' : 's'} available</div>
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
						label="You play"
						small={true}
						onchange={onVoicePositionChange}
					/>
				</div>
			{/if}
		</div>
	{:else if source === 'keyboard'}
		<div class="midi-section pixel-card keyboard-source">
			<div class="section-header font-ui">COMPUTER KEYS</div>
			<p class="keyboard-copy font-ui">Play immediately from your typing keyboard.</p>
			<div class="keyboard-hint font-code">Z–M: C3–B3 · Q–U: C4–C5</div>
			{#if engine.voiceCount > 1}
				<div class="you-play-row">
					<span class="you-play-label font-ui">You play</span>
					<PixelSelect
						options={voicePositionOptions}
						value={String(engine.voicePosition)}
						placeholder="Voice"
						label="You play"
						small={true}
						onchange={onVoicePositionChange}
					/>
				</div>
			{/if}
		</div>
	{:else if source === 'guitar'}
		<GuitarInputPanel />

		{#if adapter.capabilities.calibrationFlow}
			<details class="midi-section pixel-card calibration-section">
				<summary class="profile-summary font-ui">
					<span>DETECTOR PROFILE</span>
					<span>{calibrationStatus?.existsOnDisk ? 'CUSTOM' : 'DEFAULT'}</span>
				</summary>
				<div class="calibration-body">
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
					<div class="cal-distribution font-code" title="Per-string sample counts (E2 A D G B E4)">
						strings [{sampleDistribution}]
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
						{calibrationBusy ? 'LOADING…' : 'RELOAD'}
					</button>
					<button
						class="permission-btn pixel-btn font-ui"
						onclick={resetCalibrationToDefault}
						disabled={calibrationBusy || !calibrationStatus?.existsOnDisk}
						title="Delete the saved profile and reset to defaults"
					>
						RESET
					</button>
				</div>
					<p class="cal-note font-ui">
						Use TUNE GUITAR above to tune the strings. Detector calibration is
						separate: drop a CLI-generated
						<code>guitar_calibration_profile.json</code> into the path above,
						then RELOAD to apply it.
					</p>
				</div>
			</details>
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
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
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

	.voice-hint {
		margin: 4px 0 0;
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		line-height: 1.4;
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

	.device-status,
	.keyboard-copy {
		margin: 4px 0 0;
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		line-height: 1.4;
	}

	.device-status.warning { color: var(--color-accent-gold); }

	.keyboard-source { padding: 8px; }

	.keyboard-hint {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-xs);
		margin-top: 4px;
		padding: 5px 6px;
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
		margin-top: 4px;
	}

	.profile-summary {
		display: flex;
		justify-content: space-between;
		padding: 6px 8px;
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		cursor: pointer;
	}

	.calibration-body {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 4px 6px 6px;
		border-top: 1px solid var(--color-border);
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

	.cal-distribution {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		letter-spacing: 1px;
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
