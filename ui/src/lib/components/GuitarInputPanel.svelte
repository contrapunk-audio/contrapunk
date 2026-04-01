<script lang="ts">
	import { onMount } from 'svelte';
	import { guitar } from '$lib/stores/guitar.svelte';
	import PixelSelect from './PixelSelect.svelte';

	const techniques = [
		{ key: 'bends' as const, label: 'BENDS', get active() { return guitar.bendsEnabled; } },
		{ key: 'legato' as const, label: 'LEGATO', get active() { return guitar.legatoEnabled; } },
		{ key: 'slides' as const, label: 'SLIDES', get active() { return guitar.slidesEnabled; } },
		{ key: 'vibrato' as const, label: 'VIBRATO', get active() { return guitar.vibratoEnabled; } },
	];

	let latencyDisplay = $derived(`${guitar.latencyMs}ms`);
	let gainDisplay = $derived(guitar.gain.toFixed(2));
	let confidenceDisplay = $derived(`${Math.round(guitar.stringConfidence * 100)}%`);

	let detectionLine = $derived(
		guitar.detecting
			? `Detecting: ${guitar.currentNote}  String: ${guitar.currentString} f${guitar.currentFret}  ${guitar.confidence}%`
			: 'No signal'
	);

	let deviceOptions = $derived(
		guitar.audioDevices.map((d) => ({
			value: d.deviceId,
			label: d.label || `Audio Device ${d.deviceId.slice(0, 8)}`
		}))
	);

	let channelNumbers = $derived(
		Array.from({ length: guitar.maxChannels }, (_, i) => i + 1)
	);

	function handleDeviceChange(value: string) {
		if (value) {
			guitar.selectDevice(value);
			guitar.syncDevice();
		}
	}

	function handleChannelChange(ch: number) {
		guitar.selectChannel(ch);
		guitar.syncDevice();
	}

	function handleTechniqueToggle(technique: 'bends' | 'legato' | 'slides' | 'vibrato') {
		guitar.toggleTechnique(technique);
		guitar.syncConfig();
	}

	/** Build a text-based cents meter for the tuner. */
	function buildMeter(cents: number): string {
		const width = 21;
		const center = Math.floor(width / 2);
		const pos = Math.round((cents / 50) * center + center);
		const clamped = Math.max(0, Math.min(width - 1, pos));
		const bar = '-'.repeat(width).split('');
		bar[center] = '|';
		if (Math.abs(cents) <= 5) {
			bar[clamped] = '*';
		} else {
			bar[clamped] = '#';
		}
		return '[' + bar.join('') + ']';
	}

	/** Build a hold progress bar. */
	function buildHoldBar(progress: number): string {
		const width = 10;
		const filled = Math.round(progress * width);
		return '\u2588'.repeat(filled) + '\u2591'.repeat(width - filled);
	}

	const OPEN_STRINGS = [
		{ name: 'Low E', note: 'E2', midi: 40, freq: 82.41 },
		{ name: 'A', note: 'A2', midi: 45, freq: 110.00 },
		{ name: 'D', note: 'D3', midi: 50, freq: 146.83 },
		{ name: 'G', note: 'G3', midi: 55, freq: 196.00 },
		{ name: 'B', note: 'B3', midi: 59, freq: 246.94 },
		{ name: 'High E', note: 'E4', midi: 64, freq: 329.63 },
	];

	let tunerTarget = $derived(OPEN_STRINGS[guitar.tunerStringIndex]);

	onMount(() => {
		guitar.enumerateAudioDevices();
		guitar.loadAudioDevices();
	});
</script>

