<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { ui } from '$lib/stores/ui.svelte';
	import { adapter } from '$lib/adapter';
	import VoiceCard from './VoiceCard.svelte';
	import type { HoldMode } from '$lib/adapter/types';

	const VIRTUAL_COMPUTER_KEYBOARD = 999_998;
	const VIRTUAL_GUITAR_AUDIO = 999_997;

	type VoiceRow = {
		kind: 'melody' | 'canon' | 'counterpoint';
		label: string;
		index?: number;
		transpose: number;
		timeOffsetBeats?: number;
		holdMode?: HoldMode | null;
		output: string;
		onclick: () => void;
	};

	function inputSourceLabel(sel: number | null): string {
		if (sel === null) return 'No input';
		if (sel === VIRTUAL_COMPUTER_KEYBOARD) return 'Computer Keyboard';
		if (sel === VIRTUAL_GUITAR_AUDIO) return 'Guitar Audio';
		const dev = midi.inputs.find((d) => d.index === sel);
		return dev?.name ?? `Input ${sel}`;
	}

	function voicePositionLabel(index: number, count: number): string {
		const names = ['Soprano', 'Alto', 'Tenor', 'Bass'];
		if (count <= 4 && names[index]) return names[index];
		return `V${index + 1}`;
	}

	function outputForVoice(voiceIdx: number): string {
		const t = midi.voiceOutputs[voiceIdx];
		if (!t || t.kind === 'synth') return 'Synth';
		if (t.kind === 'off') return 'Off';
		if (t.kind === 'midi_port') {
			const dev = midi.selectedOutputs[t.port];
			if (typeof dev === 'number') {
				const out = midi.outputs.find((d) => d.index === dev);
				return out?.name ?? `Port ${t.port + 1}`;
			}
			return `Port ${t.port + 1}`;
		}
		return 'Synth';
	}

	// Canon + counterpoint emissions don't flow through the main engine's
	// last_port_map(): each Companion DispatchOp carries its own
	// VoiceOutputTarget baked in by the lane (src-tauri commands/engine.rs
	// dispatch_companion_ops). The UI doesn't track per-canon-voice
	// destinations in the midi store, so we surface a truthful "Companion"
	// label instead of inventing a per-slot mapping. A v2 enhancement can
	// expose the lane's target via canonState() once the backend includes it.
	const COMPANION_OUTPUT = 'Companion';

	function openCanonEditor() {
		ui.setActiveTab('companion');
	}
	function openMelodyEditor() {
		// "Melody" is the player's own input position. The Companion
		// tab hosts the per-voice editor; for the user voice we route
		// to the Harmony tab since voicePosition lives in the harmony
		// surface ("you play" row in MidiDevices).
		ui.setActiveTab('play');
	}

	let rows = $derived.by<VoiceRow[]>(() => {
		const out: VoiceRow[] = [];
		// 1) Melody — the user's own voice. Always shown.
		out.push({
			kind: 'melody',
			label: voicePositionLabel(engine.voicePosition, engine.voiceCount),
			transpose: 0,
			output: outputForVoice(engine.voicePosition),
			onclick: openMelodyEditor
		});

		// 2) Canon voices. Output destination is "Companion" rather than
		// a specific slot because canon DispatchOps carry their own
		// VoiceOutputTarget; the UI doesn't track that per-canon-voice.
		if (engine.companionEnabled && engine.canonEnabled) {
			for (let i = 0; i < engine.canonVoices.length; i++) {
				const v = engine.canonVoices[i];
				const hold = v.hold_mode ?? engine.canonLaneHoldMode ?? engine.companionHoldMode;
				out.push({
					kind: 'canon',
					label: v.preset_id ?? `Canon ${i + 1}`,
					index: i,
					transpose: v.transpose_degrees,
					timeOffsetBeats: v.delay_beats,
					holdMode: hold,
					output: COMPANION_OUTPUT,
					onclick: openCanonEditor
				});
			}
		}

		// 3) Counterpoint lane.
		if (engine.companionEnabled) {
			out.push({
				kind: 'counterpoint',
				label: 'Counterpoint',
				transpose: 0,
				timeOffsetBeats: 0,
				holdMode: engine.counterpointLaneHoldMode ?? engine.companionHoldMode,
				output: COMPANION_OUTPUT,
				onclick: openCanonEditor
			});
		}
		return out;
	});

	let outputNodes = $derived.by(() => {
		const seen = new Set<string>();
		const nodes: { id: string; label: string }[] = [];
		for (const r of rows) {
			if (seen.has(r.output)) continue;
			seen.add(r.output);
			nodes.push({ id: r.output, label: r.output });
		}
		return nodes;
	});
