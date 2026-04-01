<script lang="ts">
	import { guitar } from '$lib/stores/guitar.svelte';

	const techniques = [
		{ key: 'bends' as const, label: 'BENDS', get active() { return guitar.bendsEnabled; } },
		{ key: 'legato' as const, label: 'LEGATO', get active() { return guitar.legatoEnabled; } },
		{ key: 'slides' as const, label: 'SLIDES', get active() { return guitar.slidesEnabled; } },
		{ key: 'vibrato' as const, label: 'VIBRATO', get active() { return guitar.vibratoEnabled; } },
	];

	let latencyDisplay = $derived(`${guitar.latencyMs}ms`);
	let gainDisplay = $derived(guitar.gain.toFixed(1));
	let confidenceDisplay = $derived(`${Math.round(guitar.stringConfidence * 100)}%`);

	let detectionLine = $derived(
		guitar.detecting
			? `Detecting: ${guitar.currentNote}  String: ${guitar.currentString} f${guitar.currentFret}  ${guitar.confidence}%`
			: 'No signal'
	);
</script>

<div class="guitar-panel pixel-card">
	<div class="panel-header font-pixel">GUITAR INPUT</div>

	<!-- Dials row -->
	<div class="dials-row">
		<div class="dial-container">
			<div class="dial dial-cyan">
				<span class="dial-value">{latencyDisplay}</span>
			</div>
			<span class="dial-label font-pixel">LATENCY</span>
		</div>

		<div class="dial-container">
			<div class="dial dial-amber">
				<span class="dial-value">{gainDisplay}</span>
			</div>
			<span class="dial-label font-pixel">GAIN</span>
		</div>

		<div class="dial-container">
			<div class="dial dial-teal">
				<span class="dial-value">{confidenceDisplay}</span>
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
				onclick={() => guitar.toggleTechnique(tech.key)}
			>
				{tech.label}
			</button>
		{/each}
	</div>

	<!-- Tune + Calibrate button -->
	<button
		class="calibrate-btn pixel-btn"
		onclick={() => guitar.startCalibration()}
	>
		TUNE + CALIBRATE
	</button>

	<!-- Live detection status -->
	<div class="detection-status font-pixel" class:detecting={guitar.detecting}>
		{detectionLine}
	</div>
</div>

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

	/* === Dials === */
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
		height: 44px;
		border-radius: 50%;
		border: 2px solid var(--color-border);
		background: var(--color-widget-bg);
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 3px;
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
</style>
