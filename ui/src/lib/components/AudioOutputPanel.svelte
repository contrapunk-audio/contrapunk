<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { platformName } from '$lib/adapter';

	// Audio output is desktop-only (cpal). Hide toggle in browser/plugin.
	let isDesktop = $state(platformName === 'tauri');

	async function toggle(e: Event) {
		const checked = (e.currentTarget as HTMLInputElement).checked;
		try {
			await engine.setAudioOut(checked);
		} catch {
			// Leave UI as-is on error — backend will reject if not available
		}
	}
</script>

<!-- Audio Output (dev toggle — proper Routing tab in sub-project 4) -->
<div class="card">
	<div class="card-header font-pixel">Audio Output</div>

	{#if !isDesktop}
		<div class="unavailable font-pixel">Desktop only</div>
	{:else}
		<div class="hint font-pixel">
			Route harmony voices to built-in sine synth. Replaces IAC + external DAW for basic monitoring.
		</div>
		<div class="toggle-row">
			<input
				type="checkbox"
				class="pixel-checkbox"
				checked={engine.audioOutRunning}
				onchange={toggle}
			/>
			<span class="toggle-label font-pixel">{engine.audioOutRunning ? 'ON' : 'OFF'}</span>
		</div>
	{/if}
</div>

<style>
	.card {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		padding: 6px;
		margin-bottom: 4px;
		border-radius: 0;
	}

	.card-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		margin-bottom: 4px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.unavailable {
		color: var(--color-text-dim);
		font-size: 6px;
		padding: 2px 0;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.hint {
		color: var(--color-text-secondary);
		font-size: 6px;
		margin-bottom: 6px;
		line-height: 1.6;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.toggle-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.pixel-checkbox {
		appearance: none;
		-webkit-appearance: none;
		width: 12px;
		height: 12px;
		border: 1px solid var(--color-border);
		background: var(--color-widget-inactive);
		cursor: pointer;
		flex-shrink: 0;
		position: relative;
	}

	.pixel-checkbox:checked {
		background: var(--color-accent-teal);
		border-color: var(--color-accent-cyan);
		box-shadow: var(--glow-teal);
	}

	.pixel-checkbox:checked::after {
		content: '';
		position: absolute;
		inset: 2px;
		background: var(--color-accent-cyan);
	}

	.toggle-label {
		color: var(--color-text-secondary);
		font-size: 6px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}
</style>
