<script lang="ts">
	import { synth, WAVEFORMS } from '$lib/stores/synth.svelte';
	import Knob from './Knob.svelte';
	import WaveformView from './WaveformView.svelte';
	import EnvelopeView from './EnvelopeView.svelte';
	import PixelSelect from './PixelSelect.svelte';

	let waveformOptions = WAVEFORMS.map((w) => ({ value: String(w.value), label: w.label }));

	function onWaveformChange(v: string) {
		synth.setWaveform(parseInt(v, 10));
	}
</script>

<div class="chain-wrap">
	<div class="chain-header font-pixel">Audio chain</div>

	<!-- Signal-flow strip -->
	<div class="flow">
		<div class="flow-node source font-pixel">
			<div class="flow-title">Harmony</div>
			<div class="flow-sub">MIDI</div>
		</div>
		<div class="flow-arrow font-pixel">━━▶</div>
		<div class="flow-node synth-node font-pixel">
			<div class="flow-title">Synth</div>
			<div class="flow-sub">8-voice</div>
		</div>
		<div class="flow-arrow font-pixel">━━▶</div>
		<div class="flow-node fx-slot font-pixel">
			<div class="flow-title">FX</div>
			<div class="flow-sub">(coming soon)</div>
		</div>
		<div class="flow-arrow font-pixel">━━▶</div>
		<div class="flow-node sink font-pixel">
			<div class="flow-title">Output</div>
			<div class="flow-sub">Speakers</div>
		</div>
	</div>

	<!-- Synth rack: Serum-inspired -->
	<div class="rack">
		<div class="rack-header">
			<span class="rack-title font-pixel">Synth</span>
			<button
				class="pixel-btn power-btn font-pixel"
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
				<div class="section-label font-pixel">Osc</div>
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
				<div class="section-label font-pixel">Filter</div>
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
				<div class="section-label font-pixel">Env</div>
				<EnvelopeView
					attackMs={synth.attackMs}
					decayMs={synth.decayMs}
					sustain={synth.sustain}
					releaseMs={synth.releaseMs}
					height={60}
				/>
				<div class="knob-row">
					<Knob
						label="A"
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
						label="D"
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
						label="S"
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
						label="R"
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
				<div class="section-label font-pixel">Amp</div>
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

		<div class="rack-footer font-pixel">
			Drag knobs ↕ • Shift = fine • Double-click = reset
		</div>
	</div>
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

	.chain-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		letter-spacing: 2px;
		text-transform: uppercase;
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

	.flow-node.fx-slot {
		opacity: 0.5;
		border-style: dashed;
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
