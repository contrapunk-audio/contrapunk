<script lang="ts">
	import { onMount } from 'svelte';
	import PixelSelect from './PixelSelect.svelte';
	import { adapter } from '$lib/adapter';
	import {
		engine,
		ALL_KEYS,
		ALL_MODES,
		SCALE_FAMILIES,
		OCTAVE_MODES,
		VOICE_LEADING_STYLES,
		type HarmonyModeName,
		type ImitativeFormName,
		type KeyName,
		type OctaveModeName,
		type ScaleModeName,
		type VoiceLeadingStyleName,
		type CounterpointSpeciesName
	} from '$lib/stores/engine.svelte';

	const STORAGE_KEY = 'contrapunk-ensemble-presets-v1';

	type CanonVoice = (typeof engine.canonVoices)[number];
	type EnsemblePreset = {
		id: string;
		name: string;
		builtIn: boolean;
		key: KeyName;
		scaleMode: ScaleModeName;
		mode: HarmonyModeName;
		voiceCount: number;
		voicePosition: number;
		voiceLeadingEnabled: boolean;
		voiceLeadingStyle: VoiceLeadingStyleName;
		octaveMode: OctaveModeName;
		companionEnabled: boolean;
		canonEnabled: boolean;
		imitativeForm: ImitativeFormName;
		canonVoices: CanonVoice[];
		harmonySpecies?: CounterpointSpeciesName;
		counterpoint?: {
			enabled: boolean;
			species?: string;
			transpose_degrees?: number;
			prefer_above?: boolean;
		} | null;
	};

	const strictFollower = (delay: number, transpose: number): CanonVoice => ({
		delay_beats: delay,
		transpose_degrees: transpose,
		time_ratio: 1,
		harmony_mode: 'PassThrough',
		reference_voice: null,
		voice_count: 1,
		voice_position: 0,
		voice_leading_enabled: false,
		voice_leading_style: null,
		octave_mode: 'None',
		counterpoint_species: null,
		counterpoint_strictness: null,
		hold_mode: { kind: 'forever' },
		preset_id: null
	});

	const BUILT_INS: EnsemblePreset[] = [
		{
			id: 'solo-no-harmony',
			name: 'Solo — No Harmony',
			builtIn: true,
			key: 'C',
			scaleMode: 'Ionian',
			mode: 'PassThrough',
			voiceCount: 1,
			voicePosition: 0,
			voiceLeadingEnabled: false,
			voiceLeadingStyle: 'Free',
			octaveMode: 'None',
			companionEnabled: false,
			canonEnabled: false,
			imitativeForm: 'free_imitation',
			canonVoices: [],
			counterpoint: null
		},
		{
			id: 'chorale-canon-fifth',
			name: 'Chorale + Canon at the Fifth',
			builtIn: true,
			key: 'C',
			scaleMode: 'Ionian',
			mode: 'BachChorale',
			voiceCount: 4,
			voicePosition: 3,
			voiceLeadingEnabled: true,
			voiceLeadingStyle: 'BachChorale',
			octaveMode: 'Spread',
			companionEnabled: true,
			canonEnabled: true,
			imitativeForm: 'strict_canon',
			canonVoices: [strictFollower(2, 4)],
			counterpoint: null
		},
		{
			id: 'bach-chorale-quartet',
			name: 'Bach Chorale Quartet',
			builtIn: true,
			key: 'C',
			scaleMode: 'Ionian',
			mode: 'BachChorale',
			voiceCount: 4,
			voicePosition: 3,
			voiceLeadingEnabled: true,
			voiceLeadingStyle: 'BachChorale',
			octaveMode: 'Spread',
			companionEnabled: false,
			canonEnabled: false,
			imitativeForm: 'free_imitation',
			canonVoices: [],
			counterpoint: null
		},
		{
			id: 'counterpoint-harmony-only',
			name: 'Counterpoint Harmony Only',
			builtIn: true,
			key: 'C',
			scaleMode: 'Ionian',
			mode: 'StrictCounterpoint',
			harmonySpecies: 'Species1',
			voiceCount: 4,
			voicePosition: 3,
			voiceLeadingEnabled: true,
			voiceLeadingStyle: 'BachChorale',
			octaveMode: 'None',
			companionEnabled: false,
			canonEnabled: false,
			imitativeForm: 'free_imitation',
			canonVoices: [],
			counterpoint: null
		},
		{
			id: 'three-voice-imitation',
			name: 'Three-Voice Free Imitation',
			builtIn: true,
			key: 'C',
			scaleMode: 'Dorian',
			mode: 'DiatonicThirds',
			voiceCount: 2,
			voicePosition: 1,
			voiceLeadingEnabled: true,
			voiceLeadingStyle: 'Free',
			octaveMode: 'None',
			companionEnabled: true,
			canonEnabled: true,
			imitativeForm: 'free_imitation',
			canonVoices: [
				{ delay_beats: 1, transpose_degrees: 2, time_ratio: 1, reference_voice: null },
				{ delay_beats: 2, transpose_degrees: -2, time_ratio: 0.5, reference_voice: null }
			],
			counterpoint: null
		}
	];

	const VALID_MODES = new Set(ALL_MODES.map((mode) => mode.name));
	const VALID_SCALES = new Set(SCALE_FAMILIES.flatMap((family) => family.modes.map((mode) => mode.name)));
	const VALID_OCTAVES = new Set(OCTAVE_MODES.map((mode) => mode.name));
	const VALID_STYLES = new Set(VOICE_LEADING_STYLES.map((style) => style.name));

	function isUserPreset(value: unknown): value is EnsemblePreset {
		if (typeof value !== 'object' || value === null) return false;
		const preset = value as Record<string, unknown>;
		return preset.builtIn === false
			&& typeof preset.id === 'string'
			&& typeof preset.name === 'string'
			&& ALL_KEYS.includes(preset.key as KeyName)
			&& VALID_MODES.has(preset.mode as HarmonyModeName)
			&& VALID_SCALES.has(preset.scaleMode as ScaleModeName)
			&& VALID_OCTAVES.has(preset.octaveMode as OctaveModeName)
			&& VALID_STYLES.has(preset.voiceLeadingStyle as VoiceLeadingStyleName)
			&& typeof preset.voiceCount === 'number'
			&& typeof preset.voicePosition === 'number'
			&& typeof preset.voiceLeadingEnabled === 'boolean'
			&& typeof preset.companionEnabled === 'boolean'
			&& typeof preset.canonEnabled === 'boolean'
			&& (preset.imitativeForm === 'strict_canon' || preset.imitativeForm === 'free_imitation')
			&& Array.isArray(preset.canonVoices);
	}

	let userPresets = $state<EnsemblePreset[]>([]);
	let selectedId = $state(BUILT_INS[0].id);
	let appliedId = $state<string | null>(null);
	let applying = $state(false);
	let saveOpen = $state(false);
	let saveName = $state('');
	let error = $state('');
	let presets = $derived([...BUILT_INS, ...userPresets]);
	let selected = $derived(presets.find((preset) => preset.id === selectedId));
	let options = $derived(presets.map((preset) => ({ value: preset.id, label: `${preset.builtIn ? '' : '★ '}${preset.name}` })));

	onMount(() => {
		try {
			const parsed = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '[]');
			if (Array.isArray(parsed)) userPresets = parsed.filter(isUserPreset).slice(0, 64);
		} catch {
			userPresets = [];
		}
	});

	function persistUsers() {
		try {
			localStorage.setItem(STORAGE_KEY, JSON.stringify(userPresets));
		} catch {
			error = 'Could not save presets on this device.';
		}
	}

	async function applySelected() {
		if (!selected || applying) return;
		applying = true;
		error = '';
		try {
			await engine.setKey(selected.key);
			if (!adapter.capabilities.pluginMidiOutputMode) {
				await engine.setScaleMode(selected.scaleMode);
			}
			await engine.setMode(selected.mode);
			if (selected.harmonySpecies) await engine.setCounterpointSpecies(selected.harmonySpecies);
			await engine.setVoiceCount(selected.voiceCount);
			await engine.setVoicePosition(selected.voicePosition);
			await engine.setVoiceLeading(
				selected.voiceLeadingEnabled,
				adapter.capabilities.pluginMidiOutputMode ? undefined : selected.voiceLeadingStyle
			);
			await engine.setOctaveMode(selected.octaveMode);
			await engine.setCanonVoices(selected.canonVoices);
			await engine.setImitativeForm(selected.imitativeForm);
			await engine.setCanonEnabled(selected.canonEnabled);
			if (selected.counterpoint !== undefined) {
				await adapter.counterpointSetConfig(selected.counterpoint ?? { enabled: false });
			}
			await engine.setCompanionEnabled(selected.companionEnabled);
			appliedId = selected.id;
		} catch (cause) {
			error = `Apply failed: ${cause}`;
		} finally {
			applying = false;
		}
	}

	function saveCurrent() {
		const name = saveName.trim();
		if (!name) return;
		const id = `user-${Date.now()}`;
		const preset: EnsemblePreset = {
			id,
			name,
			builtIn: false,
			key: engine.key,
			scaleMode: engine.scaleMode,
			mode: engine.mode,
			voiceCount: engine.voiceCount,
			voicePosition: engine.voicePosition,
			voiceLeadingEnabled: engine.voiceLeadingEnabled,
			voiceLeadingStyle: engine.voiceLeadingStyle,
			octaveMode: engine.octaveMode,
			companionEnabled: engine.companionEnabled,
			canonEnabled: engine.canonEnabled,
			imitativeForm: engine.imitativeForm,
			canonVoices: engine.canonVoices.map((voice) => ({ ...voice }))
		};
		userPresets = [...userPresets, preset];
		persistUsers();
		selectedId = id;
		appliedId = id;
		saveName = '';
		saveOpen = false;
	}
