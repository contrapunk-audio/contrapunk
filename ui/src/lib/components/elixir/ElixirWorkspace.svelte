<script lang="ts">
	import { onMount } from 'svelte';
	import { adapter } from '$lib/adapter';
	import { synth } from '$lib/stores/synth.svelte';
	import { CHAPTER_EXAMPLES, cloneRolePatch } from '$lib/elixir/patch';
	import SlidePanel from '$lib/components/SlidePanel.svelte';
	import OscillatorPanel from './OscillatorPanel.svelte';

	let { embedded = false, masterOnly = false } = $props<{
		embedded?: boolean;
		masterOnly?: boolean;
	}>();

	const roles = ['Input', 'Harmony', 'Canon', 'Counterpoint'];
	let selectedRole = $state(0);
	let selectedExample = $state('');
	let ready = $state(false);
	let error = $state('');
	let available = $derived(masterOnly || adapter.capabilities.audioFx);

	onMount(() => {
		let cancelled = false;
		void (async () => {
			try {
				await adapter.init();
				await synth.syncFromBackend();
				const linkedExample = new URLSearchParams(window.location.search).get('elixir-example');
				if (linkedExample && CHAPTER_EXAMPLES.some((example) => example.id === linkedExample)) {
					await applyExample(linkedExample, false);
				}
				if (!cancelled) ready = true;
			} catch (cause) {
				if (!cancelled) error = cause instanceof Error ? cause.message : 'Synth unavailable';
			}
		})();
		const unsubscribe = embedded
			? () => {}
			: adapter.onPluginParamsUpdate(() => void synth.syncFromBackend());
		return () => {
			cancelled = true;
			unsubscribe();
		};
	});

	function formatGain(gain: number) {
		return gain <= 0.0001 ? '−∞ dB' : `${(20 * Math.log10(gain)).toFixed(1)} dB`;
	}

	async function applyExample(id: string, persist = true) {
		selectedExample = id;
		const example = CHAPTER_EXAMPLES.find((candidate) => candidate.id === id);
		if (!example) return;
		await synth.setAllRolePatches(example.patches.map(cloneRolePatch), persist);
	}
</script>