{#if guitar.tunerActive}
	<!-- ============ TUNER UI ============ -->
	<div class="guitar-panel pixel-card tuner-panel">
		<div class="panel-header font-pixel">GUITAR INPUT</div>

		{#if guitar.tunerPhase === 'noise-floor'}
			<div class="tuner-section">
				<div class="tuner-title font-pixel">CALIBRATING</div>
				<div class="tuner-separator"></div>
				<div class="tuner-instruction font-pixel">Measuring noise floor...</div>
				<div class="tuner-instruction font-pixel">Keep quiet for 3 seconds</div>
				<div class="noise-progress-bar">
					<div class="noise-progress-fill" style:width="{guitar.tunerNoiseProgress * 100}%"></div>
				</div>
				<div class="tuner-actions">
					<button class="tuner-btn pixel-btn" onclick={() => guitar.cancelTuner()}>CANCEL</button>
				</div>
			</div>

		{:else if guitar.tunerPhase === 'tuning'}
			<div class="tuner-section">
				<div class="tuner-title font-pixel">TUNING -- String {guitar.tunerStringIndex + 1} of 6</div>
				<div class="tuner-separator"></div>

				<div class="tuner-target font-pixel">
					Pluck {tunerTarget.name} ({tunerTarget.note})
				</div>

				{#if guitar.tunerStatus === 'waiting'}
					<div class="tuner-detected font-pixel tuner-dim">Waiting for signal...</div>
					<div class="tuner-meter font-pixel tuner-dim">{buildMeter(0)}</div>
				{:else}
					<div class="tuner-detected font-pixel">
						<span class="tuner-note">{guitar.tunerDetectedNote}</span>
						<span class="tuner-cents" class:tuner-cents-good={Math.abs(guitar.tunerCents) <= 8} class:tuner-cents-bad={Math.abs(guitar.tunerCents) > 8}>
							{guitar.tunerCents >= 0 ? '+' : ''}{guitar.tunerCents} cents
						</span>
					</div>
					<div class="tuner-meter font-pixel" class:meter-intune={Math.abs(guitar.tunerCents) <= 8} class:meter-off={Math.abs(guitar.tunerCents) > 8}>
						{buildMeter(guitar.tunerCents)}
					</div>

					{#if guitar.tunerStatus === 'sharp'}
						<div class="tuner-advice font-pixel tuner-amber">SHARP -- tune down</div>
					{:else if guitar.tunerStatus === 'flat'}
						<div class="tuner-advice font-pixel tuner-amber">FLAT -- tune up</div>
					{:else if guitar.tunerStatus === 'holding' || guitar.tunerStatus === 'in-tune'}
						<div class="tuner-advice font-pixel tuner-cyan">IN TUNE</div>
						<div class="tuner-hold font-pixel tuner-cyan">
							{buildHoldBar(guitar.tunerHoldProgress)} holding...
						</div>
					{/if}
				{/if}

				<div class="tuner-actions">
					<button class="tuner-btn pixel-btn" onclick={() => guitar.skipTunerString()}>SKIP</button>
					<button class="tuner-btn pixel-btn" onclick={() => guitar.cancelTuner()}>CANCEL</button>
				</div>
			</div>

		{:else if guitar.tunerPhase === 'complete'}
			<div class="tuner-section">
				<div class="tuner-title font-pixel tuner-cyan">TUNING COMPLETE</div>
				<div class="tuner-separator"></div>
				<div class="tuner-instruction font-pixel tuner-cyan">All 6 strings tuned!</div>
				<div class="tuner-actions">
					<button class="tuner-btn pixel-btn" onclick={() => { guitar.tunerActive = false; }}>CLOSE</button>
				</div>
			</div>
		{/if}
	</div>

{:else}
	<!-- ============ NORMAL PANEL ============ -->
	<div class="guitar-panel pixel-card">
		<div class="panel-header font-pixel">GUITAR INPUT</div>

		<!-- Audio device + channel selector -->
		<div class="device-section">
			<span class="device-label font-pixel">DEVICE</span>
			{#if guitar.audioDeviceError}
				<div class="device-error font-pixel">{guitar.audioDeviceError}</div>
			{:else}
				<div class="device-select-row">
					<PixelSelect
						options={deviceOptions}
						value={guitar.selectedDeviceId}
						placeholder="Select device..."
						small={true}
						onchange={handleDeviceChange}
					/>
				</div>
				<div class="channel-header">
					<span class="device-label font-pixel channel-label">CHANNEL</span>
					<input
						type="number"
						class="channel-max-input"
						min="1"
						max="32"
						value={guitar.maxChannels}
						onchange={(e) => { guitar.maxChannels = Math.max(1, Math.min(32, parseInt(e.currentTarget.value) || 2)); }}
						title="Set max channels (browsers report 2, Audient iD14 has 12)"
					/>
				</div>
				<div class="channel-row">
					{#each channelNumbers as ch}
						<button
							class="channel-btn font-pixel"
							class:channel-active={guitar.selectedChannel === ch}
							onclick={() => handleChannelChange(ch)}
						>
							{ch}
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Interactive Dials row -->
		<div class="dials-row">
			<div class="dial-container">
				<div class="dial dial-cyan">
					<button class="dial-arrow dial-up font-pixel" onclick={() => guitar.setLatency(guitar.latencyMs + 1)} aria-label="Increase latency">+</button>
					<span class="dial-value">{latencyDisplay}</span>
					<button class="dial-arrow dial-down font-pixel" onclick={() => guitar.setLatency(guitar.latencyMs - 1)} aria-label="Decrease latency">-</button>
				</div>
				<span class="dial-label font-pixel">LATENCY</span>
			</div>

			<div class="dial-container">
				<div class="dial dial-amber">
					<button class="dial-arrow dial-up font-pixel" onclick={() => guitar.setGain(guitar.gain + 0.05)} aria-label="Increase gain">+</button>
					<span class="dial-value">{gainDisplay}</span>
					<button class="dial-arrow dial-down font-pixel" onclick={() => guitar.setGain(guitar.gain - 0.05)} aria-label="Decrease gain">-</button>
				</div>
				<span class="dial-label font-pixel">GAIN</span>
			</div>

			<div class="dial-container">
				<div class="dial dial-teal">
					<button class="dial-arrow dial-up font-pixel" onclick={() => guitar.setStringConfidence(guitar.stringConfidence + 0.05)} aria-label="Increase confidence">+</button>
					<span class="dial-value">{confidenceDisplay}</span>
					<button class="dial-arrow dial-down font-pixel" onclick={() => guitar.setStringConfidence(guitar.stringConfidence - 0.05)} aria-label="Decrease confidence">-</button>
				</div>
				<span class="dial-label font-pixel">STRING</span>
			</div>
		</div>

		<!-- Technique toggles -->
		<div class="techniques-row">
			{#each techniques as tech}
				<button
					class="technique-btn pixel-btn"
					class:technique-active={tech.active}
					onclick={() => handleTechniqueToggle(tech.key)}
				>
					{tech.label}
				</button>
			{/each}
		</div>

		<!-- Tune + Calibrate button -->
		<button
			class="calibrate-btn pixel-btn"
			class:calibrating={guitar.calibrating}
			disabled={guitar.calibrating}
			onclick={() => guitar.startCalibration()}
		>
			{guitar.calibrating ? 'CALIBRATING...' : guitar.calibrated ? 'CALIBRATED' : 'TUNE + CALIBRATE'}
		</button>
		{#if guitar.calibrationStatus}
			<div class="calibration-status font-pixel" class:calibrated={guitar.calibrated}>
				{guitar.calibrationStatus}
			</div>
		{/if}

		<!-- Live detection status -->
		<div class="detection-status font-pixel" class:detecting={guitar.detecting}>
			{detectionLine}
		</div>
	</div>
{/if}

<style>
	.guitar-panel {
		padding: 6px;
		margin-bottom: 4px;
	}

	.panel-header {
		color: var(--color-accent-teal);
		font-size: var(--font-size-xs);
		margin-bottom: 6px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	/* === Device + Channel selector === */
	.device-section {
		margin-bottom: 6px;
		padding-bottom: 6px;
		border-bottom: 1px solid var(--color-border);
	}

	.device-label {
		display: block;
		font-size: 6px;
		color: var(--color-text-secondary);
		margin-bottom: 3px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.channel-label {
		margin-top: 4px;
	}

	.device-select-row {
		display: flex;
		gap: 4px;
		align-items: center;
	}

	.device-error {
		font-size: 6px;
		color: var(--color-text-dim);
		padding: 3px 4px;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.channel-row {
		display: flex;
		flex-wrap: wrap;
		gap: 2px;
	}

	.channel-header {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.channel-max-input {
		width: 36px;
		height: 18px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		color: var(--color-accent-cyan);
		font-family: var(--font-pixel);
		font-size: 8px;
		text-align: center;
		padding: 0;
	}
	.channel-btn {
		width: 18px;
		height: 18px;
		padding: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 6px;
		color: var(--color-text-dim);
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		border-radius: 0;
		cursor: pointer;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.channel-btn:hover {
		border-color: var(--color-accent-cyan);
		color: var(--color-text-primary);
	}

	.channel-btn.channel-active {
		background: var(--color-accent-cyan);
		color: var(--color-bg-deep);
		border-color: var(--color-accent-cyan);
	}

	/* === Interactive Dials === */
	.dials-row {
		display: flex;
		justify-content: space-between;
		gap: 4px;
		margin-bottom: 6px;
	}

	.dial-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		flex: 1;
	}

	.dial {
		width: 44px;
		height: 56px;
		border-radius: 22px;
		border: 2px solid var(--color-border);
		background: var(--color-widget-bg);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		margin-bottom: 3px;
		gap: 1px;
	}

	.dial-cyan {
		border-color: var(--color-accent-cyan);
	}

	.dial-amber {
		border-color: var(--color-accent-amber);
	}

	.dial-teal {
		border-color: var(--color-accent-teal);
	}

	.dial-value {
		font-family: var(--font-reading);
		font-size: 10px;
		color: var(--color-text-primary);
		text-align: center;
		line-height: 1;
		pointer-events: none;
	}

	.dial-arrow {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 14px;
		padding: 0;
		font-size: 8px;
		line-height: 1;
		color: var(--color-text-dim);
		background: transparent;
		border: none;
		cursor: pointer;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
		border-radius: 0;
	}

	.dial-arrow:hover {
		color: var(--color-text-primary);
	}

	.dial-cyan .dial-arrow:hover {
		color: var(--color-accent-cyan);
	}

	.dial-amber .dial-arrow:hover {
		color: var(--color-accent-amber);
	}

	.dial-teal .dial-arrow:hover {
		color: var(--color-accent-teal);
	}

	.dial-arrow:active {
		transform: scale(0.9);
	}

	.dial-label {
		font-size: 6px;
		color: var(--color-text-secondary);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	/* === Technique toggles === */
	.techniques-row {
		display: flex;
		gap: 3px;
		margin-bottom: 6px;
	}

	.technique-btn {
		flex: 1;
		font-size: 6px !important;
		padding: 3px 2px !important;
		text-align: center;
		color: var(--color-text-dim);
		background: var(--color-widget-inactive);
		border-color: var(--color-border);
	}

	.technique-btn.technique-active {
		color: var(--color-accent-cyan);
		border-color: var(--color-accent-cyan-dim);
		background: var(--color-bg-panel);
	}

	/* === Calibrate button === */
	.calibrate-btn {
		width: 100%;
		font-size: 7px !important;
		padding: 5px 8px !important;
		text-align: center;
		color: var(--color-accent-teal);
		border-color: var(--color-accent-teal);
		background: var(--color-widget-inactive);
		margin-bottom: 6px;
	}

	.calibrate-btn:hover {
		background: var(--color-bg-panel);
		border-color: var(--color-accent-teal);
	}

	.calibrate-btn.calibrating {
		opacity: 0.6;
		cursor: wait;
	}

	.calibration-status {
		font-size: 6px;
		color: var(--color-text-dim);
		padding: 2px 4px;
		margin-bottom: 4px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.calibration-status.calibrated {
		color: var(--color-accent-teal);
	}

	/* === Detection status === */
	.detection-status {
		font-size: 6px;
		color: var(--color-text-dim);
		padding: 3px 4px;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.detection-status.detecting {
		color: var(--color-text-secondary);
	}

	/* =============================== */
	/* ========= TUNER STYLES ======== */
	/* =============================== */

	.tuner-panel {
		min-height: 180px;
	}

	.tuner-section {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.tuner-title {
		font-size: 7px;
		color: var(--color-text-primary);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.tuner-separator {
		height: 1px;
		background: var(--color-border);
	}

	.tuner-instruction {
		font-size: 6px;
		color: var(--color-text-secondary);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.tuner-target {
		font-size: 8px;
		color: var(--color-accent-magenta);
		text-align: center;
		padding: 4px 0;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.tuner-detected {
		display: flex;
		justify-content: center;
		align-items: baseline;
		gap: 8px;
		font-size: 7px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.tuner-note {
		color: var(--color-accent-magenta);
		font-size: 10px;
	}

	.tuner-cents {
		font-size: 7px;
	}

	.tuner-cents-good {
		color: var(--color-accent-cyan);
	}

	.tuner-cents-bad {
		color: var(--color-accent-amber);
	}

	.tuner-meter {
		text-align: center;
		font-size: 8px;
		letter-spacing: 1px;
		color: var(--color-text-secondary);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.tuner-meter.meter-intune {
		color: var(--color-accent-cyan);
	}

	.tuner-meter.meter-off {
		color: var(--color-accent-amber);
	}

	.tuner-advice {
		text-align: center;
		font-size: 7px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.tuner-hold {
		text-align: center;
		font-size: 7px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.tuner-dim {
		color: var(--color-text-dim);
	}

	.tuner-cyan {
		color: var(--color-accent-cyan);
	}

	.tuner-amber {
		color: var(--color-accent-amber);
	}

	.noise-progress-bar {
		height: 6px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		overflow: hidden;
	}

	.noise-progress-fill {
		height: 100%;
		background: var(--color-accent-teal);
		transition: width 50ms linear;
	}

	.tuner-actions {
		display: flex;
		gap: 4px;
		justify-content: center;
		margin-top: 4px;
	}

	.tuner-btn {
		font-size: 6px !important;
		padding: 3px 8px !important;
		color: var(--color-text-secondary);
		border-color: var(--color-border);
	}

	.tuner-btn:hover {
		color: var(--color-text-primary);
		border-color: var(--color-accent-cyan);
	}
</style>
