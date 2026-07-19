<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { arrangement } from '$lib/stores/arrangement.svelte';
	import { transport } from '$lib/stores/transport.svelte';
	import { adapter } from '$lib/adapter';
	import PixelSelect from './PixelSelect.svelte';
	import Knob from './Knob.svelte';
	import { voiceLibrary } from '$lib/stores/voiceLibrary.svelte';
	import { compositeModeOptions, encodeMode, decodeMode } from '$lib/harmony/modeComposite';

	let {
		focusGroup = 'imitative',
		focusVersion = 0
	}: {
		focusGroup?: 'imitative' | 'species';
		focusVersion?: number;
	} = $props();

	// Per-voice harmony mode options. Reuses the same composite list as
	// the Harmony tab + Voices tab, plus an "Inherit global" entry at
	// the top for the canon's per-voice override.
	const MODE_OPTIONS: Array<{ value: string; label: string }> = [
		{ value: '', label: 'Inherit global' },
		...compositeModeOptions((s) => s)
	];

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
			desc: '3 voices · cascading strict counterpoint · each voice inherits the previous voice\'s CounterpointState (interval history, contour, pitch buffer) before picking its own harmony',
			family: 'reactive',
			voices: [
				// True stateful cascade: each voice's mini-engine
				// receives the prior voice's CounterpointState before
				// generating, so V2's rule scoring sees V1's emitted
				// pitch in `harmony_pitch_buffer` and avoids parallels
				// with it. Over multiple notes the chain accumulates
				// real polyphonic memory rather than each voice
				// computing in isolation.
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
	let selectedGroup = $state<'imitative' | 'species'>('imitative');
	let selectedVoice = $state(0);

	$effect(() => {
		void focusVersion;
		selectedGroup = focusGroup;
	});

	$effect(() => {
		if (selectedVoice >= engine.canonVoices.length) {
			selectedVoice = Math.max(0, engine.canonVoices.length - 1);
		}
	});

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
		if (engine.imitativeForm !== 'free_imitation') {
			await engine.setImitativeForm('free_imitation');
		}
		await engine.setCanonVoices(t.voices);
		if (!engine.companionEnabled) await engine.setCompanionEnabled(true);
		if (!engine.canonEnabled) await engine.setCanonEnabled(true);
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

	function setGlobalHold(kind: string) {
		if (kind === 'near_future') {
			engine.setCompanionHoldMode({ kind: 'near_future', tail_beats: 1 });
		} else {
			engine.setCompanionHoldMode({ kind: kind as 'cancel' | 'phrase_end' | 'forever' });
		}
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

	// CounterpointLane state is shared with arrangement snapshots and
	// preset application instead of living only inside this component.
	$effect(() => {
		void arrangement.syncFromBackend();
	});

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
	<header class="header">
		<div class="header-left">
			<h2 class="title font-ui">Counterpoint</h2>
			<span class="subtitle font-code">one focused group at a time</span>
		</div>
		<div class="header-controls">
			<label class="compact-pick">
				<span class="font-ui">GROUP</span>
				<select bind:value={selectedGroup}>
					<option value="imitative">1 · Imitative Counterpoint</option>
					<option value="species">2 · Species Counterpoint</option>
				</select>
			</label>
			<label class="compact-pick" title="What pending counterpoint notes do when you release the source note.">
				<span class="font-ui">HOLD</span>
				<select value={engine.companionHoldMode.kind} onchange={(e) => setGlobalHold((e.target as HTMLSelectElement).value)}>
					<option value="cancel">Cancel immediately</option>
					<option value="near_future">Finish near notes</option>
					<option value="phrase_end">Finish phrase</option>
					<option value="forever">Always finish</option>
				</select>
			</label>
			{#if engine.companionHoldMode.kind === 'near_future'}
				<label class="compact-pick tail-pick">
					<span class="font-ui">TAIL</span>
					<input
						type="number"
						min="0"
						max="32"
						step="0.25"
						value={engine.companionHoldMode.tail_beats}
						oninput={(e) => {
							const value = parseFloat((e.target as HTMLInputElement).value);
							if (!isNaN(value) && value >= 0 && value <= 32) engine.setCompanionHoldMode({ kind: 'near_future', tail_beats: value });
						}}
					/>
				</label>
			{/if}
			<span class="group-status font-ui">{engine.companionEnabled ? '● ACTIVE' : 'READY'}</span>
		</div>
	</header>

	<!-- BODY: vertical stack of Lane cards. Each lane is a self-
	     contained module — header (name + enable toggle), body
	     (lane-specific controls). The engine-status pill sits above
	     the lanes since it reflects global Harmony-tab state, not a
	     per-lane concern. -->
	<div class="body-stack">
		<!-- ENGINE STATUS — global Harmony-tab snapshot, shared by all
		     lanes that inherit. Placed above the lane cards so it reads
		     as context, not as a lane-specific control. -->
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

		<!-- Only the selected group is expanded. The other group keeps
		     running; this selector changes the editor, not the music. -->
		{#if selectedGroup === 'imitative'}
		<div
			class="lane-card canon-lane-card"
			class:lane-active={engine.canonEnabled}
		>
			<div class="lane-header">
				<span class="lane-title font-ui">COUNTERPOINT GROUP 1</span>
				<span class="lane-subtitle font-code">Imitative counterpoint · {engine.imitativeForm === 'strict_canon' ? 'Strict Canon' : 'Free Imitation'}</span>
				<span class="lane-actions">
					<label class="lane-hold-pick" title="Strict Canon preserves one subject exactly; Free Imitation unlocks independent voice transformations.">
						<span class="font-code">FORM</span>
						<select
							value={engine.imitativeForm}
							onchange={(e) => engine.setImitativeForm((e.target as HTMLSelectElement).value as 'strict_canon' | 'free_imitation')}
						>
							<option value="strict_canon">Strict Canon</option>
							<option value="free_imitation">Free Imitation</option>
						</select>
					</label>
					<label class="lane-hold-pick" title="Hold behavior for the whole Canon group. 'Inherit' uses the global setting.">
						<span class="font-code">HOLD</span>
						<select
							value={engine.canonLaneHoldMode === null ? 'inherit' : engine.canonLaneHoldMode.kind}
							onchange={(e) => {
								const v = (e.target as HTMLSelectElement).value;
								if (v === 'inherit') {
									engine.setCanonLaneHoldMode(null);
								} else if (v === 'near_future') {
									engine.setCanonLaneHoldMode({ kind: 'near_future', tail_beats: 1.0 });
								} else {
									engine.setCanonLaneHoldMode({ kind: v as 'cancel' | 'phrase_end' | 'forever' });
								}
							}}
						>
							<option value="inherit">Inherit</option>
							<option value="cancel">Cancel</option>
							<option value="near_future">Near</option>
							<option value="phrase_end">Phrase</option>
							<option value="forever">Forever</option>
						</select>
					</label>
					<button
						class="pixel-btn"
						class:toggle-on={engine.canonEnabled}
						onclick={async () => {
							const enabled = !engine.canonEnabled;
							if (enabled && !engine.companionEnabled) await engine.setCompanionEnabled(true);
							await engine.setCanonEnabled(enabled);
						}}
						title={engine.canonEnabled
							? 'Canon Lane is ON — delayed voices fire on every player note.'
							: 'Canon Lane is OFF. Click to enable.'}
					>
						{engine.canonEnabled ? 'ON' : 'OFF'}
					</button>
				</span>
			</div>

			<div class="lane-body canon-lane-body">
			{#if engine.imitativeForm === 'strict_canon'}
				<div class="strict-contract font-code">
					◆ STRICT IMITATION · Subject = You Play · rhythm and melodic shape locked · delay and transposition remain editable
				</div>
			{/if}
			<div class="subject-map" aria-label="Imitative counterpoint voice map">
				<div class="subject-node">
					<div class="map-node-head font-ui"><strong>YOU PLAY</strong><span>SUBJECT</span></div>
					<div class="map-node-main font-code">Live melody</div>
					<div class="map-node-meta font-code">rhythm · contour · phrasing</div>
				</div>
				<div class="subject-arrow font-ui" aria-hidden="true">FOLLOWS ›</div>
				<div class="followers-map">
					<div class="followers-head font-ui"><span>FOLLOWER VOICES</span><span>{engine.canonVoices.length} ACTIVE</span></div>
					<div class="follower-grid">
						{#each engine.canonVoices as voice, index}
							<button
								class="follower-node"
								class:selected={selectedVoice === index}
								type="button"
								title={`Edit Voice ${index + 1}: enters after ${voice.delay_beats} beats, ${transposeLabel(voice.transpose_degrees)}`}
								onclick={() => (selectedVoice = index)}
							>
								<span class="follower-name font-ui">VOICE {index + 1}</span>
								<strong class="font-code">+{voice.delay_beats}b · {transposeLabel(voice.transpose_degrees)}</strong>
								<small class="font-code">{voice.time_ratio === 1 ? 'same rhythm' : ratioLabel(voice.time_ratio)}</small>
							</button>
						{/each}
					</div>
				</div>
			</div>
			<div class="preset-toolbar">
				<label class="compact-pick preset-pick">
					<span class="font-ui">GROUP PRESET</span>
					<select
						value={selectedTemplate ?? ''}
						onchange={(e) => {
							const template = TEMPLATES.find((item) => item.id === (e.target as HTMLSelectElement).value);
							if (template) applyTemplate(template);
						}}
					>
						<option value="" disabled>Choose a preset…</option>
						<optgroup label="Built in">
							{#each TEMPLATES.filter((item) => !item.id.startsWith('user-')) as template}
								<option value={template.id}>{template.name}</option>
							{/each}
						</optgroup>
						{#if userForms.length}
							<optgroup label="My presets">
								{#each userForms as template}
									<option value={template.id}>{template.name}</option>
								{/each}
							</optgroup>
						{/if}
					</select>
				</label>
				{#if selectedTemplate}
					<span class="preset-description font-code">{TEMPLATES.find((item) => item.id === selectedTemplate)?.desc ?? ''}</span>
				{/if}
				<button class="pixel-btn" onclick={() => { showSaveForm = !showSaveForm; newFormName = ''; }}>SAVE AS</button>
				{#if selectedTemplate?.startsWith('user-')}
					<button class="pixel-btn delete-form" onclick={() => { if (selectedTemplate) deleteUserForm(selectedTemplate); }} title="Delete selected preset">DELETE</button>
				{/if}
			</div>
			{#if showSaveForm}
				<div class="save-form compact-save font-code">
					<input type="text" bind:value={newFormName} placeholder="Preset name…" class="pixel-input" onkeydown={(e) => { if (e.key === 'Enter') saveCurrentAsForm(); if (e.key === 'Escape') showSaveForm = false; }} />
					<button class="pixel-btn" onclick={saveCurrentAsForm} disabled={!newFormName.trim()}>SAVE</button>
				</div>
			{/if}

		<!-- CANON RIGHT — timeline + voice cards. Lives inside the
		     canon-lane-body alongside the forms rail. Empty state still
		     renders the layout so the UI doesn't lurch. -->
		<section class="canon-right">
			<!-- TIMELINE VISUALIZATION -->
			<details
				class="timeline-card"
				title="When you play a note, each voice enters at its delay offset."
			>
				<summary class="section-header font-ui">
					<span>ENTRY TIMELINE</span>
					<span class="hint font-code">
						{engine.canonVoices.length} voices · {timelineBeats} beats · click to expand
					</span>
				</summary>
				<div class="timeline">
					<!-- Beat / bar grid lines. Every beat is labeled with
					     its beat-within-bar number (1..bpb). Bar boundaries
					     are visually brighter; the label at a bar boundary
					     is the bar number, otherwise the beat-within-bar.

					     The overlay div is `position: absolute` and sized
					     to exactly match every .track-rail's horizontal
					     extent (left = timeline padding + label column +
					     gap; right = timeline padding). Grid-lines
					     positioned at `left: X%` therefore share one
					     coordinate space with the rail dots, eliminating
					     the prior misalignment caused by absolutely-
					     positioned children resolving percentages
					     against the full timeline padding box rather
					     than the rail content area. -->
					<div class="grid-overlay">
						{#each gridTicks as t (t.beat)}
							{@const beatInBar = (t.beat % Math.max(1, transport.beatsPerBar)) + 1}
							<div
								class="grid-line"
								class:bar-line={t.isBar}
								style="left: {(t.beat / timelineBeats) * 100}%"
								title={t.isBar
									? `Bar ${t.barIdx + 1} · beat 1`
									: `Bar ${t.barIdx + 1} · beat ${beatInBar}`}
							>
								<span class="grid-label font-code" class:bar-label={t.isBar}>
									{beatInBar}
								</span>
							</div>
						{/each}
					</div>

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
			</details>

			<!-- VOICE CARDS -->
			<div class="voices-card">
				<div
					class="section-header voice-editor-header font-ui"
					title="Edit one voice at a time. Every voice remains visible in the timeline and continues playing."
				>
					<span>VOICE EDITOR</span>
					<label class="compact-pick voice-pick">
						<span>VOICE</span>
						<select value={selectedVoice} onchange={(e) => (selectedVoice = Number((e.target as HTMLSelectElement).value))}>
							{#each engine.canonVoices as _, index}
								<option value={index}>Voice {index + 1}</option>
							{/each}
						</select>
					</label>
					<button
						class="pixel-btn"
						disabled={engine.canonVoices.length >= 8}
						onclick={async () => {
							const next = engine.canonVoices.length;
							await engine.addCanonVoice();
							selectedVoice = next;
						}}
						title="Add a new canon voice (up to 8 total)."
					>
						+ ADD
					</button>
				</div>
				<div class="voice-grid">
					{#each engine.canonVoices as voice, i (i)}
						<div class="voice-card" hidden={i !== selectedVoice}>
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
										disabled={engine.imitativeForm === 'strict_canon'}
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
								<div title="Diatonic interval from the subject. Strict Canon keeps this transformation available at group level.">
									<Knob
										value={voice.transpose_degrees}
										min={-7}
										max={7}
										step={1}
										defaultValue={0}
										label="Interval"
										accent="var(--color-accent-gold, #ffdd44)"
										size={56}
										format={(v) => transposeLabel(Math.round(v))}
										onchange={(v) => engine.updateCanonVoice(i, { transpose_degrees: Math.round(v) })}
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
									disabled={engine.imitativeForm === 'strict_canon'}
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
								title={`${engine.imitativeForm === 'strict_canon' ? 'Locked by Strict Canon. Switch Form to Free Imitation to choose a per-voice mode.' : "Mode — harmony engine routed for this voice's stack."}\n\n${voice.harmony_mode ? `Currently: ${voice.harmony_mode}\n${modeHelp(voice.harmony_mode)}` : `Currently: inheriting the global engine mode (${engine.mode}).\n${modeHelp(engine.mode)}`}\n\nCounterpoint expands into the four Fux species — pick "Counterpoint · Species N" to set both at once. Counterpoint strictness is always Strict (no Relaxed mode).`}
							>
								<span class="param-label font-ui">Mode</span>
								<PixelSelect
									options={MODE_OPTIONS}
									value={encodeMode(voice.harmony_mode, voice.counterpoint_species)}
									placeholder="Inherit"
									small={true}
									disabled={engine.imitativeForm === 'strict_canon'}
									onchange={(v) => {
										const decoded = decodeMode(v);
										engine.updateCanonVoice(i, {
											harmony_mode: decoded.mode,
											counterpoint_species: decoded.species,
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
									disabled={engine.imitativeForm === 'strict_canon'}
									onchange={(v) =>
										engine.updateCanonVoice(i, {
											reference_voice: v === '' ? null : parseInt(v, 10)
										})}
								/>
							</div>

							<div
								class="voice-param voice-param-mode"
								title={`${engine.imitativeForm === 'strict_canon' ? 'Locked per voice by Strict Canon. Use the Canon group Hold control above.' : "Hold — what happens to this voice's pending notes when you release the seeding input."}\n\nInherit = use the lane's setting (which itself may inherit the Companion global).\nCancel = drop all pending the moment you release.\nNear = let pending within 1 beat fire.\nPhrase = let pending within the current bar fire.\nForever = no cancellation (this voice always completes scheduled notes).`}
							>
								<span class="param-label font-ui">Hold</span>
								<PixelSelect
									options={[
										{ value: 'inherit', label: 'Inherit' },
										{ value: 'cancel', label: 'Cancel' },
										{ value: 'near_future', label: 'Near' },
										{ value: 'phrase_end', label: 'Phrase' },
										{ value: 'forever', label: 'Forever' }
									]}
									value={voice.hold_mode == null ? 'inherit' : voice.hold_mode.kind}
									placeholder="Inherit"
									small={true}
									disabled={engine.imitativeForm === 'strict_canon'}
									onchange={(v) => {
										const mode =
											v === 'inherit'
												? null
												: v === 'near_future'
													? { kind: 'near_future' as const, tail_beats: 1.0 }
													: { kind: v as 'cancel' | 'phrase_end' | 'forever' };
										engine.updateCanonVoice(i, { hold_mode: mode });
									}}
								/>
							</div>
						</div>
					{/each}
				</div>
			</div>

		</section>
		</div>
		</div>
		{:else}

		<!-- Species Counterpoint uses the same focused group editor. -->
		<div
			class="lane-card counterpoint-lane-card"
			class:lane-active={arrangement.counterpoint.enabled}
			title="Counterpoint Lane — a dedicated species-counterpoint voice running alongside the canon. Subdivides time per Fux species: Species 2 = 2 notes per cantus, Species 3 = 4 notes, Species 4 = syncopated entry. Picks pitches via the same CounterpointState rules (no parallel 5ths/8ves, stepwise preferred)."
		>
			<div class="lane-header">
				<span class="lane-title font-ui">COUNTERPOINT GROUP 2</span>
				<span class="lane-subtitle font-code">Species counterpoint · Fux rules</span>
				<span class="lane-actions">
					<label class="lane-hold-pick" title="Override Companion's global HoldMode for this lane only. 'Inherit' uses the global setting.">
						<span class="font-code">HOLD</span>
						<select
							value={engine.counterpointLaneHoldMode === null
								? 'inherit'
								: engine.counterpointLaneHoldMode.kind}
							onchange={(e) => {
								const v = (e.target as HTMLSelectElement).value;
								if (v === 'inherit') {
									engine.setCounterpointLaneHoldMode(null);
								} else if (v === 'near_future') {
									engine.setCounterpointLaneHoldMode({ kind: 'near_future', tail_beats: 1.0 });
								} else {
									engine.setCounterpointLaneHoldMode({ kind: v as 'cancel' | 'phrase_end' | 'forever' });
								}
							}}
						>
							<option value="inherit">Inherit</option>
							<option value="cancel">Cancel</option>
							<option value="near_future">Near</option>
							<option value="phrase_end">Phrase</option>
							<option value="forever">Forever</option>
						</select>
					</label>
					<button
						class="pixel-btn"
						class:toggle-on={arrangement.counterpoint.enabled}
						onclick={async () => {
							const enabled = !arrangement.counterpoint.enabled;
							if (enabled && !engine.companionEnabled) await engine.setCompanionEnabled(true);
							await arrangement.setCounterpoint({ enabled });
						}}
						title={arrangement.counterpoint.enabled
							? 'Counterpoint Lane is ON — emits a subdivided counterpoint line.'
							: 'Counterpoint Lane is OFF. Click to enable.'}
					>
						{arrangement.counterpoint.enabled ? 'ON' : 'OFF'}
					</button>
				</span>
			</div>

			<div class="lane-body cp-lane-body">
				<div class="species-map" aria-label="Species counterpoint relationship">
					<div class="subject-node">
						<div class="map-node-head font-ui"><strong>YOU PLAY</strong><span>CANTUS</span></div>
						<div class="map-node-main font-code">Live melody</div>
					</div>
					<div class="species-connector font-ui" aria-hidden="true">AGAINST ›</div>
					<div class="species-node">
						<div class="map-node-head font-ui"><strong>SPECIES LINE</strong><span>LIME · CH 7</span></div>
						<div class="map-node-main font-code">{arrangement.counterpoint.species} · {arrangement.counterpoint.preferAbove ? 'above' : 'below'}</div>
						<div class="map-node-meta font-code">{arrangement.counterpoint.transposeDegrees > 0 ? '+' : ''}{arrangement.counterpoint.transposeDegrees}° · independent movement</div>
					</div>
				</div>
				<div class="cp-control-grid">
				<label class="cp-row">
					<span class="param-label font-ui">Species</span>
					<PixelSelect
						options={[
							{ value: 'Species1', label: 'Species 1 (1:1 note-against-note)' },
							{ value: 'Species2', label: 'Species 2 (2:1 strong + passing)' },
							{ value: 'Species3', label: 'Species 3 (4:1 four notes per cantus)' },
							{ value: 'Species4', label: 'Species 4 (syncopated, suspensions)' }
						]}
						value={arrangement.counterpoint.species}
						small={true}
						onchange={(species) => arrangement.setCounterpoint({ species: species as typeof arrangement.counterpoint.species })}
					/>
				</label>

				<label class="cp-row">
					<span class="param-label font-ui">Direction</span>
					<PixelSelect
						options={[
							{ value: 'above', label: 'Above the player' },
							{ value: 'below', label: 'Below the player' }
						]}
						value={arrangement.counterpoint.preferAbove ? 'above' : 'below'}
						small={true}
						onchange={(value) => arrangement.setCounterpoint({ preferAbove: value === 'above' })}
					/>
				</label>

				<div class="cp-row cp-knob-row">
					<span class="param-label font-ui">Interval</span>
					<Knob
						value={arrangement.counterpoint.transposeDegrees}
						min={-7}
						max={7}
						step={1}
						defaultValue={2}
						label="Interval"
						help="Sets the starting diatonic distance between your melody and the independent species Counterpoint line."
						accent="var(--color-accent-magenta, #ff33aa)"
						size={48}
						format={(v) => (v === 0 ? 'unison' : `${v > 0 ? '+' : ''}${v}°`)}
						onchange={(value) => arrangement.setCounterpoint({ transposeDegrees: Math.round(value) })}
					/>
				</div>
				</div>
			</div>
		</div>
		{/if}

		<!-- FOOTER HINT -->
		<div class="footer-hint font-code">
			Groups run independently. Group 1 preserves the flexible Free Imitation controls; Group 2 follows the selected species rules.
		</div>
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
		min-width: 0;
	}

	.header-controls {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		min-width: 0;
	}

	.compact-pick {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		white-space: nowrap;
	}

	.compact-pick select,
	.compact-pick input {
		min-height: 26px;
		padding: 3px 6px;
		border: 1px solid var(--color-border);
		border-radius: 0;
		background: var(--color-bg);
		color: var(--color-text-primary);
		font: var(--font-size-xs) var(--font-code, monospace);
	}

	.tail-pick input { width: 4.5em; }

	.title {
		margin: 0;
		font-size: var(--font-size-lg);
		color: var(--color-accent-magenta, #ff33aa);
	}

	.subtitle {
		font-size: var(--font-size-xs);
		opacity: 0.6;
	}

	.group-status {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-xs);
		letter-spacing: 0.8px;
	}

	.body-stack {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 8px;
		flex: 1;
		min-height: 0;
		overflow-y: auto;
	}

	.lane-card {
		display: flex;
		flex-direction: column;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
	}

	.lane-card.lane-active {
		border-color: var(--color-accent-magenta, #ff33aa);
		box-shadow: 0 0 8px rgba(255, 51, 170, 0.12) inset;
	}

	.lane-header {
		display: grid;
		grid-template-columns: auto 1fr auto;
		align-items: center;
		gap: 12px;
		padding: 6px 10px;
		border-bottom: 1px solid var(--color-border);
		background: rgba(255, 255, 255, 0.02);
	}

	.lane-title {
		font-size: var(--font-size-md);
		color: var(--color-accent-cyan, #33ddff);
		letter-spacing: 0.1em;
	}

	.lane-card.lane-active .lane-title {
		color: var(--color-accent-magenta, #ff33aa);
	}

	.lane-subtitle {
		font-size: var(--font-size-xs);
		opacity: 0.55;
	}

	.lane-actions {
		display: flex;
		gap: 6px;
		justify-self: end;
	}

	.lane-body {
		flex: 1;
		min-height: 0;
	}

	.canon-lane-body {
		display: flex;
		flex-direction: column;
		min-height: 0;
	}

	.strict-contract {
		grid-column: 1 / -1;
		padding: 6px 9px;
		border-bottom: 1px solid rgba(255, 221, 68, 0.55);
		background: rgba(49, 40, 18, 0.82);
		color: var(--color-accent-gold, #ffdd44);
		font-size: 9px;
	}

	.subject-map {
		display: grid;
		grid-template-columns: minmax(150px, 0.55fr) auto minmax(280px, 1.45fr);
		align-items: stretch;
		gap: 7px;
		padding: 8px;
		border-bottom: 1px solid var(--color-border);
		background: rgba(11, 10, 21, 0.75);
	}
	.subject-node {
		padding: 8px;
		border: 1px solid var(--color-border);
		border-left: 4px solid var(--color-piano-input, #4fe8c3);
		background: rgba(27, 24, 49, 0.95);
	}
	.map-node-head,
	.followers-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 6px;
	}
	.map-node-head strong { color: var(--color-text-primary); font-size: 9px; }
	.map-node-head span,
	.followers-head { color: var(--color-text-dim); font-size: 7px; letter-spacing: 0.8px; }
	.map-node-main { margin-top: 7px; color: var(--color-piano-input, #4fe8c3); font-size: 10px; }
	.map-node-meta { margin-top: 4px; color: var(--color-text-secondary); font-size: 8px; }
	.subject-arrow { align-self: center; color: var(--color-accent-gold); font-size: 8px; writing-mode: vertical-rl; }
	.followers-map { min-width: 0; }
	.followers-head { margin-bottom: 5px; }
	.followers-head span:last-child { color: var(--color-accent-gold); }
	.follower-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 5px; }
	.follower-node {
		display: grid;
		grid-template-columns: auto 1fr;
		align-items: center;
		gap: 3px 8px;
		min-width: 0;
		padding: 6px 7px;
		border: 1px solid var(--color-border);
		border-left: 4px solid var(--color-accent-gold);
		border-radius: 0;
		background: rgba(27, 24, 49, 0.95);
		color: inherit;
		text-align: left;
		cursor: pointer;
	}
	.follower-node:hover,
	.follower-node.selected { border-color: var(--color-accent-gold); background: rgba(49, 40, 18, 0.72); }
	.follower-name { color: var(--color-accent-gold); font-size: 8px; }
	.follower-node strong { justify-self: end; color: var(--color-text-primary); font-size: 8px; white-space: nowrap; }
	.follower-node small { grid-column: 1 / -1; color: var(--color-text-dim); font-size: 7px; }

	.preset-toolbar {
		display: flex;
		align-items: center;
		gap: 7px;
		padding: 7px 8px;
		border-bottom: 1px solid var(--color-border);
		background: rgba(15, 14, 26, 0.5);
	}

	.preset-pick { flex: 0 1 310px; }
	.preset-pick select { width: 100%; }
	.preset-description {
		min-width: 0;
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		opacity: 0.7;
	}
	.compact-save { margin: 0 8px 7px; }

	.canon-right {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 8px;
		overflow-y: auto;
	}

	.cp-lane-body { padding: 8px; }
	.species-map {
		display: grid;
		grid-template-columns: minmax(150px, 0.8fr) auto minmax(220px, 1.2fr);
		align-items: stretch;
		gap: 7px;
		margin-bottom: 8px;
	}
	.species-node {
		padding: 8px;
		border: 1px solid var(--color-border);
		border-left: 4px solid #a3e635;
		background: rgba(27, 24, 49, 0.95);
	}
	.species-node .map-node-main { color: #a3e635; }
	.species-connector { align-self: center; color: #a3e635; font-size: 8px; }
	.cp-control-grid {
		display: grid;
		grid-template-columns: minmax(180px, 1fr) minmax(180px, 1fr) auto;
		align-items: center;
		gap: 8px;
		padding: 7px;
		border: 1px solid var(--color-border);
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

	.engine-status {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 8px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
	}

	.lane-hold-pick {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 0.7em;
		color: var(--color-text-secondary);
		letter-spacing: 0.06em;
		margin-right: 8px;
	}
	.lane-hold-pick select {
		font-size: 0.85em;
		padding: 2px 4px;
		background: var(--color-bg);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border);
		font-family: var(--font-code, monospace);
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
	.timeline-card > summary { cursor: pointer; margin: 0; }
	.timeline-card[open] > summary { margin-bottom: 6px; }
	.timeline-card > summary::marker { color: var(--color-accent-cyan); }

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

	/* Spans the rail content area exactly:
	   left   = timeline padding-left (8px) + label column (60px) + track gap (8px) = 76px
	   right  = timeline padding-right (8px)
	   so grid-line children at `left: X%` align with every track-rail's
	   X% under it (rails share the same horizontal extent). */
	.grid-overlay {
		position: absolute;
		left: 76px;
		right: 8px;
		top: 0;
		bottom: 0;
		pointer-events: none;
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

	.voice-editor-header {
		justify-content: flex-start;
		gap: 8px;
	}
	.voice-editor-header > span:first-child { margin-right: auto; }
	.voice-pick select { min-width: 120px; }

	.voice-grid {
		display: block;
	}
	.voice-card[hidden] { display: none; }

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
		grid-template-columns: repeat(3, 1fr);
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

	.cp-row {
		display: grid;
		grid-template-columns: 5em 1fr;
		align-items: center;
		gap: 6px;
	}

	.cp-knob-row {
		grid-template-columns: 4em auto;
		justify-content: end;
	}

	.toggle-on {
		background: rgba(255, 51, 170, 0.25);
		border-color: var(--color-accent-magenta, #ff33aa);
	}

	.footer-hint {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		opacity: 0.5;
		line-height: 1.4;
		padding: 4px 8px;
	}

	@media (max-width: 900px) {
		.header { align-items: flex-start; gap: 8px; }
		.header-controls { flex-wrap: wrap; }
		.subtitle, .preset-description { display: none; }
		.preset-pick { flex: 1; }
		.subject-map, .species-map, .cp-control-grid { grid-template-columns: 1fr; }
		.subject-arrow, .species-connector { display: none; }
		.lane-header { grid-template-columns: 1fr auto; }
		.lane-subtitle { display: none; }
	}
</style>
