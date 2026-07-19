<script lang="ts">
	import { onMount } from 'svelte';
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

	// Engine port_map snapshot — refreshed onMount + after engine config
	// changes. Maps result-index i → arrangement slot. Empty when the
	// engine has not processed a note yet (Idle); in that case
	// `slotFor()` falls back to voicePosition-based mapping for the
	// melody card and naive index-based mapping for harmonies.
	let portMap = $state<number[]>([]);
	async function refreshPortMap() {
		try {
			portMap = await adapter.getLastPortMap();
		} catch {
			portMap = [];
		}
	}

	onMount(() => {
		refreshPortMap();
	});

	// Re-fetch when config changes that affect routing. The store doesn't
	// emit a single "port_map invalidated" event, so we depend on the
	// individual scalars that drive it.
	$effect(() => {
		// Touch reactive fields so the effect re-runs on change.
		void engine.voiceCount;
		void engine.voicePosition;
		void engine.octaveMode;
		void engine.mode;
		void engine.counterpointSpecies;
		refreshPortMap();
	});

	/** Map result-index i (0=melody, 1..=harmony voices) → arrangement
	 *  slot. Uses the engine's actual port_map when available; otherwise
	 *  falls back to a config-derived best guess (melody at
	 *  voicePosition, harmonies fill the remaining slots in order).
	 *  This is the fix for brutal-critic #9. */
	function slotFor(resultIdx: number): number {
		if (resultIdx < portMap.length) {
			return portMap[resultIdx];
		}
		// Fallback when engine hasn't populated port_map yet.
		if (resultIdx === 0) return engine.voicePosition;
		// Naive: result-index i (i > 0) fills slots 0..voiceCount-1 in
		// order, skipping the user's voicePosition. Matches Pass-Through
		// and simple voicings; non-trivial voicings re-fetch port_map.
		let slot = 0;
		let counted = 0;
		for (let i = 0; i < 8; i++) {
			if (i === engine.voicePosition) continue;
			if (counted === resultIdx - 1) {
				slot = i;
				break;
			}
			counted++;
		}
		return slot;
	}

	function outputForSlot(slot: number): string {
		const t = midi.voiceOutputs[slot];
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
		// surface ("you play" row in InputPanel).
		ui.setActiveTab('play');
	}

	let rows = $derived.by<VoiceRow[]>(() => {
		const out: VoiceRow[] = [];
		// 1) Melody — the user's own voice. The melody's actual output
		// slot comes from the engine's port_map[0] (which honors
		// non-default voicings like Species 2-4 and drop voicings);
		// falls back to voicePosition when port_map is empty.
		const melodySlot = slotFor(0);
		out.push({
			kind: 'melody',
			label: voicePositionLabel(melodySlot, engine.voiceCount),
			transpose: 0,
			output: outputForSlot(melodySlot),
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

		// 3) Counterpoint lane — only when Companion is enabled AND
		// counterpoint mode is engaged. Previously the row was emitted
		// whenever companionEnabled was true, advertising a lane the
		// user might not actually have on (brutal-critic #9 follow-up).
		if (engine.companionEnabled && engine.mode === 'StrictCounterpoint') {
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
		<div>
			<h3 class="font-ui">Arrangement map</h3>
			<p class="vgc-sub font-ui">A visual summary of who plays, who follows, and where each part goes.</p>
		</div>
		<span class="source-pill font-code" title="Current live input source">SOURCE · {inputSourceLabel(midi.selectedInput)}</span>
	</header>

	<div class="core-parts">
		<button class="part-card player-part" type="button" onclick={openMelodyEditor} title="Your live melody and its position in the harmonic texture.">
			<div class="part-top font-ui"><strong>YOU PLAY</strong><span>TEAL · CH 1</span></div>
			<div class="part-main font-code">{rows[0]?.label ?? 'Melody'} · live subject</div>
			<div class="part-meta font-code">{inputSourceLabel(midi.selectedInput)} → {rows[0]?.output ?? 'Synth'}</div>
		</button>
		<div class="relationship-mark font-ui" aria-hidden="true">+</div>
		<button class="part-card harmony-part" type="button" onclick={openMelodyEditor} title="The chordal voices generated at the same time as your melody.">
			<div class="part-top font-ui"><strong>HARMONIC SUPPORT</strong><span>MAGENTA · CH 2–5</span></div>
			<div class="part-main font-code">{engine.mode} · {Math.max(0, engine.voiceCount - 1)} generated voice{engine.voiceCount === 2 ? '' : 's'}</div>
			<div class="part-meta font-code">{engine.voiceLeadingEnabled ? `${engine.voiceLeadingStyle} voice leading` : 'Direct movement'} · Spread {Math.round(engine.octaveIntensity * 100)}%</div>
		</button>
	</div>

	<div class="companion-heading font-ui">
		<span>COUNTERPOINT LINES</span>
		<span>{rows.filter((row) => row.kind !== 'melody').length || 'NO'} ACTIVE</span>
	</div>
	<div class="vgc-voices">
		{#each rows.filter((row) => row.kind !== 'melody') as row (row.kind + ':' + (row.index ?? row.label))}
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
		{:else}
			<div class="empty-lines font-code">No counterpoint lines enabled. Harmonic Support still follows your melody.</div>
		{/each}
	</div>

	<div class="output-map">
		<span class="output-label font-ui">OUTPUT</span>
		<div class="output-line" aria-hidden="true"></div>
		<div class="vgc-outputs">
			{#each outputNodes as out (out.id)}
				<div class="vgc-node out-node font-ui" title={`Destination: ${out.label}`}>
					<div class="node-title">{out.label}</div>
					<div class="node-sub">
						{out.label === 'Synth' ? (adapter.capabilities.audioFx ? 'built-in' : 'host') : out.label === 'Off' ? 'muted' : out.label === 'Companion' ? 'internal' : 'external'}
					</div>
				</div>
			{/each}
		</div>
	</div>
</section>

<style>
	.vgc-wrap {
		display: grid;
		gap: 8px;
		padding: 9px;
		background: linear-gradient(180deg, #0f0d1d, #0a0918);
		border: 1px solid var(--color-border);
	}
	.vgc-header,
	.part-top,
	.companion-heading,
	.output-map {
		display: flex;
		align-items: center;
	}
	.vgc-header { justify-content: space-between; gap: 10px; }
	.vgc-header h3 {
		margin: 0;
		color: var(--color-accent-gold);
		font-size: var(--font-size-sm);
		letter-spacing: 2px;
		text-transform: uppercase;
	}
	.vgc-sub { margin: 2px 0 0; color: var(--color-text-dim); font-size: var(--font-size-xs); }
	.source-pill {
		padding: 4px 7px;
		border: 1px solid var(--color-accent-cyan-dim, var(--color-border));
		color: var(--color-accent-cyan);
		font-size: var(--font-size-xs);
		white-space: nowrap;
	}
	.core-parts {
		display: grid;
		grid-template-columns: minmax(170px, 0.8fr) auto minmax(250px, 1.4fr);
		align-items: stretch;
		gap: 6px;
	}
	.part-card {
		min-width: 0;
		padding: 8px 9px;
		border: 1px solid var(--color-border);
		border-radius: 0;
		background: rgba(28, 25, 52, 0.92);
		color: inherit;
		text-align: left;
		cursor: pointer;
	}
	.part-card:hover { background: rgba(35, 31, 62, 0.98); }
	.player-part { border-left: 4px solid var(--color-piano-input); }
	.harmony-part { border-left: 4px solid var(--color-piano-harmony); }
	.part-top { justify-content: space-between; gap: 6px; }
	.part-top strong { color: var(--color-text-primary); font-size: 9px; letter-spacing: 0.8px; }
	.part-top span { color: var(--color-text-dim); font-size: 7px; }
	.part-main { margin-top: 7px; color: var(--color-text-primary); font-size: 10px; }
	.part-meta { margin-top: 4px; color: var(--color-text-secondary); font-size: 8px; }
	.relationship-mark { align-self: center; color: var(--color-accent-cyan); font-size: 13px; }
	.companion-heading {
		justify-content: space-between;
		padding: 3px 0;
		border-bottom: 1px solid var(--color-border);
		color: var(--color-text-secondary);
		font-size: 8px;
		letter-spacing: 1.4px;
	}
	.companion-heading span:last-child { color: var(--color-accent-gold); }
	.vgc-voices { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 6px; }
	.vgc-voices :global(.voice-card) { width: 100%; max-width: none; min-width: 0; border-left-width: 4px; }
	.empty-lines {
		grid-column: 1 / -1;
		padding: 9px;
		border: 1px dashed var(--color-border);
		color: var(--color-text-dim);
		font-size: var(--font-size-xs);
	}
	.output-map { gap: 8px; padding-top: 2px; }
	.output-label { color: var(--color-accent-amber); font-size: 8px; letter-spacing: 1.2px; }
	.output-line { height: 1px; flex: 1; background: linear-gradient(90deg, var(--color-accent-amber), transparent); opacity: 0.5; }
	.vgc-outputs { display: flex; flex-wrap: wrap; gap: 4px; }
	.vgc-node {
		min-width: 86px;
		padding: 5px 8px;
		border: 1px solid var(--color-accent-amber);
		background: var(--color-widget-bg);
		text-align: center;
	}
	.node-title { color: var(--color-accent-amber); font-size: var(--font-size-xs); letter-spacing: 1px; }
	.node-sub { margin-top: 2px; color: var(--color-text-dim); font-size: var(--font-size-xs); }

	@media (max-width: 760px) {
		.vgc-header { align-items: flex-start; }
		.core-parts { grid-template-columns: 1fr; }
		.relationship-mark { display: none; }
		.vgc-voices { grid-template-columns: 1fr; }
		.output-map { align-items: flex-start; }
	}
</style>
