<script lang="ts">
	import { onMount } from 'svelte';
	import { synth, WAVEFORMS } from '$lib/stores/synth.svelte';
	import { delay, reverb } from '$lib/stores/fx.svelte';
	import { chainStore, parsePluginId } from '$lib/stores/chain.svelte';
	import { adapter } from '$lib/adapter';
	import Knob from './Knob.svelte';
	import WaveformView from './WaveformView.svelte';
	import EnvelopeView from './EnvelopeView.svelte';
	import PixelSelect from './PixelSelect.svelte';
	import ClapPluginPicker from './ClapPluginPicker.svelte';

	let waveformOptions = WAVEFORMS.map((w) => ({ value: String(w.value), label: w.label }));
	let pickerOpen = $state(false);

	function onWaveformChange(v: string) {
		synth.setWaveform(parseInt(v, 10));
	}

	onMount(() => {
		reverb.syncFromBackend();
		delay.syncFromBackend();
		chainStore.refresh();
	});
</script>

<div class="chain-wrap">
	<div class="chain-header-row">
		<div class="chain-header font-ui">Audio chain</div>
		<button class="pixel-btn font-ui add-plugin-btn" onclick={() => (pickerOpen = true)}>
			+ Plugin
		</button>
	</div>

	<!-- Signal-flow strip: dynamic based on chain state -->
	<div class="flow">
		<div class="flow-node source font-ui">
			<div class="flow-title">Harmony</div>
			<div class="flow-sub">MIDI</div>
		</div>
		{#each chainStore.blocks as b, i (i + ':' + b.typeId)}
			<div class="flow-arrow font-ui">━━▶</div>
			{#if b.typeId === 'builtin.synth'}
				<div class="flow-node synth-node font-ui">
					<div class="flow-title">Synth</div>
					<div class="flow-sub">8-voice</div>
				</div>
			{:else if b.typeId === 'builtin.delay'}
				<div class="flow-node delay-node font-ui" class:active={delay.enabled}>
					<div class="flow-title">Delay</div>
					<div class="flow-sub">{delay.enabled ? 'on' : 'bypass'}</div>
				</div>
			{:else if b.typeId === 'builtin.reverb'}
				<div class="flow-node reverb-node font-ui" class:active={reverb.enabled}>
					<div class="flow-title">Reverb</div>
					<div class="flow-sub">{reverb.enabled ? 'on' : 'bypass'}</div>
				</div>
			{:else}
				<!-- Plugin flow node -->
				<div class="flow-node plugin-node font-ui" title={b.typeId}>
					<div class="flow-title">{b.name}</div>
					<div class="flow-sub">CLAP</div>
				</div>
			{/if}
		{/each}
		<div class="flow-arrow font-ui">━━▶</div>
		<div class="flow-node sink font-ui">
			<div class="flow-title">Output</div>
			<div class="flow-sub">Speakers</div>
		</div>
	</div>

	{#if adapter.capabilities.audioFx}
	<!-- Synth rack: Serum-inspired -->
	<div class="rack">
		<div class="rack-header">
			<span class="rack-title font-ui">Synth</span>
			<button
				class="pixel-btn power-btn font-ui"
				class:active={synth.enabled}
				onclick={() => synth.setEnabled(!synth.enabled)}
				title={synth.enabled ? 'Bypass synth' : 'Enable synth'}
			>
				{synth.enabled ? 'ON' : 'OFF'}
			</button>
		</div>

		<div class="rack-grid">
			<!-- OSC section -->
			<section class="rack-section osc">
				<div class="section-label font-ui">Osc</div>
				<WaveformView waveform={synth.waveform} height={60} />
				<PixelSelect
					options={waveformOptions}
					value={String(synth.waveform)}
					placeholder="Wave"
					onchange={onWaveformChange}
				/>
			</section>

			<!-- FILTER section -->
			<section class="rack-section filter">
				<div class="section-label font-ui">Filter</div>
				<div class="knob-row">
					<Knob
						label="Cutoff"
						value={synth.cutoffHz}
						min={200}
						max={18000}
						step={50}
						defaultValue={6000}
						size={48}
						format={(v) => (v >= 1000 ? `${(v / 1000).toFixed(1)}k` : `${Math.round(v)}`)}
						accent="var(--color-accent-cyan)"
						onchange={(v) => synth.setCutoffHz(Math.round(v))}
					/>
					<Knob
						label="Reso"
						value={synth.resonance}
						min={0}
						max={1}
						step={0.01}
						defaultValue={0.2}
						size={48}
						format={(v) => `${Math.round(v * 100)}%`}
						accent="var(--color-accent-cyan)"
						onchange={(v) => synth.setResonance(v)}
					/>
				</div>
			</section>

			<!-- ENV section -->
			<section class="rack-section env">
				<div class="section-label font-ui">Env</div>
				<EnvelopeView
					attackMs={synth.attackMs}
					decayMs={synth.decayMs}
					sustain={synth.sustain}
					releaseMs={synth.releaseMs}
					height={60}
				/>
				<div class="knob-row">
					<Knob
						label="Attack"
						help="How quickly sound reaches full level after MIDI NoteOn."
						value={synth.attackMs}
						min={1}
						max={1500}
						step={1}
						defaultValue={5}
						size={44}
						format={(v) => `${Math.round(v)}ms`}
						accent="var(--color-accent-magenta)"
						onchange={(v) => synth.setAttackMs(Math.round(v))}
					/>
					<Knob
						label="Decay"
						help="Time to fall from the attack peak to the sustain level."
						value={synth.decayMs}
						min={1}
						max={2000}
						step={1}
						defaultValue={120}
						size={44}
						format={(v) => `${Math.round(v)}ms`}
						accent="var(--color-accent-magenta)"
						onchange={(v) => synth.setDecayMs(Math.round(v))}
					/>
					<Knob
						label="Sustain level"
						help="Held volume while Guitar Input keeps MIDI NoteOn active; it does not lengthen the note."
						value={synth.sustain}
						min={0}
						max={1}
						step={0.01}
						defaultValue={0.7}
						size={44}
						format={(v) => `${Math.round(v * 100)}%`}
						accent="var(--color-accent-magenta)"
						onchange={(v) => synth.setSustain(v)}
					/>
					<Knob
						label="Release tail"
						help="How long sound fades after Guitar Input emits MIDI NoteOff. Increase this for a longer tail."
						value={synth.releaseMs}
						min={1}
						max={4000}
						step={1}
						defaultValue={250}
						size={44}
						format={(v) => `${Math.round(v)}ms`}
						accent="var(--color-accent-magenta)"
						onchange={(v) => synth.setReleaseMs(Math.round(v))}
					/>
				</div>
			</section>

			<!-- AMP section -->
			<section class="rack-section amp">
				<div class="section-label font-ui">Amp</div>
				<div class="knob-row single">
					<Knob
						label="Master"
						value={synth.masterGain}
						min={0}
						max={1}
						step={0.01}
						defaultValue={0.25}
						format={(v) => `${Math.round(v * 100)}%`}
						accent="var(--color-accent-gold)"
						size={64}
						onchange={(v) => synth.setMasterGain(v)}
					/>
				</div>
			</section>
		</div>

		<div class="rack-footer font-ui">
			Drag knobs ↕ • Shift = fine • Double-click = reset
		</div>
	</div>

	<ClapPluginPicker bind:open={pickerOpen} />

	<!-- Delay rack -->
	<div class="rack delay-rack">
		<div class="rack-header">
			<span class="rack-title delay-title font-ui">Delay</span>
			<button
				class="pixel-btn power-btn font-ui"
				class:active={delay.enabled}
				onclick={() => delay.setEnabled(!delay.enabled)}
				title={delay.enabled ? 'Bypass delay' : 'Enable delay'}
			>
				{delay.enabled ? 'ON' : 'OFF'}
			</button>
		</div>

		{#if adapter.capabilities.delayTempoSync}
		<div class="delay-sync-row">
			<button
				class="pixel-btn sync-btn font-ui"
				class:active={delay.syncEnabled}
				onclick={() => delay.setSyncEnabled(!delay.syncEnabled)}
				title={delay.syncEnabled
					? 'Tap length is locked to transport BPM'
					: 'Free mode — tap length set in ms'}
			>
				SYNC {delay.syncEnabled ? 'ON' : 'OFF'}
			</button>
			<select
				class="sub-select font-code"
				value={delay.subdivision}
				disabled={!delay.syncEnabled}
				onchange={(e) =>
					delay.setSubdivision(
						(e.currentTarget as HTMLSelectElement).value as
							| '1/4'
							| '1/8d'
							| '1/8'
							| '1/8t'
							| '1/16'
							| '1/16t'
					)}
				title="Tempo-synced tap subdivision"
			>
				<option value="1/4">1/4</option>
				<option value="1/8d">1/8 dotted</option>
				<option value="1/8">1/8</option>
				<option value="1/8t">1/8 triplet</option>
				<option value="1/16">1/16</option>
				<option value="1/16t">1/16 triplet</option>
			</select>
		</div>
		{/if}

		<div class="reverb-knobs">
			<Knob
				label="Mix"
				value={delay.mix}
				min={0}
				max={1}
				step={0.01}
				defaultValue={0.3}
				size={56}
				format={(v) => `${Math.round(v * 100)}%`}
				accent="var(--color-accent-magenta)"
				onchange={(v) => delay.setMix(v)}
			/>
			<Knob
				label="Time"
				value={delay.timeMs}
				min={10}
				max={2000}
				step={1}
				defaultValue={375}
				size={56}
				format={(v) => `${Math.round(v)}ms`}
				accent="var(--color-accent-cyan)"
				disabled={delay.syncEnabled}
				onchange={(v) => delay.setTimeMs(Math.round(v))}
			/>
			<Knob
				label="Fbk"
				value={delay.feedback}
				min={0}
				max={0.95}
				step={0.01}
				defaultValue={0.35}
				size={56}
				format={(v) => `${Math.round(v * 100)}%`}
				accent="var(--color-accent-gold)"
				onchange={(v) => delay.setFeedback(v)}
			/>
		</div>
	</div>

	<!-- Reverb rack -->
	<div class="rack reverb-rack">
		<div class="rack-header">
			<span class="rack-title reverb-title font-ui">Reverb</span>
			<button
				class="pixel-btn power-btn font-ui"
				class:active={reverb.enabled}
				onclick={() => reverb.setEnabled(!reverb.enabled)}
				title={reverb.enabled ? 'Bypass reverb' : 'Enable reverb'}
			>
				{reverb.enabled ? 'ON' : 'OFF'}
			</button>
		</div>

		<div class="reverb-knobs">
			<Knob
				label="Mix"
				value={reverb.mix}
				min={0}
				max={1}
				step={0.01}
				defaultValue={0.3}
				size={56}
				format={(v) => `${Math.round(v * 100)}%`}
				accent="var(--color-accent-cyan)"
				onchange={(v) => reverb.setMix(v)}
			/>
			<Knob
				label="Room"
				value={reverb.roomSize}
				min={0}
				max={1}
				step={0.01}
				defaultValue={0.7}
				size={56}
				format={(v) => `${Math.round(v * 100)}%`}
				accent="var(--color-accent-magenta)"
				onchange={(v) => reverb.setRoomSize(v)}
			/>
			<Knob
				label="Damp"
				value={reverb.damping}
				min={0}
				max={1}
				step={0.01}
				defaultValue={0.5}
				size={56}
				format={(v) => `${Math.round(v * 100)}%`}
				accent="var(--color-accent-gold)"
				onchange={(v) => reverb.setDamping(v)}
			/>
		</div>
	</div>
	{/if}

	<!-- One rack per loaded CLAP plugin -->
	{#each chainStore.blocks as b, i (i + ':' + b.typeId)}
		{#if !b.typeId.startsWith('builtin.')}
			{@const pluginId = parsePluginId(b.typeId)}
			<div class="rack plugin-rack">
				<div class="rack-header">
					<span class="rack-title plugin-title font-ui">{b.name}</span>
					<div class="plugin-actions">
						<button
							class="pixel-btn font-ui plugin-open-btn"
							onclick={() => pluginId !== null && chainStore.openPluginGui(pluginId)}
							disabled={pluginId === null}
							title={pluginId === null ? 'Plugin has no id' : 'Open plugin window'}
						>
							Open UI
						</button>
						<button
							class="pixel-btn font-ui plugin-remove-btn"
							onclick={() => chainStore.removeAt(i)}
							title="Remove from chain"
						>
							Remove
						</button>
					</div>
				</div>
				<div class="plugin-body font-ui">
					<div class="plugin-row">
						<span class="plugin-label">Type</span>
						<span class="plugin-value">CLAP</span>
					</div>
					<div class="plugin-row">
						<span class="plugin-label">Plugin ID</span>
						<span class="plugin-value plugin-id font-code" title={b.typeId}>{b.typeId}</span>
					</div>
				</div>
			</div>
		{/if}
	{/each}
</div>

<style>
	.chain-wrap {
		height: 100%;
		overflow-y: auto;
		padding: 10px 14px 14px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.chain-header-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.chain-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		letter-spacing: 2px;
		text-transform: uppercase;
	}

	.add-plugin-btn {
		padding: 2px 10px !important;
		font-size: var(--font-size-xs) !important;
	}

	.blocks-strip {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		padding: 4px 8px;
		background: rgba(15, 14, 26, 0.4);
		border: 1px solid var(--color-border);
		min-height: 30px;
		align-items: center;
	}

	.blocks-empty {
		color: var(--color-text-dim);
		font-size: var(--font-size-xs);
		padding: 4px;
	}

	.block-chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 2px 8px;
		background: rgba(15, 14, 26, 0.7);
		border: 1px solid var(--color-border);
		max-width: 200px;
		overflow: hidden;
	}

	.block-chip:not(.builtin) {
		border-color: var(--color-accent-cyan-dim);
	}

	.block-name {
		color: var(--color-accent-magenta);
		font-size: var(--font-size-xs);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.block-chip:not(.builtin) .block-name {
		color: var(--color-accent-cyan);
	}

	.block-remove {
		background: transparent;
		border: 1px solid var(--color-border);
		color: var(--color-text-secondary);
		padding: 0 6px;
		cursor: pointer;
		font-size: var(--font-size-xs);
	}

	.block-remove:hover {
		color: rgb(255, 120, 120);
		border-color: rgba(200, 40, 40, 0.6);
	}

	/* Signal flow strip */
	.flow {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		flex-wrap: wrap;
		padding: 4px 8px;
		background: rgba(15, 14, 26, 0.6);
		border: 1px solid var(--color-border);
	}

	.flow-node {
		padding: 4px 10px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		text-align: center;
		min-width: 70px;
	}

	.flow-title {
		color: var(--color-accent-magenta);
		font-size: var(--font-size-xs);
		letter-spacing: 1px;
	}

	.flow-sub {
		color: var(--color-text-dim);
		font-size: var(--font-size-xs);
		margin-top: 1px;
	}

	.flow-node.source {
		border-color: var(--color-accent-cyan-dim);
	}
	.flow-node.source .flow-title {
		color: var(--color-accent-cyan);
	}

	.flow-node.synth-node {
		border-color: var(--color-accent-magenta-dim);
		box-shadow: var(--glow-magenta);
	}

	.flow-node.plugin-node {
		border-color: var(--color-accent-cyan-dim);
		background: rgba(0, 60, 90, 0.5);
		box-shadow: 0 0 10px rgba(0, 200, 255, 0.15);
	}
	.flow-node.plugin-node .flow-title {
		color: var(--color-accent-cyan);
	}

	.flow-node.reverb-node,
	.flow-node.delay-node {
		opacity: 0.55;
		border-style: dashed;
	}
	.flow-node.reverb-node.active {
		opacity: 1;
		border-style: solid;
		border-color: var(--color-accent-cyan-dim);
		box-shadow: 0 0 10px rgba(0, 200, 255, 0.18);
	}
	.flow-node.reverb-node.active .flow-title {
		color: var(--color-accent-cyan);
	}
	.flow-node.delay-node.active {
		opacity: 1;
		border-style: solid;
		border-color: var(--color-accent-magenta-dim);
		box-shadow: 0 0 10px rgba(255, 51, 136, 0.18);
	}
	.flow-node.delay-node.active .flow-title {
		color: var(--color-accent-magenta);
	}

	.flow-node.sink {
		border-color: var(--color-accent-amber);
	}
	.flow-node.sink .flow-title {
		color: var(--color-accent-amber);
	}

	.flow-arrow {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-sm);
		letter-spacing: -2px;
	}

	/* Synth rack */
	.rack {
		background: linear-gradient(180deg, #12101f, #0a0918);
		border: 1px solid var(--color-border);
		padding: 8px 10px 10px;
		box-shadow:
			inset 0 1px 0 rgba(255, 255, 255, 0.03),
			0 0 20px rgba(255, 51, 136, 0.06);
	}

	.rack-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-bottom: 6px;
		border-bottom: 1px solid var(--color-border);
		margin-bottom: 8px;
	}

	.rack-title {
		color: var(--color-accent-magenta);
		font-size: var(--font-size-sm);
		letter-spacing: 2px;
		text-transform: uppercase;
	}

	.power-btn {
		padding: 2px 10px !important;
		font-size: var(--font-size-xs) !important;
		min-width: 40px;
	}
	.power-btn.active {
		background: var(--color-accent-teal);
		border-color: var(--color-accent-cyan);
		box-shadow: var(--glow-teal);
		color: #ffffff;
	}

	.rack-grid {
		display: grid;
		grid-template-columns: 1.2fr 0.9fr 1.6fr 0.6fr;
		gap: 8px;
	}

	.rack-section {
		background: rgba(15, 14, 26, 0.5);
		border: 1px solid var(--color-border);
		padding: 6px 8px 8px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.section-label {
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		letter-spacing: 1.5px;
		text-transform: uppercase;
		padding-bottom: 2px;
		border-bottom: 1px solid var(--color-border);
	}

	.knob-row {
		display: flex;
		justify-content: space-around;
		align-items: flex-start;
		gap: 6px;
	}

	.knob-row.single {
		justify-content: center;
	}

	.rack-footer {
		margin-top: 6px;
		padding-top: 4px;
		border-top: 1px solid var(--color-border);
		color: var(--color-text-dim);
		font-size: var(--font-size-xs);
		letter-spacing: 0.5px;
		text-align: center;
	}

	/* Reverb rack */
	.reverb-rack {
		box-shadow:
			inset 0 1px 0 rgba(255, 255, 255, 0.03),
			0 0 20px rgba(0, 200, 255, 0.06);
	}

	.reverb-title {
		color: var(--color-accent-cyan);
	}

	.delay-rack {
		box-shadow:
			inset 0 1px 0 rgba(255, 255, 255, 0.03),
			0 0 20px rgba(255, 51, 136, 0.06);
	}

	.delay-title {
		color: var(--color-accent-magenta);
	}

	/* Plugin racks (one per loaded CLAP) */
	.plugin-rack {
		box-shadow:
			inset 0 1px 0 rgba(255, 255, 255, 0.03),
			0 0 20px rgba(0, 200, 255, 0.08);
		border-color: var(--color-accent-cyan-dim);
	}

	.plugin-title {
		color: var(--color-accent-cyan);
	}

	.plugin-actions {
		display: flex;
		gap: 6px;
	}

	.plugin-open-btn,
	.plugin-remove-btn {
		padding: 2px 10px !important;
		font-size: var(--font-size-xs) !important;
	}

	.plugin-open-btn[disabled] {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.plugin-remove-btn:hover {
		border-color: rgba(200, 40, 40, 0.6);
		color: rgb(255, 120, 120);
	}

	.plugin-body {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 6px 4px 2px;
	}

	.plugin-row {
		display: flex;
		gap: 8px;
		align-items: center;
		font-size: var(--font-size-xs);
	}

	.plugin-label {
		color: var(--color-text-secondary);
		min-width: 72px;
		letter-spacing: 1px;
		text-transform: uppercase;
	}

	.plugin-value {
		color: var(--color-text-primary);
	}

	.plugin-value.plugin-id {
		color: var(--color-text-dim);
		font-family: var(--font-code);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.plugin-note {
		color: var(--color-text-dim);
		font-size: var(--font-size-xs);
		padding-top: 4px;
		font-style: italic;
	}

	.reverb-knobs {
		display: flex;
		justify-content: space-around;
		align-items: flex-start;
		gap: 12px;
		padding: 6px 4px 2px;
	}

	.delay-sync-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 8px 0;
	}
	.sync-btn {
		font-size: var(--font-size-xs);
		padding: 2px 8px;
		border: 1px solid var(--color-border);
		background: var(--color-widget-bg);
		color: var(--color-text-dim);
		cursor: pointer;
	}
	.sync-btn.active {
		color: var(--color-accent-cyan);
		border-color: var(--color-accent-cyan);
		box-shadow: 0 0 4px var(--color-accent-cyan);
	}
	.sub-select {
		flex: 1;
		min-width: 0;
		padding: 2px 6px;
		border: 1px solid var(--color-border);
		background: var(--color-widget-bg);
		color: var(--color-text-primary);
		font-size: var(--font-size-xs);
	}
	.sub-select:disabled {
		opacity: 0.35;
	}

	/* Collapse rack-grid on narrow windows */
	@media (max-width: 980px) {
		.rack-grid {
			grid-template-columns: 1fr 1fr;
		}
	}
	@media (max-width: 600px) {
		.rack-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
