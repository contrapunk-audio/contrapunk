<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { transport } from '$lib/stores/transport.svelte';
	import PixelSelect from './PixelSelect.svelte';
	import Knob from './Knob.svelte';
	import { voiceLibrary } from '$lib/stores/voiceLibrary.svelte';

	// Per-voice harmony mode options. Counterpoint expands into the
	// four Fux species — picking "Counterpoint · Species 2" sets both
	// harmony_mode = StrictCounterpoint AND counterpoint_species =
	// Species2 in one selection. Composite values use a colon separator
	// (mode:species); plain modes use just the mode name.
	const MODE_OPTIONS: Array<{ value: string; label: string }> = [
		{ value: '', label: 'Inherit global' },
		{ value: 'PassThrough', label: 'Pass-Through' },
		{ value: 'DiatonicThirds', label: 'Diatonic Thirds' },
		{ value: 'DiatonicFourths', label: 'Diatonic Fourths' },
		{ value: 'ContraryMotion', label: 'Contrary Motion' },
		{ value: 'StrictCounterpoint:Species1', label: 'Counterpoint · Species 1 (note-against-note)' },
		{ value: 'StrictCounterpoint:Species2', label: 'Counterpoint · Species 2 (2:1)' },
		{ value: 'StrictCounterpoint:Species3', label: 'Counterpoint · Species 3 (4:1)' },
		{ value: 'StrictCounterpoint:Species4', label: 'Counterpoint · Species 4 (syncopated)' },
		{ value: 'FunctionalHarmony', label: 'Functional' },
		{ value: 'BachChorale', label: 'Bach Chorale' },
		{ value: 'BarryHarris', label: 'Barry Harris' }
	];

	/** Encode the (mode, species) pair into a composite dropdown value
	 *  matching MODE_OPTIONS. Counterpoint mode includes the species;
	 *  every other mode is just the mode string. Empty/null = inherit. */
	function encodeModeValue(
		harmony_mode: string | null | undefined,
		species: string | null | undefined
	): string {
		if (!harmony_mode) return '';
		if (harmony_mode === 'StrictCounterpoint') {
			return `StrictCounterpoint:${species ?? 'Species1'}`;
		}
		return harmony_mode;
	}

	/** Inverse of encodeModeValue. Returns {harmony_mode, counterpoint_species}
	 *  where species is set only for counterpoint, null otherwise. */
	function decodeModeValue(v: string): {
		harmony_mode: string | null;
		counterpoint_species: string | null;
	} {
		if (!v) return { harmony_mode: null, counterpoint_species: null };
		if (v.startsWith('StrictCounterpoint:')) {
			return {
				harmony_mode: 'StrictCounterpoint',
				counterpoint_species: v.slice('StrictCounterpoint:'.length)
			};
		}
		return { harmony_mode: v, counterpoint_species: null };
	}

	// Knob bounds match canon_lane.rs (delay_beats clamp + time_ratio
	// clamp). 16 beats covers long delays; 0.125 / 8 time-ratio covers
	// extreme augmentation / diminution.
	const DELAY_MIN = 0;
	const DELAY_MAX = 16;
	const DELAY_STEP = 0.125;
	const SPEED_MIN = 0.125;
	const SPEED_MAX = 8;
	const SPEED_STEP = 0.125;

	// One-line plain-language explanations for each harmony mode.
	// Surfaced as tooltips on the Mode dropdown so users learn what
	// each option does without leaving the panel.
	const MODE_HELP: Record<string, string> = {
		PassThrough: 'Pass-Through — no added harmony. The voice just echoes the input pitch.',
		DiatonicThirds:
			'Diatonic Thirds — adds a third above (or below) each note, stepping inside the key.',
		DiatonicFourths: 'Diatonic Fourths — adds a fourth, gives a more open / quartal sound.',
		ContraryMotion:
			'Contrary Motion — moves opposite to the input line. Up → down, down → up.',
		StrictCounterpoint:
			'Strict Counterpoint — Fux-style species rules: avoids parallel 5ths/8ves, prefers stepwise motion.',
		FunctionalHarmony:
			'Functional Harmony — places each input on a chord function (I–IV–V) and harmonizes accordingly.',
		BachChorale: 'Bach Chorale — 4-part chorale-style voicings with smooth voice-leading.',
		BarryHarris:
			'Barry Harris — jazz 6th-diminished system. Alternates 6th and dim7 chords so every scale tone is a chord tone — buttery bebop voice-leading.'
	};

	function modeHelp(name: string): string {
		return MODE_HELP[name] ?? '';
	}

	// --- Forms: each is one preset the user picks from the left rail.
	// Two families today: REACTIVE (canon-style delayed voices) and
	// HARMONIC (synchronous accompaniment via engine config). Future
	// families (looper, drone, arpeggiator) ship as additional lanes.
	type Form = {
		id: string;
		name: string;
		desc: string;
		family: 'reactive' | 'harmonic';
		voices?: Array<{
			delay_beats: number;
			transpose_degrees: number;
			time_ratio: number;
			harmony_mode?: string | null;
			reference_voice?: number | null;
		}>;
		harmonic?: {
			mode: string;
			scale_mode?: string;
			voice_count: number;
			voice_leading_enabled?: boolean;
			voice_leading_style?: string;
		};
	};

	// Built-in templates showcase the harmony engine. Each one assigns
	// per-voice harmony_mode so the voices route through Contrapunk's
	// stateful modes (Counterpoint, BachChorale, ContraryMotion, etc.)
	// rather than just transposing — that's the point of the engine.
	// reference_voice is set wherever the form is genuinely cascaded
	// (V2 follows V1's subject, not the player), null where the form
	// is a simultaneous stack against the player (Functional Pillar,
	// Barry Harris Stack, Contrary Mirror).
	const BUILTIN_TEMPLATES: Form[] = [
		{
			id: 'pure-counterpoint',
			name: 'Pure Counterpoint',
			desc: '4 voices, every voice in StrictCounterpoint · cascading 3rds, 1b apart (SATB)',
			family: 'reactive',
			voices: [
				{
					delay_beats: 1,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: null
				},
				{
					delay_beats: 2,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: 0
				},
				{
					delay_beats: 3,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: 1
				},
				{
					delay_beats: 4,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: 2
				}
			]
		},
		{
			id: 'bach-fugue',
			name: 'Bach Fugue',
			desc: '5-voice fugue · chorale subject → 5th answer → countersubject → octave entry → episode',
			family: 'reactive',
			voices: [
				{
					delay_beats: 0.5,
					transpose_degrees: 0,
					time_ratio: 1.0,
					harmony_mode: 'BachChorale',
					reference_voice: null
				},
				{
					delay_beats: 2,
					transpose_degrees: 4,
					time_ratio: 1.0,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: 0
				},
				{
					delay_beats: 3.5,
					transpose_degrees: 0,
					time_ratio: 1.0,
					harmony_mode: 'BachChorale',
					reference_voice: 1
				},
				{
					delay_beats: 5,
					transpose_degrees: 7,
					time_ratio: 1.0,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: 0
				},
				{
					delay_beats: 6.5,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'BachChorale',
					reference_voice: 2
				}
			]
		},
		{
			id: 'modal-cascade',
			name: 'Modal Cascade',
			desc: '6 voices climbing every harmony mode · Thirds → Fourths → CP → Chorale → Functional → Barry',
			family: 'reactive',
			voices: [
				{
					delay_beats: 0.5,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'DiatonicThirds',
					reference_voice: null
				},
				{
					delay_beats: 1,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'DiatonicFourths',
					reference_voice: 0
				},
				{
					delay_beats: 1.5,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: 1
				},
				{
					delay_beats: 2,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'BachChorale',
					reference_voice: 2
				},
				{
					delay_beats: 2.5,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'FunctionalHarmony',
					reference_voice: 3
				},
				{
					delay_beats: 3,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'BarryHarris',
					reference_voice: 4
				}
			]
		},
		{
			id: 'contrary-mirror',
			name: 'Contrary Mirror',
			desc: '4 voices · fourfold symmetric mirror at 3rd and 6th, above + below',
			family: 'reactive',
			voices: [
				{
					delay_beats: 0.25,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'ContraryMotion',
					reference_voice: null
				},
				{
					delay_beats: 0.25,
					transpose_degrees: -2,
					time_ratio: 1.0,
					harmony_mode: 'ContraryMotion',
					reference_voice: null
				},
				{
					delay_beats: 1,
					transpose_degrees: 5,
					time_ratio: 1.0,
					harmony_mode: 'ContraryMotion',
					reference_voice: 0
				},
				{
					delay_beats: 1,
					transpose_degrees: -5,
					time_ratio: 1.0,
					harmony_mode: 'ContraryMotion',
					reference_voice: 1
				}
			]
		},
		{
			id: 'augmentation-canon',
			name: 'Augmentation Canon',
			desc: '4 voices · subject + 2× slow answer + 4× ultra-slow + counter-augmentation',
			family: 'reactive',
			voices: [
				{
					delay_beats: 0.5,
					transpose_degrees: 0,
					time_ratio: 1.0,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: null
				},
				{
					delay_beats: 1,
					transpose_degrees: 4,
					time_ratio: 2.0,
					harmony_mode: 'BachChorale',
					reference_voice: 0
				},
				{
					delay_beats: 2,
					transpose_degrees: 7,
					time_ratio: 4.0,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: 0
				},
				{
					delay_beats: 1.5,
					transpose_degrees: -3,
					time_ratio: 2.0,
					harmony_mode: 'BachChorale',
					reference_voice: 1
				}
			]
		},
		{
			id: 'diminution-fugue',
			name: 'Diminution Fugue',
			desc: '4 voices · subject + 2× / 4× stretto + counter-diminution · running imitation layers',
			family: 'reactive',
			voices: [
				{
					delay_beats: 0.5,
					transpose_degrees: 0,
					time_ratio: 1.0,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: null
				},
				{
					delay_beats: 1,
					transpose_degrees: 4,
					time_ratio: 0.5,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: 0
				},
				{
					delay_beats: 1.5,
					transpose_degrees: 0,
					time_ratio: 0.25,
					harmony_mode: 'BachChorale',
					reference_voice: 0
				},
				{
					delay_beats: 2,
					transpose_degrees: 7,
					time_ratio: 0.5,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: 1
				}
			]
		},
		{
			id: 'functional-pillar',
			name: 'Functional Pillar',
			desc: '5 voices · full V7 arpeggio · root → 3rd → 5th → 7th → octave restated',
			family: 'reactive',
			voices: [
				{
					delay_beats: 0.25,
					transpose_degrees: 0,
					time_ratio: 1.0,
					harmony_mode: 'FunctionalHarmony',
					reference_voice: null
				},
				{
					delay_beats: 0.5,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'FunctionalHarmony',
					reference_voice: null
				},
				{
					delay_beats: 0.75,
					transpose_degrees: 4,
					time_ratio: 1.0,
					harmony_mode: 'FunctionalHarmony',
					reference_voice: null
				},
				{
					delay_beats: 1,
					transpose_degrees: 6,
					time_ratio: 1.0,
					harmony_mode: 'FunctionalHarmony',
					reference_voice: null
				},
				{
					delay_beats: 1.5,
					transpose_degrees: 7,
					time_ratio: 1.0,
					harmony_mode: 'BachChorale',
					reference_voice: 0
				}
			]
		},
		{
			id: 'barry-harris',
			name: 'Barry Harris Stack',
			desc: '5 voices · 6th-diminished pad · root → 3 → 5 → 6 → octave restatement (V5 ← V1)',
			family: 'reactive',
			voices: [
				{
					delay_beats: 0.25,
					transpose_degrees: 0,
					time_ratio: 1.0,
					harmony_mode: 'BarryHarris',
					reference_voice: null
				},
				{
					delay_beats: 0.5,
					transpose_degrees: 2,
					time_ratio: 1.0,
					harmony_mode: 'BarryHarris',
					reference_voice: null
				},
				{
					delay_beats: 0.75,
					transpose_degrees: 4,
					time_ratio: 1.0,
					harmony_mode: 'BarryHarris',
					reference_voice: null
				},
				{
					delay_beats: 1,
					transpose_degrees: 5,
					time_ratio: 1.0,
					harmony_mode: 'BarryHarris',
					reference_voice: null
				},
				{
					delay_beats: 1.5,
					transpose_degrees: 7,
					time_ratio: 1.0,
					harmony_mode: 'BarryHarris',
					reference_voice: 0
				}
			]
		},
		{
			id: 'mensuration-quartet',
			name: 'Mensuration Quartet',
			desc: '4 voices · 1× / 2× / 0.5× / 4× speeds intertwining · proportional canon (Ockeghem)',
			family: 'reactive',
			voices: [
				{
					delay_beats: 0.5,
					transpose_degrees: 0,
					time_ratio: 1.0,
					harmony_mode: 'StrictCounterpoint',
					reference_voice: null
				},
				{
					delay_beats: 1,
					transpose_degrees: 4,
					time_ratio: 2.0,
					harmony_mode: 'BachChorale',
					reference_voice: 0
				},
				{
					delay_beats: 1.5,
					transpose_degrees: -3,
					time_ratio: 0.5,
					harmony_mode: 'ContraryMotion',
					reference_voice: 0
				},
				{
					delay_beats: 2,
					transpose_degrees: 7,
					time_ratio: 4.0,
					harmony_mode: 'BachChorale',
					reference_voice: 1
				}
			]
		}
	];

	let selectedTemplate = $state<string | null>(null);

	// User-defined forms — saved snapshots of the current voice config.
	// Stored separately from the main settings so they survive schema
	// migrations and a stale Form payload can't break the engine.
	const USER_FORMS_KEY = 'contrapunk-companion-user-forms';
	let userForms = $state<Form[]>(loadUserForms());
	let showSaveForm = $state(false);
	let newFormName = $state('');

	function loadUserForms(): Form[] {
		try {
			const raw = localStorage.getItem(USER_FORMS_KEY);
			if (!raw) return [];
			const parsed = JSON.parse(raw);
			if (!Array.isArray(parsed)) return [];
			return parsed.filter(
				(f): f is Form =>
					f &&
					typeof f.id === 'string' &&
					typeof f.name === 'string' &&
					Array.isArray(f.voices)
			);
		} catch {
			return [];
		}
	}

	function persistUserForms() {
		try {
			localStorage.setItem(USER_FORMS_KEY, JSON.stringify(userForms));
		} catch {
			/* localStorage full / unavailable — ignore */
		}
	}

	function saveCurrentAsForm() {
		const name = newFormName.trim();
		if (!name) return;
		const voices = engine.canonVoices.map((v) => ({
			delay_beats: v.delay_beats,
			transpose_degrees: v.transpose_degrees,
			time_ratio: v.time_ratio
		}));
		const id = `user-${Date.now()}`;
		const desc = `${voices.length} voice${voices.length === 1 ? '' : 's'} — custom`;
		userForms = [...userForms, { id, name, desc, family: 'reactive', voices }];
		persistUserForms();
		newFormName = '';
		showSaveForm = false;
		selectedTemplate = id;
	}

	function deleteUserForm(id: string) {
		userForms = userForms.filter((f) => f.id !== id);
		persistUserForms();
		if (selectedTemplate === id) selectedTemplate = null;
	}

	let TEMPLATES = $derived([...BUILTIN_TEMPLATES, ...userForms]);

	async function applyTemplate(t: Form) {
		selectedTemplate = t.id;
		if (!t.voices) return;
		await engine.setCanonVoices(t.voices);
		if (!engine.companionEnabled) await engine.setCompanionEnabled(true);
		if (!engine.canonEnabled) await engine.setCanonEnabled(true);
	}

	async function toggleCompanion() {
		const next = !engine.companionEnabled;
		await engine.setCompanionEnabled(next);
		// Also flip canon so the master toggle has audible effect.
		await engine.setCanonEnabled(next);
	}

	// --- Readout helpers ---
	function ratioLabel(r: number): string {
		if (Math.abs(r - 1.0) < 0.01) return 'strict';
		if (r < 1.0) return `${(1 / r).toFixed(2)}× faster`;
		return `${r.toFixed(2)}× slower`;
	}

	function transposeLabel(d: number): string {
		if (d === 0) return 'unison';
		const names = ['unison', '2nd', '3rd', '4th', '5th', '6th', '7th', 'octave'];
		const abs = Math.abs(d);
		const name = abs <= 7 ? names[abs] : `${abs}°`;
		const dir = d > 0 ? '↑' : '↓';
		return `${name} ${dir}`;
	}

	// --- Timeline geometry ---
	// Timeline window auto-fits the longest voice delay; minimum window
	// is 2 bars at the active transport time signature, so the grid
	// reflects the actual meter (no hardcoded 4/4 assumption). Each
	// voice gets a horizontal track. Player melody is track 0
	// (always at beat 0). Diminution / augmentation are hinted by a
	// trailing arrow on the dot.
	let timelineBeats = $derived.by(() => {
		const bpb = Math.max(1, transport.beatsPerBar);
		const minWindow = bpb * 2; // 2 bars at current meter
		const maxDelay = engine.canonVoices.reduce(
			(acc, v) => Math.max(acc, v.delay_beats),
			0
		);
		// Round up to the next bar so the rightmost grid line is a bar line.
		const padded = Math.max(minWindow, Math.ceil((maxDelay + 0.5) / bpb) * bpb);
		return padded;
	});

	let gridTicks = $derived.by(() => {
		const bpb = Math.max(1, transport.beatsPerBar);
		const total = timelineBeats;
		return Array.from({ length: total + 1 }, (_, i) => ({
			beat: i,
			isBar: i % bpb === 0,
			barIdx: Math.floor(i / bpb)
		}));
	});

	function entryX(beats: number): number {
		const clamped = Math.max(0, Math.min(timelineBeats, beats));
		return (clamped / timelineBeats) * 100;
	}

	// --- Drag-to-set-delay on timeline dots ---
	// Hold pointer on a voice's dot and drag horizontally — the
	// voice's delay_beats follows the cursor. The rail's width and
	// the active timelineBeats snapshot are frozen at drag-start so
	// the auto-fit doesn't fight the cursor mid-drag. Step snaps to
	// 16th notes (0.125b); hold Shift for free positioning.
	let drag = $state<{
		voiceIdx: number;
		railLeft: number;
		railWidth: number;
		beatsAtDragStart: number;
	} | null>(null);

	function onDotPointerDown(e: PointerEvent, voiceIdx: number) {
		// The dot lives inside .track-rail. Walk up to find the rail.
		const dot = e.currentTarget as HTMLElement;
		const rail = dot.parentElement;
		if (!rail) return;
		const rect = rail.getBoundingClientRect();
		dot.setPointerCapture(e.pointerId);
		drag = {
			voiceIdx,
			railLeft: rect.left,
			railWidth: rect.width,
			beatsAtDragStart: timelineBeats
		};
		e.preventDefault();
	}

	function onDotPointerMove(e: PointerEvent) {
		if (!drag) return;
		const dx = e.clientX - drag.railLeft;
		const ratio = Math.max(0, Math.min(1, dx / Math.max(1, drag.railWidth)));
		let beats = ratio * drag.beatsAtDragStart;
		if (!e.shiftKey) {
			beats = Math.round(beats / DELAY_STEP) * DELAY_STEP;
		}
		beats = Math.max(DELAY_MIN, Math.min(DELAY_MAX, beats));
		engine.updateCanonVoice(drag.voiceIdx, { delay_beats: beats });
	}

	function onDotPointerUp(e: PointerEvent) {
		if (!drag) return;
		const dot = e.currentTarget as HTMLElement;
		try {
			dot.releasePointerCapture(e.pointerId);
		} catch {
			/* pointer wasn't actually captured — ignore */
		}
		drag = null;
	}

	// --- Live-binding: when a Voice Library preset is edited in the
	// Voices tab, any canon voice bound to that preset (via preset_id)
	// re-applies the preset's fields automatically. The guard checks
	// for genuine differences so the effect doesn't feed back when the
	// fields already match.
	$effect(() => {
		for (let i = 0; i < engine.canonVoices.length; i++) {
			const v = engine.canonVoices[i];
			if (!v.preset_id) continue;
			const p = voiceLibrary.byId(v.preset_id);
			if (!p) continue;
			const diff =
				v.harmony_mode !== p.harmony_mode ||
				v.voice_count !== p.voice_count ||
				v.voice_position !== p.voice_position ||
				v.voice_leading_enabled !== p.voice_leading_enabled ||
				v.voice_leading_style !== p.voice_leading_style ||
				v.octave_mode !== p.octave_mode ||
				v.counterpoint_species !== p.counterpoint_species ||
				v.counterpoint_strictness !== p.counterpoint_strictness;
			if (diff) {
				engine.updateCanonVoice(i, {
					harmony_mode: p.harmony_mode,
					voice_count: p.voice_count,
					voice_position: p.voice_position,
					voice_leading_enabled: p.voice_leading_enabled,
					voice_leading_style: p.voice_leading_style,
					octave_mode: p.octave_mode,
					counterpoint_species: p.counterpoint_species,
					counterpoint_strictness: p.counterpoint_strictness
				});
			}
		}
	});