</script>

<section class="vgc-wrap">
	<header class="vgc-header">
		<h3 class="font-ui">Voice generation chain</h3>
		<p class="vgc-sub font-ui">
			Click a card to edit. Live indicators land in a v2 pass.
		</p>
	</header>

	<div class="vgc-flow">
		<div class="vgc-node source-node font-ui" title="Live input source">
			<div class="node-title">Input</div>
			<div class="node-sub">{inputSourceLabel(midi.selectedInput)}</div>
		</div>

		<div class="vgc-arrow font-ui">▶</div>

		<div class="vgc-voices">
			{#each rows as row (row.kind + ':' + (row.index ?? row.label))}
				<VoiceCard
					kind={row.kind}
					label={row.label}
					index={row.index}
					transpose={row.transpose}
					timeOffsetBeats={row.timeOffsetBeats}
					holdMode={row.holdMode}
					output={row.output}
					onclick={row.onclick}
				/>
			{/each}
		</div>

		<div class="vgc-arrow font-ui">▶</div>

		<div class="vgc-outputs">
			{#each outputNodes as out (out.id)}
				<div class="vgc-node out-node font-ui" title={out.label}>
					<div class="node-title">{out.label}</div>
					<div class="node-sub">
						{out.label === 'Synth'
							? adapter.capabilities.audioFx ? 'built-in' : 'host'
							: out.label === 'Off' ? 'muted' : 'external'}
					</div>
				</div>
			{:else}
				<div class="vgc-node out-node muted font-ui">
					<div class="node-title">No output</div>
				</div>
			{/each}
		</div>
	</div>
</section>

<style>
	.vgc-wrap {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 10px 12px;
		background: linear-gradient(180deg, #0f0d1d, #0a0918);
		border: 1px solid var(--color-border);
	}

	.vgc-header h3 {
		margin: 0;
		font-size: var(--font-size-sm);
		color: var(--color-accent-gold);
		letter-spacing: 2px;
		text-transform: uppercase;
	}

	.vgc-sub {
		margin: 2px 0 0;
		font-size: var(--font-size-xs);
		color: var(--color-text-dim);
	}

	.vgc-flow {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}

	.vgc-voices {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		flex: 1;
		min-width: 0;
	}

	.vgc-outputs {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.vgc-node {
		padding: 6px 10px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		min-width: 90px;
		text-align: center;
	}

	.node-title {
		font-size: var(--font-size-xs);
		color: var(--color-accent-magenta);
		letter-spacing: 1px;
	}

	.node-sub {
		font-size: var(--font-size-xs);
		color: var(--color-text-dim);
		margin-top: 2px;
	}

	.source-node {
		border-color: var(--color-accent-cyan-dim, var(--color-accent-cyan));
	}
	.source-node .node-title {
		color: var(--color-accent-cyan);
	}

	.out-node {
		border-color: var(--color-accent-amber);
	}
	.out-node .node-title {
		color: var(--color-accent-amber);
	}

	.out-node.muted {
		opacity: 0.4;
	}

	.vgc-arrow {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-sm);
	}

	@media (max-width: 760px) {
		.vgc-flow {
			flex-direction: column;
			align-items: stretch;
		}
		.vgc-arrow {
			text-align: center;
			transform: rotate(90deg);
		}
		.vgc-outputs {
			flex-direction: row;
			flex-wrap: wrap;
		}
	}
</style>
