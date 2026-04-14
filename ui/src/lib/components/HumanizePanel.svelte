<script lang="ts">
	import { adapter, platformName } from '$lib/adapter';
	import type { HumanizeState } from '$lib/adapter';

	// Humanization is now available in all runtimes that implement the
	// adapter (native/Tauri, WASM browser, and — as a no-op — plugin).
	// The plugin DAW provides its own humanization so we still hide the
	// UI there to avoid double-processing.
	let isPluginMode = $state(platformName === 'plugin');

	// Local reactive state for humanize config
	let enabled = $state(false);
	let jitterMs = $state(10);
	let velocityVariation = $state(15);
	let durationMs = $state(20);
	let swingAmount = $state(0);
	let bpm = $state(120);
	let metronomeEnabled = $state(false);

	// Sync from backend on mount
	$effect(() => {
		loadHumanizeState();
	});

	async function loadHumanizeState() {
		try {
			const state = await adapter.getHumanizeState();
			enabled = state.enabled;
			jitterMs = state.jitterMaxMs;
			velocityVariation = state.velocityVariation;
			durationMs = state.durationVariationMs;
			swingAmount = Math.round(state.swingAmount * 100);
			bpm = state.bpm;
			metronomeEnabled = state.metronomeEnabled;
		} catch {
			// Adapter not ready yet — use defaults
		}
	}

	async function pushConfig() {
		try {
			await adapter.setHumanizeConfig({
				enabled,
				jitterEnabled: enabled,
				jitterMinMs: 1,
				jitterMaxMs: jitterMs,
				velocityEnabled: enabled,
				velocityVariation,
				durationEnabled: enabled,
				durationVariationMs: durationMs,
				swingEnabled: enabled && swingAmount > 0,
				swingAmount: swingAmount / 100,
				bpm,
				metronomeEnabled,
			});
		} catch {
			// Adapter call failed — UI state stays as-is
		}
	}

	function toggleEnabled() {
		enabled = !enabled;
		pushConfig();
	}

	function toggleMetronome() {
		metronomeEnabled = !metronomeEnabled;
		pushConfig();
	}

	function onJitterChange(e: Event) {
		jitterMs = Number((e.target as HTMLInputElement).value);
		pushConfig();
	}

	function onVelocityChange(e: Event) {
		velocityVariation = Number((e.target as HTMLInputElement).value);
		pushConfig();
	}

	function onDurationChange(e: Event) {
		durationMs = Number((e.target as HTMLInputElement).value);
		pushConfig();
	}

	function onSwingChange(e: Event) {
		swingAmount = Number((e.target as HTMLInputElement).value);
		pushConfig();
	}

	function onBpmChange(e: Event) {
		bpm = Number((e.target as HTMLInputElement).value);
		pushConfig();
	}
</script>

