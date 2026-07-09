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

	type Source = 'midi' | 'guitar' | 'voice' | 'none';

	/** Map the (potentially null) midi.selectedInput onto a Source. A
	 *  null selection now reports 'none' so the radio doesn't light up
	 *  the MIDI tile when nothing is wired (brutal-critic #3). */
	const sourceFromSelection = (sel: number | null): Source => {
		if (sel === null) return 'none';
		if (sel === VIRTUAL_GUITAR_AUDIO) return 'guitar';
		return 'midi';
	};

	let source = $derived<Source>(sourceFromSelection(midi.selectedInput));

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
		voiceUnavailableHint = false;
		if (next === 'guitar') {
			await switchInput(() => midi.selectVirtualInput(VIRTUAL_GUITAR_AUDIO));
			return;
		}
		if (source === 'midi') {
			await midi.refresh();
			return;
		}
		await switchInput(() => {
			let restored = false;
			if (lastMidiSelection !== null) {
				midi.selectInput(lastMidiSelection);
				restored = midi.selectedInput === lastMidiSelection;
			}
			if (!restored) {
				midi.selectVirtualInput(VIRTUAL_COMPUTER_KEYBOARD);
			}
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

	let isComputerKeyboard = $derived(midi.selectedInput === VIRTUAL_COMPUTER_KEYBOARD);

	// MIDI source — only "real" MIDI devices + Computer Keyboard.
	// Guitar Audio is hoisted out into the top-level source radio.
	let midiInputOptions = $derived([
		...midi.inputs.map((d) => ({ value: String(d.index), label: d.name })),
		{ value: String(VIRTUAL_COMPUTER_KEYBOARD), label: 'Computer Keyboard' }
	]);

	async function handleMidiInputChange(value: string) {
		await switchInput(() => {
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
	<!-- Source radio: MIDI / Guitar Audio / Voice (disabled). Replaces
	     the inline Guitar Audio option in the MIDI input dropdown so the
	     source choice is a top-level concern. Voice is reserved for the
	     v1.4 vocal pipeline (PSOLA harmonizer). -->
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
				title="MIDI keyboard or computer-keyboard input"
			>
				MIDI
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
