<script lang="ts">
	import { onMount } from 'svelte';
	import { adapter } from '$lib/adapter';
	import { synth } from '$lib/stores/synth.svelte';
	import OscillatorPanel from './OscillatorPanel.svelte';

	let { embedded = false, masterOnly = false } = $props<{
		embedded?: boolean;
		masterOnly?: boolean;
	}>();

	const roles = ['Input', 'Harmony', 'Canon', 'Counterpoint'];
	let ready = $state(false);
	let error = $state('');
	let panicSent = $state(false);
	let available = $derived(masterOnly || adapter.capabilities.audioFx);

	onMount(() => {
		let cancelled = false;
		void (async () => {
			try {
				await adapter.init();
				await synth.syncFromBackend();
				if (!cancelled) ready = true;
			} catch (cause) {
				if (!cancelled) error = cause instanceof Error ? cause.message : 'Synth unavailable';
			}
		})();
		const unsubscribe = adapter.onPluginParamsUpdate(() => void synth.syncFromBackend());
		return () => {
			cancelled = true;
			unsubscribe();
		};
	});

	function formatGain(gain: number) {
		return gain <= 0.0001 ? '−∞ dB' : `${(20 * Math.log10(gain)).toFixed(1)} dB`;
	}

	async function panic() {
		await adapter.panicAllNotesOff();
		panicSent = true;
		window.setTimeout(() => (panicSent = false), 700);
	}
</script>

<main class="workspace" class:embedded>
	<section class="synth-panel">
		<header class="titlebar">
			<strong class="product">ELIXIR</strong>
			<div class="summary font-code">
				<span>16 VOICES</span>
				<span class:online={ready && available}>{ready && available ? 'READY' : 'OFFLINE'}</span>
			</div>
			{#if !masterOnly}
				<div class="actions">
					<button
						class:enabled={synth.enabled}
						aria-pressed={synth.enabled}
						disabled={!ready || !available}
						onclick={() => synth.setEnabled(!synth.enabled)}
					>
						{synth.enabled ? 'Enabled' : 'Bypassed'}
					</button>
					<button class="panic" class:confirmed={panicSent} disabled={!ready} onclick={panic}>
						{panicSent ? 'Cleared' : 'Panic'}
					</button>
				</div>
			{/if}
		</header>

		{#if error}
			<div class="notice" role="alert">{error}</div>
		{:else if !available}
			<div class="notice">This host does not provide an audio output for the synth.</div>
		{:else}
			<div class="main-grid">
				<OscillatorPanel />

				<section class="mixer" aria-label="Output mixer">
					<header><h2>Output</h2></header>
					<div class="mixer-controls">
						<label class="gain-row master">
							<span><b>Master</b><output>{formatGain(synth.masterGain)}</output></span>
							<input
								type="range"
								min="0"
								max="1"
								step="0.01"
								value={synth.masterGain}
								disabled={!ready || (!masterOnly && !synth.enabled)}
								oninput={(event) => synth.setMasterGain(Number(event.currentTarget.value))}
							/>
						</label>

						{#if !masterOnly && adapter.capabilities.roleMix}
							<div class="role-section">
								<h3>Roles</h3>
								{#each roles as role, index}
									<label class="gain-row">
										<span><b>{role}</b><output>{formatGain(synth.mixGains[index])}</output></span>
										<input
											type="range"
											min="0"
											max="1"
											step="0.01"
											value={synth.mixGains[index]}
											disabled={!ready || !synth.enabled}
											oninput={(event) => synth.setMixGain(index, Number(event.currentTarget.value))}
										/>
									</label>
								{/each}
							</div>
						{/if}
					</div>
				</section>
			</div>
		{/if}

	</section>
</main>

<style>
	.workspace {
		height: 100%;
		overflow: auto;
		box-sizing: border-box;
		padding: 24px;
		background: #181818;
		color: #d8d8d8;
	}
	.workspace.embedded { padding: 14px; }
	.synth-panel {
		width: min(1040px, 100%);
		min-height: 560px;
		margin: 0 auto;
		border: 1px solid #3a3a3a;
		background: #202020;
		display: flex;
		flex-direction: column;
	}
	.titlebar {
		min-height: 52px;
		display: grid;
		grid-template-columns: 1fr auto auto;
		align-items: center;
		gap: 18px;
		padding: 0 14px;
		border-bottom: 1px solid #3a3a3a;
		background: #282828;
	}
	.product { font: 700 15px var(--font-ui); letter-spacing: .08em; color: #f0f0f0; }
	.summary { display: flex; gap: 12px; color: #888; font-size: 10px; }
	.summary .online { color: #8fb584; }
	.actions { display: flex; gap: 6px; }
	button {
		height: 30px;
		padding: 0 11px;
		border: 1px solid #4a4a4a;
		background: #303030;
		color: #bbb;
		font: 12px var(--font-ui);
	}
	button:hover:not(:disabled) { background: #393939; color: #eee; }
	button.enabled { border-color: #668563; color: #b7d0b3; }
	button.panic:hover:not(:disabled), button.confirmed { border-color: #9b6666; color: #e0aaaa; }
	button:disabled { opacity: .45; }
	.main-grid {
		display: grid;
		grid-template-columns: minmax(0, 1.65fr) minmax(260px, .75fr);
		gap: 10px;
		padding: 10px;
		flex: 1;
	}
	.mixer { border: 1px solid #3b3b3b; background: #242424; min-width: 0; }
	.mixer > header { height: 38px; display: flex; align-items: center; padding: 0 12px; border-bottom: 1px solid #3b3b3b; background: #292929; }
	h2, h3 { margin: 0; font: 600 12px var(--font-ui); color: #d0d0d0; }
	h2 { text-transform: uppercase; letter-spacing: .06em; }
	.mixer-controls { padding: 12px; }
	.gain-row { display: grid; gap: 7px; padding: 9px 0; }
	.gain-row span { display: flex; justify-content: space-between; gap: 12px; }
	.gain-row b { font: 500 12px var(--font-ui); color: #bbb; }
	.gain-row output { font: 10px var(--font-code); color: #aaa; }
	.gain-row input { width: 100%; accent-color: #7c9eb8; }
	.gain-row input:disabled { opacity: .35; }
	.gain-row.master { padding-top: 2px; }
	.role-section { margin-top: 10px; padding-top: 12px; border-top: 1px solid #3b3b3b; }
	.role-section h3 { margin-bottom: 5px; color: #888; text-transform: uppercase; letter-spacing: .06em; }
	.notice { margin: 10px; padding: 14px; border: 1px solid #484848; background: #242424; color: #aaa; font: 12px var(--font-ui); }
	@media (max-width: 760px) {
		.titlebar { grid-template-columns: 1fr auto; }
		.summary { display: none; }
		.main-grid { grid-template-columns: 1fr; }
		.synth-panel { min-height: 0; }
	}
</style>
