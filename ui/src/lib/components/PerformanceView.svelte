<!--
	PerformanceView — simplified 8-knob layout for non-experts and live use.

	Spec lives in .planning/PERFORMANCE-VIEW.md. Eight knobs:
	  Mode  Voices  Tightness  Adventurous
	  Key   Scale   Your register   Spread

	Composite knobs fan out to multiple engine setters on change. Tightness
	round-trips from voice-leading state; Adventurous and Spread retain their
	last-touched positions.
-->

<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { engine, ALL_KEYS, KEY_DISPLAY } from '$lib/stores/engine.svelte';
	import type {
		HarmonyModeName,
		ScaleModeName,
		VoiceLeadingStyleName,
		KeyName
	} from '$lib/stores/engine.svelte';
	import { adapter } from '$lib/adapter';
	import { knobMidiMap, type KnobIndex } from '$lib/stores/knobMidiMap.svelte';
	import Knob from './Knob.svelte';

	let { midiLearnEnabled = true }: { midiLearnEnabled?: boolean } = $props();

	// === Mode (5 detents) ===
	const MODE_LABELS = [
		'Off',
		'Parallel 3rds',
		'Parallel 4ths',
		'Contrary',
		'Functional',
		...(adapter.capabilities.intervalMaps ? ['Interval Map'] : [])
	];
	const MODE_TO_ENGINE: HarmonyModeName[] = [
		'PassThrough',
		'DiatonicThirds',
		'DiatonicFourths',
		'ContraryMotion',
		'FunctionalHarmony',
		...(adapter.capabilities.intervalMaps ? (['ExplicitIntervals'] as HarmonyModeName[]) : [])
	];
	function engineModeToIndex(m: HarmonyModeName): number {
		const i = MODE_TO_ENGINE.indexOf(m);
		return i >= 0 ? i : 0;
	}
	let modeIndex = $derived(engineModeToIndex(engine.mode));
	function onModeChange(v: number) {
		const i = Math.round(v);
		void engine.setMode(MODE_TO_ENGINE[i]);
	}

	// === Scale (8 categories, representative sub-mode each) ===
	const SCALE_CATEGORIES: { label: string; engine: ScaleModeName }[] = [
		{ label: 'Pentatonic', engine: 'MajorPentatonic' },
		{ label: 'Major', engine: 'Ionian' },
		{ label: 'Natural Minor', engine: 'Aeolian' },
		{ label: 'Harmonic Min', engine: 'HarmonicMinor' },
		{ label: 'Melodic Min', engine: 'MelodicMinor' },
		{ label: 'Modal', engine: 'Dorian' },
		{ label: 'World', engine: 'Pelog' },
		{ label: 'Symmetric', engine: 'WholeTone' }
	];
	function engineScaleToCategoryIndex(s: ScaleModeName): number {
		// Map any sub-mode back to its category index for first-paint.
		const cats: Record<string, number> = {
			MajorPentatonic: 0, MinorPentatonic: 0, Hirajoshi: 0, InSen: 0, Iwato: 0, Yo: 0, Kumoi: 0,
			Ionian: 1,
			Aeolian: 2,
			HarmonicMinor: 3, LocrianNat6: 3, IonianAug: 3, DorianSharp4: 3,
			PhrygianDominant: 3, LydianSharp2: 3, SuperLocrianDim: 3,
			MelodicMinor: 4, DorianFlat2: 4, LydianAug: 4, LydianDominant: 4,
			MixolydianFlat6: 4, LocrianNat2: 4, SuperLocrian: 4,
			Dorian: 5, Phrygian: 5, Lydian: 5, Mixolydian: 5, Locrian: 5,
			Pelog: 6, Persian: 6, NeapolitanMajor: 6, NeapolitanMinor: 6,
			Enigmatic: 6, HungarianMajor: 6, HungarianMinor: 6, Oriental: 6,
			WholeTone: 7, DiminishedWholeHalf: 7, DiminishedHalfWhole: 7, AugmentedHex: 7
		};
		return cats[s] ?? 1;
	}
	let scaleIndex = $derived(engineScaleToCategoryIndex(engine.scaleMode));
	function onScaleChange(v: number) {
		const i = Math.round(v);
		void engine.setScaleMode(SCALE_CATEGORIES[i].engine);
	}

	// === Key (12 detents) + Auto-key toggle ===
	function engineKeyToIndex(k: KeyName): number {
		const i = ALL_KEYS.indexOf(k);
		return i >= 0 ? i : 0;
	}
	let keyIndex = $derived(engineKeyToIndex(engine.key));
	function onKeyChange(v: number) {
		const i = Math.round(v);
		// Touching the Key knob disables Auto-key with a visible state change.
		// Per the brutal-critic UI review fix (.planning/PERFORMANCE-VIEW.md).
		if (engine.autoKey) {
			void engine.setAutoKey(false);
		}
		void engine.setKey(ALL_KEYS[i]);
	}

	// === Voices (1-8) ===
	function onVoicesChange(v: number) {
		void engine.setVoiceCount(Math.round(v));
	}

	// === Your register (1..voiceCount) ===
	function onRegisterChange(v: number) {
		void engine.setVoicePosition(Math.round(v) - 1);
	}

	// === Tightness (0..1, composite) ===
	let tightness = $derived(
		!engine.voiceLeadingEnabled
			? 0
			: engine.voiceLeadingStyle === 'Free'
				? 0.45
				: engine.voiceLeadingStyle === 'Jazz'
					? 0.7
					: 1
	);
	function tightnessLabel(t: number): string {
		if (t < 0.3) return 'Off';
		if (t < 0.6) return 'Loose';
		if (t < 0.85) return 'Smooth';
		return 'Strict';
	}
	function applyTightness(t: number) {
		if (t < 0.3) {
			void engine.setVoiceLeading(false);
			return;
		}
		const style: VoiceLeadingStyleName =
			t < 0.6 ? 'Free' : t < 0.85 ? 'Jazz' : 'Palestrina';
		void engine.setVoiceLeading(true, style);
	}

	// === Adventurous (0..1, composite) ===
	let adventurous = $state(0);
	function adventurousLabel(a: number): string {
		if (a <= 0.05) return 'In-key';
		const range = Math.max(1, Math.min(7, Math.round(a * 7)));
		return `Borrow ${range}`;
	}
	function applyAdventurous(a: number) {
		adventurous = a;
		if (a <= 0) {
			void engine.setInterchange(false, 1);
			return;
		}
		const range = Math.max(1, Math.min(7, Math.round(a * 7)));
		void engine.setInterchange(true, range);
	}

	// === Spread (continuous via engine octave_intensity coefficient) ===
	// Engine takes a continuous f32 intensity that scales per-voice
	// octave displacement. 0 = no spread (matches OctaveMode::None);
	// 1 = full-octave spread (matches legacy OctaveMode::Spread).
	// Mirror / BassTrebleSplit aren't exposed in this knob — they
	// remain available in the Advanced view's full Octave Mode picker.
	let spread = $state(0);
	function spreadLabel(s: number): string {
		if (s <= 0.01) return 'Tight';
		return `Wide ${Math.round(s * 100)}%`;
	}
	function applySpread(s: number) {
		spread = s;
		if (s <= 0.01) {
			void engine.setOctaveMode('None');
		} else {
			// Set mode first so apply_octave_mode hits the Spread branch,
			// then push the intensity coefficient.
			void engine.setOctaveMode('Spread');
			void engine.setOctaveIntensity(s);
		}
	}

	// Auto-key handling
	function toggleAutoKey() {
		void engine.setAutoKey(!engine.autoKey);
	}

	// === Hardware-knob CC routing (MIDI Learn) ===
	// Backend emits "knob-cc-raw" events with `{ cc, value }` for every
	// MIDI CC received. We resolve the CC → knob-index via the user's
	// learned mapping (`knobMidiMap`), then dispatch to the same apply
	// function the on-screen knob uses. While a knob is in "learning"
	// state, the next CC observed binds to that knob instead.
	function dispatchKnobValue(idx: KnobIndex, value: number) {
		switch (idx) {
			case 0: // Mode detents
				onModeChange(value * (MODE_TO_ENGINE.length - 1));
				break;
			case 1: // Voices (1-8)
				onVoicesChange(1 + value * 7);
				break;
			case 2: // Tightness (0..1)
				applyTightness(value);
				break;
			case 3: // Adventurous (0..1)
				applyAdventurous(value);
				break;
			case 4: // Key (12 detents) — also disables auto-key on hardware turn
				onKeyChange(value * 11);
				break;
			case 5: // Scale (8 categories)
				onScaleChange(value * 7);
				break;
			case 6: // Your register (1..voiceCount)
				onRegisterChange(1 + value * Math.max(0, engine.voiceCount - 1));
				break;
			case 7: // Spread (continuous)
				applySpread(value);
				break;
		}
	}

	let unlistenKnobCc: (() => void) | undefined;
	onMount(() => {
		unlistenKnobCc = adapter.onKnobCcRaw((cc, value) => {
			if (!midiLearnEnabled) return;
			const learning = knobMidiMap.learning;
			if (learning !== null) {
				// Capture this CC as the bound CC for the learning knob.
				knobMidiMap.bind(learning, cc);
				knobMidiMap.cancelLearn();
				return;
			}
			const idx = knobMidiMap.knobForCc(cc);
			if (idx !== null) dispatchKnobValue(idx, value);
		});

		// Escape cancels in-progress learning.
		const onKey = (e: KeyboardEvent) => {
			if (e.key === 'Escape' && knobMidiMap.learning !== null) {
				knobMidiMap.cancelLearn();
			}
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});
	onDestroy(() => {
		unlistenKnobCc?.();
	});

	function toggleLearn(idx: KnobIndex) {
		knobMidiMap.toggleLearn(idx);
	}

	function ccLabel(idx: KnobIndex): string {
		const cc = knobMidiMap.map[idx];
		return cc === null ? '—' : `CC${cc}`;
	}

	function resetMidiMap() {
		knobMidiMap.resetToMpkDefaults();
	}
</script>

{#snippet learnPip(idx: KnobIndex)}
	<button
		type="button"
		class="learn-pip font-ui"
		class:learning={knobMidiMap.learning === idx}
		onclick={() => toggleLearn(idx)}
		aria-pressed={knobMidiMap.learning === idx}
		title={knobMidiMap.learning === idx
			? 'Waiting for MIDI CC… (Esc to cancel)'
			: `Click, then move a hardware knob to bind it (${ccLabel(idx)})`}
	>
		{knobMidiMap.learning === idx ? 'LEARN…' : ccLabel(idx)}
	</button>
{/snippet}

<div class="performance-view">
	<div class="knob-grid">
		<!-- Row 1 — live performance dials -->
		<div class="knob-cell">
			<div class="learn-stack">
				<Knob
					value={modeIndex}
					min={0}
					max={MODE_TO_ENGINE.length - 1}
					step={1}
					size={72}
					label="Mode"
					format={(v) => MODE_LABELS[Math.round(v)] ?? ''}
					onchange={onModeChange}
				/>
				{@render learnPip(0)}
			</div>
		</div>
		<div class="knob-cell">
			<div class="learn-stack">
				<Knob
					value={engine.voiceCount}
					min={1}
					max={8}
					step={1}
					size={72}
					label="Voices"
					format={(v) => Math.round(v).toString()}
					onchange={onVoicesChange}
				/>
				{@render learnPip(1)}
			</div>
		</div>
		<div class="knob-cell">
			<div class="learn-stack">
				<Knob
					value={tightness}
					min={0}
					max={1}
					step={0.01}
					size={72}
					label="Tightness"
					help="Voice-leading strength, not timing: Off disables smoothing; Loose allows free motion; Smooth favors compact jazz movement; Strict applies Palestrina-style constraints."
					format={tightnessLabel}
					onchange={applyTightness}
				/>
				{@render learnPip(2)}
			</div>
		</div>
		<div class="knob-cell">
			<div class="learn-stack">
				<Knob
					value={adventurous}
					min={0}
					max={1}
					step={0.01}
					size={72}
					label="Adventurous"
					format={adventurousLabel}
					onchange={applyAdventurous}
				/>
				{@render learnPip(3)}
			</div>
		</div>

		<!-- Row 2 — per-song / per-set setup -->
		<div class="knob-cell">
			<div class="learn-stack">
				<Knob
					value={keyIndex}
					min={0}
					max={11}
					step={1}
					size={72}
					label="Key"
					format={(v) => KEY_DISPLAY[ALL_KEYS[Math.round(v)] ?? 'C'] ?? '?'}
					onchange={onKeyChange}
				/>
				<div class="pip-row">
					<button
						type="button"
						class="auto-pip font-ui"
						class:on={engine.autoKey}
						onclick={toggleAutoKey}
						aria-pressed={engine.autoKey}
						title={engine.autoKey ? 'Auto-key on (engine detects)' : 'Auto-key off (manual)'}
					>
						{engine.autoKey ? 'AUTO' : 'MAN'}
					</button>
					{@render learnPip(4)}
				</div>
			</div>
		</div>
		<div class="knob-cell">
			<div class="learn-stack">
				<Knob
					value={scaleIndex}
					min={0}
					max={7}
					step={1}
					size={72}
					label="Scale"
					format={(v) => SCALE_CATEGORIES[Math.round(v)]?.label ?? ''}
					onchange={onScaleChange}
				/>
				{@render learnPip(5)}
			</div>
		</div>
		<div class="knob-cell">
			<div class="learn-stack">
				<Knob
					value={engine.voicePosition + 1}
					min={1}
					max={Math.max(1, engine.voiceCount)}
					step={1}
					size={72}
					label="Your register"
					format={(v) => {
						const i = Math.round(v) - 1;
						if (engine.voiceCount <= 4) {
							return ['Soprano', 'Alto', 'Tenor', 'Bass'][i] ?? `V${i + 1}`;
						}
						return `V${i + 1}`;
					}}
					onchange={onRegisterChange}
				/>
				{@render learnPip(6)}
			</div>
		</div>
		<div class="knob-cell">
			<div class="learn-stack">
				<Knob
					value={spread}
					min={0}
					max={1}
					step={0.01}
					size={72}
					label="Spread"
					format={spreadLabel}
					onchange={applySpread}
				/>
				{@render learnPip(7)}
			</div>
		</div>
	</div>

	<div class="midi-learn-footer">
		<button type="button" class="reset-btn font-ui" onclick={resetMidiMap}
			title="Restore the MPK Mini default mapping (knobs 1-8 → CC 70-77)">
			Reset to MPK Mini defaults
		</button>
	</div>
</div>

<style>
	.performance-view {
		padding: 0.5rem;
		container-type: inline-size;
	}

	/* Always 4 columns × 2 rows — knobs scale via the inline `size`
	   prop based on container width via @container queries below.
	   Each cell takes equal width so the layout stays a clean 4-grid
	   regardless of the parent column's width. */
	.knob-grid {
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 0.5rem;
		justify-content: stretch;
		align-items: start;
	}

	.knob-cell {
		display: flex;
		justify-content: center;
		align-items: flex-start;
		min-height: 100px;
		min-width: 0;
	}

	.learn-stack {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
	}

	.pip-row {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 4px;
		flex-wrap: wrap;
		justify-content: center;
	}

	.auto-pip,
	.learn-pip {
		padding: 2px 10px;
		font-size: var(--font-size-xs, 11px);
		background: transparent;
		border: 1px solid var(--color-border, #444);
		color: var(--color-text-dim, #888);
		cursor: pointer;
		letter-spacing: 0.1em;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}
	.auto-pip.on {
		background: var(--color-accent-teal, #1a8a8a);
		border-color: var(--color-accent-cyan, #4dd);
		color: #fff;
		box-shadow: var(--glow-teal, 0 0 6px #1a8a8a);
	}
	.auto-pip:hover,
	.learn-pip:hover {
		border-color: var(--color-accent-cyan, #4dd);
	}

	/* "Learning" state — pulsing border + amber tint to make it
	 * unambiguous that the next CC will bind. */
	.learn-pip.learning {
		background: var(--color-accent-amber, #c98a1a);
		border-color: var(--color-accent-amber, #c98a1a);
		color: #111;
		animation: learn-pulse 0.8s infinite;
	}
	@keyframes learn-pulse {
		0%, 100% { box-shadow: 0 0 0 0 var(--color-accent-amber, #c98a1a); }
		50%      { box-shadow: 0 0 8px 2px var(--color-accent-amber, #c98a1a); }
	}

	.midi-learn-footer {
		display: flex;
		justify-content: flex-end;
		margin-top: 0.75rem;
	}

	.reset-btn {
		padding: 4px 12px;
		font-size: var(--font-size-xs, 11px);
		background: transparent;
		border: 1px solid var(--color-border, #444);
		color: var(--color-text-dim, #888);
		cursor: pointer;
		letter-spacing: 0.05em;
	}
	.reset-btn:hover {
		border-color: var(--color-accent-cyan, #4dd);
		color: var(--color-text, #ddd);
	}
</style>