<main class="workspace" class:embedded>
	<section class="synth-panel">
		<header class="titlebar">
			<strong class="product">ELIXIR</strong>
			<div class="summary font-code">
				<span>6 HARMONICS × 16 VOICES</span>
				<span class:online={ready && available}>{ready && available ? 'READY' : 'OFFLINE'}</span>
			</div>
			{#if !masterOnly}
				<div class="actions">
					<button
						class:enabled={synth.enabled}
						aria-pressed={synth.enabled}
						disabled={!ready || !available}
						onclick={() => void synth.setEnabled(!synth.enabled)}
					>
						{synth.enabled ? 'Enabled' : 'Bypassed'}
					</button>
				</div>
			{/if}
		</header>

		{#if error}
			<div class="notice" role="alert">{error}</div>
		{:else if !available}
			<div class="notice">This host does not provide an audio output for the synth.</div>
		{:else}
			<div class="instrument-toolbar">
				{#if !masterOnly}
					<div class="role-tabs" role="tablist" aria-label="Elixir role patch">
						{#each roles as role, index}
							<button type="button" role="tab" class:active={selectedRole === index} aria-selected={selectedRole === index} onclick={() => (selectedRole = index)}>{role}</button>
						{/each}
					</div>
				{/if}
				<label class="example-picker"><span>Chapter example</span><select value={selectedExample} onchange={(event) => void applyExample(event.currentTarget.value)}><option value="">Choose a recording-ready state</option>{#each CHAPTER_EXAMPLES as example}<option value={example.id}>Ch {example.chapter} · {example.name}</option>{/each}</select></label>
			</div>
			{#if selectedExample}
				{@const example = CHAPTER_EXAMPLES.find((candidate) => candidate.id === selectedExample)}
				{#if example}<p class="example-description"><strong>{example.name}.</strong> {example.description}</p>{/if}
			{/if}
			<div class="main-grid">
				<OscillatorPanel role={masterOnly ? 0 : selectedRole} disabled={!ready || (!masterOnly && !synth.enabled)} />

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
								oninput={(event) => void synth.setMasterGain(Number(event.currentTarget.value))}
							/>
						</label>

						{#if !masterOnly && adapter.capabilities.roleMix}
							<div class="role-section">
								<h3>Roles</h3>
								{#each roles as role, index}
									<div class="gain-row">
										<span><b>{role}</b><output>{formatGain(synth.mixGains[index])}</output></span>
										<div class="role-controls">
											<input
												aria-label={`${role} level`}
												type="range"
												min="0"
												max="1"
												step="0.01"
												value={synth.mixGains[index]}
												disabled={!ready || !synth.enabled}
												oninput={(event) => void synth.setMixGain(index, Number(event.currentTarget.value))}
											/>
											<button class:on={synth.muted[index]} type="button" aria-label={`Mute ${role}`} aria-pressed={synth.muted[index]} disabled={!ready || !synth.enabled} onclick={() => void synth.toggleMute(index)}>{synth.muted[index] ? 'M✓' : 'M'}</button>
											<button class:on={synth.solo === index} type="button" aria-label={`Solo ${role}`} aria-pressed={synth.solo === index} disabled={!ready || !synth.enabled} onclick={() => void synth.toggleSolo(index)}>{synth.solo === index ? 'S✓' : 'S'}</button>
										</div>
									</div>
								{/each}
								{#if synth.mixError}<p class="mix-error" role="alert">{synth.mixError}</p>{/if}
							</div>
						{/if}
					</div>
				</section>
			</div>
			{#if !masterOnly}
				<section class="slide-instrument" aria-label="Continuous pitch controls">
					<header><span>CHAPTER 2</span><div><h2>Continuous pitch</h2><p>Slide moves pitch after harmony and tuning. It remains shared with Arrangement so internal audio and routed MIDI follow one path.</p></div></header>
					<SlidePanel />
				</section>
			{/if}
			{#if synth.patchError}<p class="patch-error" role="alert">{synth.patchError}</p>{/if}
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
	button.enabled, button.on { border-color: #668563; color: #b7d0b3; }
	button:disabled { opacity: .45; }
	.instrument-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 9px 10px 0;
	}
	.role-tabs { display: flex; gap: 3px; }
	.role-tabs button { min-width: 78px; }
	.role-tabs button.active { border-color: #8aaac0; background: #26333b; color: #e0e9ee; }
	.example-picker { display: grid; grid-template-columns: auto minmax(210px, 280px); align-items: center; gap: 8px; color: #8b8b8b; font: 9px var(--font-code); text-transform: uppercase; letter-spacing: .06em; }
	.example-picker select { height: 29px; border: 1px solid #484848; background: #1c1c1c; color: #ddd; font: 10px var(--font-ui); text-transform: none; letter-spacing: 0; }
	.example-description { margin: 7px 10px 0; padding: 7px 9px; border-left: 2px solid #8aaac0; background: #1b2226; color: #9da8ae; font: 10px/1.45 var(--font-ui); }
	.example-description strong { color: #d5e0e5; }
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
	.gain-row input { width: 100%; min-width: 0; accent-color: #7c9eb8; }
	.gain-row input:disabled { opacity: .35; }
	.gain-row.master { padding-top: 2px; }
	.role-controls { display: grid; grid-template-columns: minmax(0, 1fr) 30px 30px; align-items: center; gap: 5px; }
	.role-controls button { width: 30px; height: 26px; padding: 0; font: 700 10px var(--font-code); }
	.mix-error { margin: 8px 0 0; color: #e0aaaa; font: 10px/1.4 var(--font-code); }
	.role-section { margin-top: 10px; padding-top: 12px; border-top: 1px solid #3b3b3b; }
	.role-section h3 { margin-bottom: 5px; color: #888; text-transform: uppercase; letter-spacing: .06em; }
	.slide-instrument { margin: 0 10px 10px; border: 1px solid #3b3b3b; background: #202020; }
	.slide-instrument > header { display: flex; align-items: start; gap: 10px; padding: 10px 12px; border-bottom: 1px solid #333; }
	.slide-instrument > header > span { color: #7f9eb2; font: 700 8px var(--font-code); letter-spacing: .11em; }
	.slide-instrument > header h2 { margin: 0; }
	.slide-instrument > header p { margin: 3px 0 0; color: #858585; font: 10px/1.4 var(--font-ui); }
	.slide-instrument :global(.slide-panel) { margin: 0; border: 0; }
	.patch-error { margin: 0 10px 10px; padding: 8px; border: 1px solid #704848; color: #e0aaaa; font: 10px var(--font-ui); }
	.notice { margin: 10px; padding: 14px; border: 1px solid #484848; background: #242424; color: #aaa; font: 12px var(--font-ui); }
	@media (max-width: 760px) {
		.titlebar { grid-template-columns: 1fr auto; }
		.summary { display: none; }
		.instrument-toolbar { align-items: stretch; flex-direction: column; }
		.role-tabs { overflow-x: auto; }
		.example-picker { grid-template-columns: 1fr; }
		.main-grid { grid-template-columns: 1fr; }
		.synth-panel { min-height: 0; }
	}
</style>
