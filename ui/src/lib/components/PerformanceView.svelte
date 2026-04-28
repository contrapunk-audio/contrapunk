<!--
	PerformanceView — simplified 8-knob layout for non-experts and live use.

	Spec lives in .planning/PERFORMANCE-VIEW.md. Eight knobs:
	  Mode  Voices  Tightness  Adventurous
	  Key   Scale   You play   Spread

	Composite knobs (Tightness / Adventurous / Spread) hold local state and
	fan out to multiple engine setters on change. They do NOT round-trip
	from engine state — they just display the user's last-touched position
	(per .planning/PERFORMANCE-VIEW.md, "Implementation notes").
-->

<script lang="ts">
	import { engine, ALL_KEYS, KEY_DISPLAY } from '$lib/stores/engine.svelte';
	import type {
		HarmonyModeName,
		ScaleModeName,
		VoiceLeadingStyleName,
		KeyName
	} from '$lib/stores/engine.svelte';
	import Knob from './Knob.svelte';

	// === Mode (5 detents) ===
	const MODE_LABELS = ['Off', 'Parallel 3rds', 'Parallel 4ths', 'Contrary', 'Functional'];
	const MODE_TO_ENGINE: HarmonyModeName[] = [
		'PassThrough',
		'DiatonicThirds',
		'DiatonicFourths',
		'ContraryMotion',
		'FunctionalHarmony'
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

	// === You play (1..voiceCount) ===
	function onYouPlayChange(v: number) {
		void engine.setVoicePosition(Math.round(v) - 1);
	}

	// === Tightness (0..1, composite) ===
	let tightness = $state(0.5);
	function tightnessLabel(t: number): string {
		if (t < 0.3) return 'Off';
		if (t < 0.6) return 'Loose';
		if (t < 0.85) return 'Smooth';
		return 'Strict';
	}
	function applyTightness(t: number) {
		tightness = t;
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
</script>

<div class="performance-view">
	<div class="knob-grid">
		<!-- Row 1 — live performance dials -->
		<div class="knob-cell">
			<Knob
				value={modeIndex}
				min={0}
				max={4}
				step={1}
				size={72}
				label="Mode"
				format={(v) => MODE_LABELS[Math.round(v)] ?? ''}
				onchange={onModeChange}
			/>
		</div>
		<div class="knob-cell">
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
		</div>
		<div class="knob-cell">
			<Knob
				value={tightness}
				min={0}
				max={1}
				step={0.01}
				size={72}
				label="Tightness"
				format={tightnessLabel}
				onchange={applyTightness}
			/>
		</div>
		<div class="knob-cell">
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
		</div>

		<!-- Row 2 — per-song / per-set setup -->
		<div class="knob-cell">
			<div class="key-stack">
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
			</div>
		</div>
		<div class="knob-cell">
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
		</div>
		<div class="knob-cell">
			<Knob
				value={engine.voicePosition + 1}
				min={1}
				max={Math.max(1, engine.voiceCount)}
				step={1}
				size={72}
				label="You play"
				format={(v) => {
					const i = Math.round(v) - 1;
					if (engine.voiceCount <= 4) {
						return ['Soprano', 'Alto', 'Tenor', 'Bass'][i] ?? `V${i + 1}`;
					}
					return `V${i + 1}`;
				}}
				onchange={onYouPlayChange}
			/>
		</div>
		<div class="knob-cell">
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
		</div>
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

	.key-stack {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
	}

	.auto-pip {
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
	.auto-pip:hover {
		border-color: var(--color-accent-cyan, #4dd);
	}

</style>
