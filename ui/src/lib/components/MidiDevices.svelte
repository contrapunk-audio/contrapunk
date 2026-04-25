<script lang="ts">
	import { onMount } from 'svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { platformName } from '$lib/adapter';
	import PixelSelect from './PixelSelect.svelte';
	import type { Snippet } from 'svelte';

	let { children }: { children?: Snippet } = $props();

	// Browser-only permission state. Tauri / Plugin paths report `'granted'`
	// from the adapter so this whole block disappears in those builds.
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

	// Virtual input sentinel values — must match src-tauri/src/commands/engine.rs
	const VIRTUAL_COMPUTER_KEYBOARD = 999_998;
	const VIRTUAL_GUITAR_AUDIO = 999_997;

	// Sentinel strings used as <select> values for the non-MIDI-device
	// slot options. MIDI device options use their numeric device index
	// stringified. We pick values that can never collide with an int.
	const SYNTH_VALUE = '__synth__';
	const OFF_VALUE = '__off__';

	// Auto-refresh on mount if device lists are empty (defensive — page init should
	// already call midi.refresh(), but this ensures devices show even if that failed).
	// Also hydrate per-voice routing from localStorage + adapter.
	onMount(() => {
		if (midi.inputs.length === 0 && midi.outputs.length === 0 && !midi.isLoading) {
			midi.refresh();
		}
		midi.hydrateVoiceOutputs();
	});

	// Slot count is driven by the configured voice count (1..8) so the
	// UI only shows slots that actually produce sound.
	const slotCount = $derived(Math.max(1, Math.min(engine.voiceCount, 8)));

	// Voice-count picker lives next to the OUTPUTS header (was in
	// ControlPanel; consolidated here so the count is visible alongside
	// the per-voice routing it controls).
	const voiceCountOptions = [1, 2, 3, 4].map((c) => ({
		value: String(c),
		label: c === 1 ? '1 voice' : `${c} voices`
	}));

	function onVoiceCountChange(value: string) {
		const n = parseInt(value, 10);
		if (Number.isFinite(n)) {
			engine.setVoiceCount(n);
		}
	}

	// Toggle: when ON every voice is forced to Internal Synth, when OFF
	// every voice reverts to UseDefault (legacy fan-out). State derived
	// from the routing table so it stays in sync if individual slots
	// are tweaked manually.
	const allToSynth = $derived(
		Array.from({ length: slotCount }, (_, i) => midi.voiceOutputs[i]).every(
			(t) => t?.kind === 'synth'
		)
	);

	function toggleAllToSynth() {
		// On — every voice forced to Synth.
		// Off — every voice cleared to Off so the per-voice slots
		// re-appear and the user can pick MIDI destinations from a
		// blank slate. (No "use default" anymore — three explicit
		// destinations only.)
		const target: 'synth' | 'off' = allToSynth ? 'off' : 'synth';
		for (let i = 0; i < slotCount; i++) {
			midi.setVoiceOutput(i, { kind: target });
		}
	}

	// Derived: is Computer Keyboard selected as input?
	let isComputerKeyboard = $derived(midi.selectedInput === VIRTUAL_COMPUTER_KEYBOARD);

	let inputOptions = $derived([
		...midi.inputs.map((d) => ({ value: String(d.index), label: d.name })),
		{ value: String(VIRTUAL_COMPUTER_KEYBOARD), label: 'Computer Keyboard' },
		{ value: String(VIRTUAL_GUITAR_AUDIO), label: 'Guitar Audio' }
	]);

	// Per-slot dropdown options: Internal Synth first (sensible default for
	// no-MIDI users), then each available MIDI device, then Off last.
	let outputOptions = $derived([
		{ value: SYNTH_VALUE, label: 'Internal Synth' },
		...midi.outputs.map((d) => ({ value: String(d.index), label: d.name })),
		{ value: OFF_VALUE, label: 'Off' }
	]);

	function handleInputChange(value: string) {
		if (value === '') {
			midi.clearInput();
		} else {
			const idx = parseInt(value, 10);
			if (idx === VIRTUAL_COMPUTER_KEYBOARD || idx === VIRTUAL_GUITAR_AUDIO) {
				midi.selectVirtualInput(idx);
			} else {
				midi.selectInput(idx);
			}
		}
	}

	function handleOutputChange(slotIndex: number, value: string) {
		if (value === SYNTH_VALUE) {
			midi.setVoiceOutput(slotIndex, { kind: 'synth' });
			return;
		}
		if (value === OFF_VALUE) {
			midi.setVoiceOutput(slotIndex, { kind: 'off' });
			return;
		}
		// MIDI device picked. Make sure the device is in the router's
		// port pool (selectedOutputs) and route this voice directly to
		// that port via MidiPort. No more positional / use_default
		// indirection — voice_outputs is the only routing source now.
		const deviceIdx = parseInt(value, 10);
		if (Number.isNaN(deviceIdx)) return;
		const existingPort = midi.selectedOutputs.indexOf(deviceIdx);
		const port =
			existingPort >= 0
				? existingPort
				: (midi.selectedOutputs = [...midi.selectedOutputs, deviceIdx]).length - 1;
		midi.setVoiceOutput(slotIndex, { kind: 'midi_port', port });
	}

	function getSlotValue(slotIndex: number): string {
		const t = midi.voiceOutputs[slotIndex];
		if (!t || t.kind === 'synth') return SYNTH_VALUE;
		if (t.kind === 'off') return OFF_VALUE;
		// MidiPort → resolve back to the absolute device index so the
		// dropdown highlights the correct device.
		if (t.kind === 'midi_port') {
			const dev = midi.selectedOutputs[t.port];
			if (typeof dev === 'number') return String(dev);
		}
		return SYNTH_VALUE;
	}

	function slotLabel(index: number): string {
		if (index === 0) return 'Voice 1 (melody)';
		return `Voice ${index + 1}`;
	}