<!-- Humanize Section -->
<div class="card">
	<div class="card-header font-pixel">Humanize</div>
	{#if isPluginMode}
		<div class="unavailable font-pixel">DAW handles this</div>
	{:else}
	<div class="toggle-row">
		<button
			class="pixel-btn toggle-btn"
			class:toggle-on={enabled}
			onclick={toggleEnabled}
		>
			{enabled ? 'ON' : 'OFF'}
		</button>
	</div>

	{#if enabled}
		<div class="sliders">
			<!-- Jitter -->
			<div class="slider-row">
				<span class="slider-label font-pixel">JITTER</span>
				<div class="slider-track-wrapper">
					<input
						type="range"
						min="0"
						max="30"
						step="1"
						value={jitterMs}
						oninput={onJitterChange}
						class="hld-range"
					/>
					<div class="slider-fill" style:width="{(jitterMs / 30) * 100}%"></div>
				</div>
				<span class="slider-value font-pixel">{jitterMs}ms</span>
			</div>

			<!-- Velocity -->
			<div class="slider-row">
				<span class="slider-label font-pixel">VELOCITY</span>
				<div class="slider-track-wrapper">
					<input
						type="range"
						min="0"
						max="20"
						step="1"
						value={velocityVariation}
						oninput={onVelocityChange}
						class="hld-range"
					/>
					<div class="slider-fill" style:width="{(velocityVariation / 20) * 100}%"></div>
				</div>
				<span class="slider-value font-pixel">{velocityVariation}</span>
			</div>

			<!-- Duration -->
			<div class="slider-row">
				<span class="slider-label font-pixel">DURATION</span>
				<div class="slider-track-wrapper">
					<input
						type="range"
						min="0"
						max="50"
						step="1"
						value={durationMs}
						oninput={onDurationChange}
						class="hld-range"
					/>
					<div class="slider-fill" style:width="{(durationMs / 50) * 100}%"></div>
				</div>
				<span class="slider-value font-pixel">{durationMs}ms</span>
			</div>

			<!-- Swing -->
			<div class="slider-row">
				<span class="slider-label font-pixel">SWING</span>
				<div class="slider-track-wrapper">
					<input
						type="range"
						min="0"
						max="100"
						step="1"
						value={swingAmount}
						oninput={onSwingChange}
						class="hld-range"
					/>
					<div class="slider-fill" style:width="{swingAmount}%"></div>
				</div>
				<span class="slider-value font-pixel">{swingAmount}%</span>
			</div>
		</div>
	{/if}
	{/if}
</div>

{#if !isPluginMode}
<!-- Tempo Section -->
<div class="card">
	<div class="card-header font-pixel">Tempo</div>

	<div class="tempo-row">
		<div class="bpm-display font-pixel">{bpm}</div>
		<span class="bpm-unit font-pixel">BPM</span>
	</div>

	<div class="slider-row">
		<div class="slider-track-wrapper">
			<input
				type="range"
				min="40"
				max="240"
				step="1"
				value={bpm}
				oninput={onBpmChange}
				class="hld-range"
			/>
			<div class="slider-fill" style:width="{((bpm - 40) / 200) * 100}%"></div>
		</div>
	</div>

	<div class="toggle-row metro-row">
		<button
			class="pixel-btn toggle-btn"
			class:toggle-on={metronomeEnabled}
			onclick={toggleMetronome}
		>
			{metronomeEnabled ? 'ON' : 'OFF'}
		</button>
		<span class="metro-label font-pixel">METRO</span>
		{#if metronomeEnabled}
			<div class="beat-dot"></div>
		{/if}
	</div>
</div>
{/if}

<style>
	.card {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		padding: 6px;
		margin-bottom: 4px;
		border-radius: 0;
	}

	.unavailable {
		color: var(--color-text-dim);
		font-size: 6px;
		padding: 2px 0;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.card-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		margin-bottom: 4px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.toggle-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.toggle-btn {
		min-width: 40px;
	}

	.toggle-btn.toggle-on {
		background: var(--color-accent-teal);
		border-color: var(--color-accent-cyan);
		box-shadow: var(--glow-teal);
		color: #ffffff;
	}

	/* === HLD Health Bar Sliders === */
	.sliders {
		margin-top: 6px;
		display: flex;
		flex-direction: column;
		gap: 5px;
	}

	.slider-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.slider-label {
		color: var(--color-text-secondary);
		font-size: 6px;
		min-width: 42px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.slider-value {
		color: var(--color-accent-cyan);
		font-size: 6px;
		min-width: 28px;
		text-align: right;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.slider-track-wrapper {
		flex: 1;
		position: relative;
		height: 8px;
		background: var(--color-widget-inactive);
		border: 1px solid var(--color-border);
	}

	.slider-fill {
		position: absolute;
		top: 0;
		left: 0;
		height: 100%;
		background: linear-gradient(90deg, var(--color-accent-cyan-dim), var(--color-accent-cyan));
		box-shadow: 0 0 4px var(--color-accent-cyan), inset 0 0 2px rgba(51, 221, 255, 0.3);
		pointer-events: none;
		z-index: 1;
	}

	.hld-range {
		position: relative;
		z-index: 2;
		width: 100%;
		height: 100%;
		-webkit-appearance: none;
		appearance: none;
		background: transparent;
		margin: 0;
		cursor: pointer;
	}

	.hld-range::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 6px;
		height: 12px;
		background: var(--color-accent-magenta);
		border: 1px solid var(--color-accent-magenta-dim);
		border-radius: 0;
		cursor: pointer;
		box-shadow: 0 0 4px var(--color-accent-magenta);
	}

	.hld-range::-moz-range-thumb {
		width: 6px;
		height: 12px;
		background: var(--color-accent-magenta);
		border: 1px solid var(--color-accent-magenta-dim);
		border-radius: 0;
		cursor: pointer;
		box-shadow: 0 0 4px var(--color-accent-magenta);
	}

	/* === Tempo Section === */
	.tempo-row {
		display: flex;
		align-items: baseline;
		gap: 4px;
		margin-bottom: 4px;
	}

	.bpm-display {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-lg);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.bpm-unit {
		color: var(--color-text-dim);
		font-size: 6px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.metro-row {
		margin-top: 6px;
	}

	.metro-label {
		color: var(--color-text-secondary);
		font-size: 6px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.beat-dot {
		width: 6px;
		height: 6px;
		background: var(--color-accent-teal);
		box-shadow: 0 0 6px var(--color-accent-teal);
		animation: beat-pulse 0.5s ease-in-out infinite;
	}

	@keyframes beat-pulse {
		0%, 100% { opacity: 0.4; }
		50% { opacity: 1; }
	}
</style>
