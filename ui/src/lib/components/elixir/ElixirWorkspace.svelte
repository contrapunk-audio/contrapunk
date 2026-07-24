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

	async function panic() {
		await adapter.panicAllNotesOff();
		panicSent = true;
		window.setTimeout(() => (panicSent = false), 700);
	}
</script>

<main class="workspace" class:embedded>
	<header class="masthead">
		<div>
			<p class="kicker font-code">ELIXIR / SINE INSTRUMENT</p>
			<h1>One source.<br /><em>Exact by design.</em></h1>
		</div>
		<div class="status" class:live={ready && available} role="status">
			<span aria-hidden="true"></span>
			<div>
				<strong>{ready && available ? 'SINE CORE READY' : 'SINE CORE'}</strong>
				<small>16 voices · fixed 5 ms de-click</small>
			</div>
		</div>
	</header>

	{#if error}
		<div class="notice" role="alert">{error}</div>
	{:else if !available}
		<div class="notice">This host does not expose an audio-rendering Elixir instance.</div>
	{:else}
		<div class="workspace-grid">
			<OscillatorPanel roleAware={!masterOnly} />
			<section class="controls" aria-label="Elixir controls">
				<header>
					<div><p class="font-code">OUTPUT</p><h2>Level</h2></div>
					{#if !masterOnly}
						<button class:active={synth.enabled} onclick={() => synth.setEnabled(!synth.enabled)}>
							{synth.enabled ? 'ENABLED' : 'BYPASSED'}
						</button>
					{/if}
				</header>

				<label class="master">
					<span><b>Master Gain</b><output>{Math.round(synth.masterGain * 100)}%</output></span>
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
					<div class="role-mix">
						<p class="font-code">VOICE ROLES</p>
						{#each roles as role, index}
							<label>
								<span><b>{role}</b><output>{Math.round(synth.mixGains[index] * 100)}%</output></span>
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

				{#if !masterOnly}
					<button class="panic" class:confirmed={panicSent} disabled={!ready} onclick={panic}>
						{panicSent ? 'VOICES CLEARED' : 'PANIC / CLEAR VOICES'}
					</button>
				{/if}
			</section>
		</div>
	{/if}
</main>

<style>
	.workspace {
		height: 100%;
		overflow: auto;
		box-sizing: border-box;
		padding: clamp(24px, 5vw, 72px);
		background:
			radial-gradient(circle at 78% 12%, rgba(0, 204, 170, 0.09), transparent 24rem),
			linear-gradient(120deg, rgba(255, 51, 136, 0.04), transparent 45%);
	}
	.workspace.embedded { padding: clamp(18px, 3vw, 42px); }
	.masthead { display: flex; justify-content: space-between; align-items: flex-end; gap: 40px; max-width: 1180px; margin: 0 auto clamp(26px, 4vw, 48px); }
	.kicker { color: var(--color-accent-teal); font-size: var(--font-size-xs); letter-spacing: 0.2em; margin: 0 0 12px; }
	h1 { margin: 0; max-width: 720px; font: 600 clamp(32px, 5.8vw, 74px)/0.92 var(--font-display); letter-spacing: -0.055em; color: var(--color-text-primary); }
	h1 em { color: transparent; font-style: normal; -webkit-text-stroke: 1px var(--color-text-secondary); }
	.status { display: flex; align-items: center; gap: 11px; min-width: 230px; padding: 11px 13px; border: 1px solid rgba(255, 170, 51, 0.45); background: rgba(255, 170, 51, 0.07); }
	.status > span { width: 8px; height: 8px; background: var(--color-accent-amber); }
	.status.live { border-color: rgba(0, 204, 170, 0.5); background: rgba(0, 204, 170, 0.07); }
	.status.live > span { background: var(--color-accent-teal); box-shadow: 0 0 10px rgba(0, 204, 170, 0.6); }
	.status div { display: grid; gap: 2px; }
	.status strong { color: var(--color-accent-amber); font: 700 var(--font-size-xs) var(--font-code); letter-spacing: 0.14em; }
	.status.live strong { color: var(--color-accent-teal); }
	.status small { color: var(--color-text-secondary); font: var(--font-size-xs) var(--font-ui); }
	.workspace-grid { display: grid; grid-template-columns: minmax(0, 1.45fr) minmax(280px, .55fr); gap: 18px; max-width: 1180px; margin: 0 auto; align-items: stretch; }
	.controls { display: flex; flex-direction: column; min-width: 0; border: 1px solid var(--color-border); background: rgba(10, 10, 26, 0.88); }
	.controls > header { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 16px 18px; border-bottom: 1px solid var(--color-border); }
	.controls header p, .role-mix > p { margin: 0 0 4px; color: var(--color-accent-magenta); font-size: var(--font-size-xs); letter-spacing: .14em; }
	.controls h2 { margin: 0; font-size: var(--font-size-lg); }
	.controls button { border: 1px solid var(--color-border); background: transparent; color: var(--color-text-secondary); font: 700 var(--font-size-xs) var(--font-code); letter-spacing: .08em; }
	.controls header button { min-height: 32px; padding: 0 10px; }
	.controls button.active { border-color: var(--color-accent-teal); color: var(--color-accent-teal); }
	.master, .role-mix label { display: grid; gap: 10px; padding: 18px; border-bottom: 1px solid var(--color-border); }
	.master span, .role-mix label span { display: flex; justify-content: space-between; gap: 16px; color: var(--color-text-secondary); font: var(--font-size-sm) var(--font-ui); }
	.master output, .role-mix output { color: var(--color-accent-cyan); font: var(--font-size-xs) var(--font-code); }
	.controls input[type='range'] { width: 100%; accent-color: var(--color-accent-cyan); }
	.controls input:disabled { opacity: .35; }
	.role-mix { padding-top: 16px; }
	.role-mix > p { padding: 0 18px 10px; }
	.role-mix label { padding-block: 12px; }
	.panic { min-height: 42px; margin: auto 18px 18px; }
	.panic:hover, .panic.confirmed { border-color: var(--color-accent-magenta); color: var(--color-accent-magenta); }
	.notice { max-width: 1180px; margin: 0 auto; padding: 18px; border: 1px solid var(--color-border); color: var(--color-text-secondary); }
	@media (max-width: 900px) {
		.masthead { align-items: flex-start; flex-direction: column; }
		.workspace-grid { grid-template-columns: 1fr; }
	}
</style>