</script>

<div class="companion-root">
	<!-- HEADER: single master toggle. Canon is implicit today. -->
	<header class="header">
		<div class="header-left">
			<h2 class="title font-ui">Companion</h2>
			<span class="subtitle font-code">reactive companions — canon today, more lanes incoming</span>
		</div>
		<button
			class="pixel-btn toggle-btn master-toggle"
			class:toggle-on={engine.companionEnabled}
			onclick={toggleCompanion}
			title={engine.companionEnabled
				? 'Companion is ON — voices will play in response to your input. Click to disable.'
				: 'Companion is OFF. Click to enable reactive voices.'}
		>
			{engine.companionEnabled ? 'ON' : 'OFF'}
		</button>
	</header>

	<!-- BODY: templates left, visual + voices right -->
	<div class="body">
		<!-- LEFT: forms rail. Flat list — the underlying family
		     (harmonic vs reactive) is implicit in what each form does;
		     the user said the labels were noise. Reactive forms
		     populate the timeline + voice cards on the right; harmonic
		     forms reconfigure the engine and don't surface here. -->
		<aside class="templates">
			<div
				class="section-header font-ui"
				title="Forms are voice presets. Built-ins showcase Contrapunk's harmony engine; you can save your own with +Save."
			>
				FORMS
				<button
					class="pixel-btn"
					onclick={() => {
						showSaveForm = !showSaveForm;
						newFormName = '';
					}}
					title="Save the current voice configuration as a custom form. Stored locally — survives reload."
				>
					+ Save
				</button>
			</div>

			{#if showSaveForm}
				<div class="save-form font-code">
					<input
						type="text"
						bind:value={newFormName}
						placeholder="Form name…"
						class="pixel-input"
						onkeydown={(e) => {
							if (e.key === 'Enter') saveCurrentAsForm();
							if (e.key === 'Escape') showSaveForm = false;
						}}
					/>
					<button class="pixel-btn" onclick={saveCurrentAsForm} disabled={!newFormName.trim()}>
						Save
					</button>
				</div>
			{/if}

			{#each TEMPLATES as t (t.id)}
				{@const isUser = t.id.startsWith('user-')}
				<div class="template-row-wrap" class:user-form={isUser}>
					<button
						class="template-row"
						class:active={selectedTemplate === t.id}
						onclick={() => applyTemplate(t)}
						title={t.desc}
					>
						<div class="template-name font-ui">
							{t.name}
							{#if isUser}<span class="user-badge font-code">USER</span>{/if}
						</div>
						<div class="template-desc font-code">{t.desc}</div>
					</button>
					{#if isUser}
						<button
							class="pixel-btn delete-form"
							onclick={() => deleteUserForm(t.id)}
							title="Delete this form"
						>
							×
						</button>
					{/if}
				</div>
			{/each}
		</aside>

		<!-- RIGHT: visualization + voice cards. Always renders the same
		     layout so the UI doesn't lurch when no voices are
		     configured — the empty case just has zero voice tracks /
		     zero voice cards and the Add Voice button stays prominent. -->
		<section class="right">
			<!-- ENGINE STATUS — gives harmonic forms a visible effect.
			     Reactive forms only change canon voices (timeline below);
			     harmonic forms change engine.mode / voiceCount /
			     voiceLeading globally, which without this row would be
			     invisible on the Companion tab. -->
			<div
				class="engine-status"
				title={`Global engine state — what each Companion voice inherits when its Mode is set to "Inherit global". Change these on the Harmony tab.\n\nKey: ${engine.key}\nMode: ${engine.mode}\nVoices: ${engine.voiceCount}${engine.voiceLeadingEnabled ? `\nVoice-leading: ${engine.voiceLeadingStyle}` : ''}`}
			>
				<span class="status-label font-ui">ENGINE</span>
				<span class="status-pill font-code">
					<span class="pill-key">{engine.key}</span> ·
					<span class="pill-mode">{engine.mode}</span> ·
					{engine.voiceCount} voice{engine.voiceCount === 1 ? '' : 's'}{#if engine.voiceLeadingEnabled} · VL {engine.voiceLeadingStyle}{/if}
				</span>
			</div>

			<!-- TIMELINE VISUALIZATION -->
			<div
				class="timeline-card"
				title="When you play a note, each voice enters at its delay offset. The grid is measured in bars / beats of the active transport time signature, not arbitrary 4/4."
			>
				<div class="section-header font-ui">
					ENTRY TIMELINE
					<span class="hint font-code">
						player → delayed voices ({timelineBeats} beat{timelineBeats === 1 ? '' : 's'} · {transport.beatsPerBar}/{transport.beatUnit})
					</span>
				</div>
				<div class="timeline">
					<!-- Beat / bar grid lines. Every beat is labeled with
					     its beat-within-bar number (1..bpb). Bar boundaries
					     are visually brighter; the label at a bar boundary
					     is the bar number, otherwise the beat-within-bar. -->
					{#each gridTicks as t (t.beat)}
						{@const beatInBar = (t.beat % Math.max(1, transport.beatsPerBar)) + 1}
						<div
							class="grid-line"
							class:bar-line={t.isBar}
							style="left: {(t.beat / timelineBeats) * 100}%"
						>
							<span class="grid-label font-code" class:bar-label={t.isBar}>
								{t.isBar ? `${t.barIdx + 1}` : beatInBar}
							</span>
						</div>
					{/each}

					<!-- Player track (always at beat 0) -->
					<div class="track player-track">
						<span class="track-label font-ui">PLAYER</span>
						<div class="track-rail">
							<div class="entry-dot player-dot" style="left: 0%" title="Player melody enters at beat 0">
								◉
							</div>
						</div>
					</div>

					<!-- Canon voice tracks -->
					{#each engine.canonVoices as voice, i (i)}
						{@const ref =
							voice.reference_voice != null ? `V${voice.reference_voice + 1}` : 'Player'}
						{@const mode = voice.harmony_mode ?? `${engine.mode} (inherited)`}
						<div
							class="track voice-track"
							title={`Voice ${i + 1}\n• enters ${voice.delay_beats.toFixed(2)} beats after ${ref}\n• transposes ${transposeLabel(voice.transpose_degrees)}\n• plays at ${ratioLabel(voice.time_ratio)}\n• harmony mode: ${mode}\n• follows: ${ref}\n\nDrag the dot to set the delay (Shift = free, no snap).`}
						>
							<span class="track-label font-ui">V{i + 1}</span>
							<div class="track-rail">
								<!-- svelte-ignore a11y_no_static_element_interactions a11y_no_noninteractive_element_interactions -->
								<div
									class="entry-dot voice-dot draggable"
									class:dragging={drag?.voiceIdx === i}
									class:transpose-pos={voice.transpose_degrees > 0}
									class:transpose-neg={voice.transpose_degrees < 0}
									style="left: {entryX(voice.delay_beats)}%"
									role="slider"
									tabindex="0"
									aria-label="Voice {i + 1} delay in beats"
									aria-valuemin={DELAY_MIN}
									aria-valuemax={DELAY_MAX}
									aria-valuenow={voice.delay_beats}
									onpointerdown={(e) => onDotPointerDown(e, i)}
									onpointermove={onDotPointerMove}
									onpointerup={onDotPointerUp}
									onpointercancel={onDotPointerUp}
								>
									{voice.time_ratio < 1
										? '◀'
										: voice.time_ratio > 1
											? '▶'
											: '◉'}
									{#if drag?.voiceIdx === i}
										<span class="drag-readout font-code">
											{voice.delay_beats.toFixed(2)}b
										</span>
									{/if}
								</div>
							</div>
						</div>
					{/each}
				</div>
			</div>

			<!-- VOICE CARDS -->
			<div class="voices-card">
				<div
					class="section-header font-ui"
					title="One card per canon voice. Each voice has its own delay, time-scale, harmony mode, and cascade target. Up to 8 voices."
				>
					VOICES ({engine.canonVoices.length})
					<button
						class="pixel-btn"
						disabled={engine.canonVoices.length >= 8}
						onclick={() => engine.addCanonVoice()}
						title="Add a new canon voice (up to 8 total). Defaults to a 1-beat unison echo."
					>
						+ Add Voice
					</button>
				</div>
				<div class="voice-grid">
					{#each engine.canonVoices as voice, i (i)}
						<div class="voice-card">
							<div
								class="voice-header"
								title="Voice {i + 1} of {engine.canonVoices.length}"
							>
								<span class="voice-label font-ui">V{i + 1}</span>
								<button
									class="pixel-btn remove-btn"
									disabled={engine.canonVoices.length <= 1}
									onclick={() => engine.removeCanonVoice(i)}
									title="Delete this voice. (At least one voice must remain.)"
								>
									×
								</button>
							</div>

							<div class="voice-knobs">
								<div
									title="Delay — how many beats this voice waits before entering, measured from the phrase anchor (or its reference voice's entry). Drag vertically · Shift = fine · scroll-wheel = step · dbl-click = reset to 1. Range 0–16 beats."
								>
									<Knob
										value={voice.delay_beats}
										min={DELAY_MIN}
										max={DELAY_MAX}
										step={DELAY_STEP}
										defaultValue={1}
										label="Delay"
										accent="var(--color-accent-magenta, #ff33aa)"
										size={56}
										format={(v) =>
											v < 1 ? `${v.toFixed(2)}b` : `${v.toFixed(2)}b`}
										onchange={(v) => engine.updateCanonVoice(i, { delay_beats: v })}
									/>
								</div>
								<div
									title="Speed — time-stretch factor. 1× strict imitation. 2× = augmentation (plays at half speed, notes twice as long). 0.5× = diminution (double speed). Bach used both in his Art of Fugue. Range 0.125×–8×."
								>
									<Knob
										value={voice.time_ratio}
										min={SPEED_MIN}
										max={SPEED_MAX}
										step={SPEED_STEP}
										defaultValue={1}
										label="Speed"
										accent="var(--color-accent-cyan, #33ddff)"
										size={56}
										format={(v) =>
											Math.abs(v - 1) < 0.01
												? '1×'
												: v < 1
													? `${(1 / v).toFixed(2)}×↑`
													: `${v.toFixed(2)}×↓`}
										onchange={(v) => engine.updateCanonVoice(i, { time_ratio: v })}
									/>
								</div>
							</div>

							<div
								class="voice-param voice-param-mode"
								title="Preset — bind this voice to a named Voice Library entry. Edits to the preset in the Voices tab live-update every bound canon voice. Pick 'Unbind' to detach (the voice keeps its current sound but stops following the preset)."
							>
								<span class="param-label font-ui">Preset</span>
								<PixelSelect
									options={[
										{ value: '', label: '— unbind —' },
										...voiceLibrary.all.map((p) => ({
											value: p.id,
											label: `${p.builtIn ? '' : '★ '}${p.name}`
										}))
									]}
									value={voice.preset_id ?? ''}
									placeholder="— unbind —"
									small={true}
									onchange={(v) => {
										if (!v) {
											engine.updateCanonVoice(i, { preset_id: null });
											return;
										}
										const preset = voiceLibrary.byId(v);
										if (!preset) return;
										engine.updateCanonVoice(i, {
											preset_id: preset.id,
											harmony_mode: preset.harmony_mode,
											voice_count: preset.voice_count,
											voice_position: preset.voice_position,
											voice_leading_enabled: preset.voice_leading_enabled,
											voice_leading_style: preset.voice_leading_style,
											octave_mode: preset.octave_mode,
											counterpoint_species: preset.counterpoint_species,
											counterpoint_strictness: preset.counterpoint_strictness
										});
									}}
								/>
							</div>

							<div
								class="voice-param voice-param-mode"
								title={`Mode — harmony engine routed for this voice's stack.\n\n${voice.harmony_mode ? `Currently: ${voice.harmony_mode}\n${modeHelp(voice.harmony_mode)}` : `Currently: inheriting the global engine mode (${engine.mode}).\n${modeHelp(engine.mode)}`}\n\nCounterpoint expands into the four Fux species — pick "Counterpoint · Species N" to set both at once. Counterpoint strictness is always Strict (no Relaxed mode).`}
							>
								<span class="param-label font-ui">Mode</span>
								<PixelSelect
									options={MODE_OPTIONS}
									value={encodeModeValue(voice.harmony_mode, voice.counterpoint_species)}
									placeholder="Inherit"
									small={true}
									onchange={(v) => {
										const decoded = decodeModeValue(v);
										engine.updateCanonVoice(i, {
											harmony_mode: decoded.harmony_mode,
											counterpoint_species: decoded.counterpoint_species,
											counterpoint_strictness: 'Strict'
										});
									}}
								/>
							</div>

							<div
								class="voice-param voice-param-mode"
								title={`Relative — which voice this one harmonizes against.\n\nPlayer = this voice mirrors the user's input.\nV1/V2/… = this voice cascades off an earlier voice's emitted subject (fugue-style chain).\n\nOnly voices defined before this one are available — references must point backwards to avoid cycles.`}
							>
								<span class="param-label font-ui">Relative</span>
								<PixelSelect
									options={[
										{ value: '', label: 'Player' },
										...Array.from({ length: i }, (_, k) => ({
											value: String(k),
											label: `V${k + 1}`
										}))
									]}
									value={voice.reference_voice != null ? String(voice.reference_voice) : ''}
									placeholder="Player"
									small={true}
									onchange={(v) =>
										engine.updateCanonVoice(i, {
											reference_voice: v === '' ? null : parseInt(v, 10)
										})}
								/>
							</div>
						</div>
					{/each}
				</div>
			</div>

			<!-- FOOTER HINT -->
			<div class="footer-hint font-code">
				Voices fire relative to a phrase anchor. Phrase resets after 2 beats of silence.
				Interval uses the engine's modal-interchange logic when enabled — out-of-scale input borrows
				from a parallel mode rather than emitting bare unison. Transport must be playing for voices
				to fire.
			</div>
		</section>
	</div>
</div>

<style>
	.companion-root {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border);
		background: var(--color-widget-bg);
	}

	.header-left {
		display: flex;
		align-items: baseline;
		gap: 8px;
	}

	.title {
		margin: 0;
		font-size: var(--font-size-lg);
		color: var(--color-accent-magenta, #ff33aa);
	}

	.subtitle {
		font-size: var(--font-size-xs);
		opacity: 0.6;
	}

	.master-toggle {
		min-width: 70px;
	}

	.body {
		display: grid;
		grid-template-columns: 220px 1fr;
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.templates {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px;
		border-right: 1px solid var(--color-border);
		overflow-y: auto;
		background: rgba(15, 14, 26, 0.5);
	}

	.section-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.1em;
		margin-bottom: 6px;
	}

	.template-row-wrap {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 2px;
		align-items: stretch;
	}

	.template-row {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 1px;
		padding: 6px 8px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		text-align: left;
		cursor: pointer;
		color: inherit;
		min-width: 0;
	}

	.template-row:hover {
		border-color: var(--color-accent-cyan, #33ddff);
	}

	.template-row.active {
		border-color: var(--color-accent-magenta, #ff33aa);
		background: rgba(255, 51, 170, 0.1);
	}

	.user-badge {
		font-size: 0.6em;
		color: var(--color-accent-cyan, #33ddff);
		opacity: 0.8;
		letter-spacing: 0.1em;
		margin-left: 4px;
	}

	.delete-form {
		font-size: var(--font-size-sm);
		padding: 2px 6px;
		min-width: 0;
		align-self: center;
	}

	.save-form {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 4px;
		padding: 4px 0 6px;
		border-bottom: 1px dashed rgba(255, 255, 255, 0.1);
		margin-bottom: 4px;
	}

	.pixel-input {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		color: var(--color-text);
		padding: 4px 6px;
		font-size: var(--font-size-xs);
		font-family: inherit;
	}
	.pixel-input:focus {
		outline: none;
		border-color: var(--color-accent-cyan, #33ddff);
	}

	.template-name {
		font-size: var(--font-size-sm);
		color: var(--color-accent-cyan, #33ddff);
	}

	.template-desc {
		font-size: var(--font-size-xs);
		opacity: 0.6;
		line-height: 1.3;
	}

	.right {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 8px;
		overflow-y: auto;
	}

	.engine-status {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 8px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
	}

	.status-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		letter-spacing: 0.1em;
	}

	.status-pill {
		font-size: var(--font-size-xs);
		opacity: 0.85;
	}

	.pill-key,
	.pill-mode {
		color: var(--color-accent-cyan, #33ddff);
	}

	.timeline-card,
	.voices-card {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		padding: 8px;
	}

	.hint {
		font-size: var(--font-size-xs);
		opacity: 0.5;
		font-weight: normal;
		text-transform: none;
		letter-spacing: 0;
	}

	.timeline {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 18px 8px 8px;
		background: var(--color-bg);
		border: 1px solid var(--color-border);
	}

	.grid-line {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 1px;
		background: rgba(255, 255, 255, 0.06);
		pointer-events: none;
	}

	.grid-line.bar-line {
		background: rgba(255, 255, 255, 0.18);
		width: 1px;
	}

	.grid-label {
		position: absolute;
		top: 0;
		transform: translateX(-50%);
		font-size: 0.6em;
		color: var(--color-text-secondary);
		opacity: 0.4;
	}

	.grid-label.bar-label {
		opacity: 0.95;
		color: var(--color-accent-cyan, #33ddff);
		font-size: 0.7em;
	}

	.track {
		display: grid;
		grid-template-columns: 60px 1fr;
		align-items: center;
		gap: 8px;
		min-height: 22px;
	}

	.track-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		text-align: right;
	}

	.player-track .track-label {
		color: var(--color-accent-cyan, #33ddff);
	}

	.voice-track .track-label {
		color: var(--color-accent-magenta, #ff33aa);
	}

	.track-rail {
		position: relative;
		height: 16px;
		background: rgba(255, 255, 255, 0.04);
		border: 1px solid rgba(255, 255, 255, 0.08);
	}

	.entry-dot {
		position: absolute;
		top: 50%;
		transform: translate(-50%, -50%);
		font-size: 14px;
		line-height: 1;
		pointer-events: auto;
	}

	.entry-dot.draggable {
		cursor: grab;
		touch-action: none;
		user-select: none;
	}

	.entry-dot.dragging {
		cursor: grabbing;
		filter: drop-shadow(0 0 6px currentColor);
	}

	.drag-readout {
		position: absolute;
		top: -16px;
		left: 50%;
		transform: translateX(-50%);
		padding: 1px 5px;
		font-size: 0.65em;
		background: rgba(15, 14, 26, 0.95);
		border: 1px solid currentColor;
		color: var(--color-text);
		white-space: nowrap;
		pointer-events: none;
		z-index: 10;
	}

	.player-dot {
		color: var(--color-accent-cyan, #33ddff);
	}

	.voice-dot {
		color: var(--color-accent-magenta, #ff33aa);
	}

	.voice-dot.transpose-pos {
		color: var(--color-accent-cyan, #33ddff);
	}

	.voice-dot.transpose-neg {
		color: #ffaa33;
	}

	.voice-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 6px;
	}

	.voice-card {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 6px 8px;
		background: var(--color-bg);
		border: 1px solid var(--color-border);
	}

	.voice-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.voice-label {
		color: var(--color-accent-magenta, #ff33aa);
		font-size: var(--font-size-md);
	}

	.remove-btn {
		font-size: var(--font-size-sm);
		padding: 2px 6px;
	}

	.voice-knobs {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 4px;
		justify-items: center;
		padding: 4px 0 6px;
	}

	.voice-param {
		display: grid;
		grid-template-columns: 4.5em 1fr 5.5em;
		align-items: center;
		gap: 4px;
	}

	.param-label {
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
	}

	.voice-param-mode {
		grid-template-columns: 4.5em 1fr;
	}

	.footer-hint {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		opacity: 0.5;
		line-height: 1.4;
		padding: 4px 8px;
	}
</style>