</script>

<section class="preset-bar" aria-label="Ensemble preset">
	<div class="preset-label font-ui">ENSEMBLE</div>
	<PixelSelect options={options} value={selectedId} label="Ensemble preset" help="Chooses a complete arrangement of harmony, Canon, and Counterpoint settings. Nothing changes until you press Apply." onchange={(value) => { selectedId = value; appliedId = null; }} />
	<button class="apply font-ui" type="button" disabled={!selected || applying} onclick={applySelected}>
		{applying ? 'APPLYING…' : appliedId === selectedId ? 'APPLIED' : 'APPLY'}
	</button>
	<button class="save font-ui" type="button" onclick={() => (saveOpen = !saveOpen)}>SAVE AS…</button>
	{#if saveOpen}
		<div class="save-row">
			<input class="font-code" bind:value={saveName} aria-label="New ensemble preset name" placeholder="Preset name" onkeydown={(event) => { if (event.key === 'Enter') saveCurrent(); if (event.key === 'Escape') saveOpen = false; }} />
			<button class="font-ui" type="button" disabled={!saveName.trim()} onclick={saveCurrent}>SAVE</button>
		</div>
	{/if}
	{#if error}<div class="error font-code">{error}</div>{/if}
</section>

<style>
	.preset-bar {
		order: 1;
		display: grid;
		grid-template-columns: 104px minmax(0, 1fr) auto auto;
		align-items: center;
		gap: 7px;
		padding: 6px 8px;
		border: 1px solid rgba(51, 221, 255, 0.48);
		background: linear-gradient(90deg, rgba(16, 34, 46, 0.96), rgba(18, 16, 32, 0.96));
	}
	.preset-label { color: var(--color-accent-cyan); font-size: 9px; letter-spacing: 1px; }
	button { min-height: 27px; padding: 0 9px; border-radius: 0; cursor: pointer; }
	.apply { border: 1px solid var(--color-accent-cyan); background: rgba(18, 49, 64, 0.86); color: var(--color-accent-cyan); }
	.save { border: 1px solid var(--color-border); background: var(--color-widget-bg); color: var(--color-text-secondary); }
	button:disabled { opacity: 0.5; cursor: not-allowed; }
	.save-row { grid-column: 2 / -1; display: grid; grid-template-columns: 1fr auto; gap: 5px; }
	.save-row input { min-width: 0; height: 27px; padding: 0 7px; border: 1px solid var(--color-border); background: var(--color-bg-deep); color: var(--color-text-primary); }
	.save-row button { border: 1px solid var(--color-accent-cyan); background: rgba(18, 49, 64, 0.86); color: var(--color-accent-cyan); }
	.error { grid-column: 2 / -1; color: #ff6b81; font-size: 8px; }
	@media (max-width: 720px) {
		.preset-bar { grid-template-columns: 78px minmax(0, 1fr) auto; }
		.save { display: none; }
	}
</style>