</script>

{#if showPermissionCard}
	<!-- Web MIDI permission card. Browsers gate
	     `navigator.requestMIDIAccess()` behind a user gesture, so on first
	     visit (and any time permission is denied or unsupported) the user
	     needs an explicit affordance to trigger the prompt. -->
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
				MIDI permission denied. Open your browser's site settings to allow it,
				then try again.
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
				Your browser doesn't support Web MIDI. Use Chrome, Edge, or Opera —
				or run Contrapunk natively on the desktop:
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

<!-- Input Device Section -->
<div class="midi-section pixel-card">
	<div class="section-header font-ui">INPUT</div>

	<div class="input-row">
		<PixelSelect
			options={inputOptions}
			value={midi.selectedInput !== null ? String(midi.selectedInput) : ''}
			placeholder="Select..."
			onchange={handleInputChange}
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
		<div class="keyboard-hint font-code">Z-M: C3-B3, Q-U: C4-C5</div>
	{/if}

	{#if midi.error}
		<div class="error-text font-ui">{midi.error}</div>
	{/if}
</div>

<!-- Slot for Guitar Input panel (rendered between INPUT and OUTPUTS) -->
{@render children?.()}

<!-- Output Device Section -->
<div class="midi-section pixel-card">
	<div class="output-header-row">
		<span class="section-header font-ui">OUTPUTS</span>
		<div class="voice-count-control" title="Number of voices the engine generates">
			<PixelSelect
				options={voiceCountOptions}
				value={String(engine.voiceCount)}
				small={true}
				onchange={onVoiceCountChange}
			/>
		</div>
	</div>

	<label class="route-all-toggle font-ui" title="Route every voice to the internal synth (skip external MIDI)">
		<input
			type="checkbox"
			checked={allToSynth}
			onchange={toggleAllToSynth}
		/>
		<span class="route-all-label">All → Internal Synth</span>
	</label>

	{#if !allToSynth}
		<div class="output-slots">
			{#each Array.from({ length: slotCount }, (_, i) => i) as slotIdx}
				<div class="output-slot">
					<span class="slot-label font-ui">{slotLabel(slotIdx)}</span>
					<PixelSelect
						options={outputOptions}
						value={getSlotValue(slotIdx)}
						placeholder="None"
						small={true}
						onchange={(val) => handleOutputChange(slotIdx, val)}
					/>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.midi-section {
		padding: 6px;
		margin-bottom: 4px;
	}

	.section-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		margin-bottom: 4px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	/* OUTPUTS header gets the voice-count picker inline next to the
	   label so the count is visible alongside the per-voice slots. */
	.output-header-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 4px;
	}
	.output-header-row .section-header {
		margin-bottom: 0;
	}
	.voice-count-control {
		display: flex;
		align-items: center;
	}

	.route-all-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: var(--font-size-xs);
		padding: 3px 4px;
		margin-bottom: 4px;
		cursor: pointer;
		color: var(--color-text-secondary);
		user-select: none;
	}
	.route-all-toggle input[type='checkbox'] {
		margin: 0;
		accent-color: var(--color-accent-cyan);
		cursor: pointer;
	}
	.route-all-toggle:hover .route-all-label {
		color: var(--color-accent-cyan);
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
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.error-text {
		color: #ff4466;
		font-size: var(--font-size-xs);
		margin-top: 3px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.output-slots {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.output-slot {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.slot-label {
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		white-space: nowrap;
		min-width: 56px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
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
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
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
