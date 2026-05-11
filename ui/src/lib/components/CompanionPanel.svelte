<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { transport } from '$lib/stores/transport.svelte';
	import PixelSelect from './PixelSelect.svelte';
	import Knob from './Knob.svelte';

	// Per-voice harmony mode options (slice G dropdown). Empty value =
	// inherit the engine's global mode. Names match
	// parse_harmony_mode in src-tauri/src/commands/harmony.rs.
	const MODE_OPTIONS: Array<{ value: string; label: string }> = [
		{ value: '', label: 'Inherit global' },
		{ value: 'PassThrough', label: 'Pass-Through' },
		{ value: 'DiatonicThirds', label: 'Diatonic Thirds' },
		{ value: 'DiatonicFourths', label: 'Diatonic Fourths' },
		{ value: 'ContraryMotion', label: 'Contrary Motion' },
		{ value: 'StrictCounterpoint', label: 'Counterpoint' },
		{ value: 'FunctionalHarmony', label: 'Functional' },
		{ value: 'BachChorale', label: 'Bach Chorale' },
		{ value: 'BarryHarris', label: 'Barry Harris' }
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
	const BUILTIN_TEMPLATES: Form[] = [
		{
			id: 'bach-fugue',
			name: 'Bach Fugue',
			desc: '3 voices · subject + tonal answer + countersubject (BachChorale × Counterpoint)',
			family: 'reactive',
			voices: [
				{ delay_beats: 0, transpose_degrees: 0, time_ratio: 1.0, harmony_mode: 'BachChorale' },
				{ delay_beats: 4, transpose_degrees: 4, time_ratio: 1.0, harmony_mode: 'StrictCounterpoint' },
				{ delay_beats: 8, transpose_degrees: 0, time_ratio: 1.0, harmony_mode: 'BachChorale' }
			]
		},
		{
			id: 'modal-cascade',
			name: 'Modal Cascade',
			desc: '3 voices · Thirds → Fourths → Counterpoint, each mode stacks',
			family: 'reactive',
			voices: [
				{ delay_beats: 1, transpose_degrees: 2, time_ratio: 1.0, harmony_mode: 'DiatonicThirds' },
				{ delay_beats: 2, transpose_degrees: 3, time_ratio: 1.0, harmony_mode: 'DiatonicFourths' },
				{ delay_beats: 3, transpose_degrees: 4, time_ratio: 1.0, harmony_mode: 'StrictCounterpoint' }
			]
		},
		{
			id: 'contrary-mirror',
			name: 'Contrary Mirror',
			desc: '2 voices · ContraryMotion above + below — opposing line motion',
			family: 'reactive',
			voices: [
				{ delay_beats: 1, transpose_degrees: 4, time_ratio: 1.0, harmony_mode: 'ContraryMotion' },
				{ delay_beats: 2, transpose_degrees: -4, time_ratio: 1.0, harmony_mode: 'ContraryMotion' }
			]
		},
		{
			id: 'augmentation-canon',
			name: 'Augmentation Canon',
			desc: 'Subject + 2× slower answer · Bach mensuration trick',
			family: 'reactive',
			voices: [
				{ delay_beats: 0, transpose_degrees: 0, time_ratio: 1.0, harmony_mode: 'StrictCounterpoint' },
				{ delay_beats: 1, transpose_degrees: 4, time_ratio: 2.0, harmony_mode: 'BachChorale' }
			]
		},
		{
			id: 'diminution-fugue',
			name: 'Diminution Fugue',
			desc: 'Subject + 2× faster stretto · counter-running imitation',
			family: 'reactive',
			voices: [
				{ delay_beats: 0, transpose_degrees: 0, time_ratio: 1.0, harmony_mode: 'StrictCounterpoint' },
				{ delay_beats: 1, transpose_degrees: 4, time_ratio: 0.5, harmony_mode: 'StrictCounterpoint' }
			]
		},
		{
			id: 'functional-pillar',
			name: 'Functional Pillar',
			desc: '3 voices · I–IV–V driven harmony · root + third + fifth via FunctionalHarmony',
			family: 'reactive',
			voices: [
				{ delay_beats: 0.5, transpose_degrees: 0, time_ratio: 1.0, harmony_mode: 'FunctionalHarmony' },
				{ delay_beats: 0.5, transpose_degrees: 2, time_ratio: 1.0, harmony_mode: 'FunctionalHarmony' },
				{ delay_beats: 0.5, transpose_degrees: 4, time_ratio: 1.0, harmony_mode: 'FunctionalHarmony' }
			]
		},
		{
			id: 'barry-harris',
			name: 'Barry Harris Stack',
			desc: '3 voices · 6th-diminished system · jazz tertian harmony',
			family: 'reactive',
			voices: [
				{ delay_beats: 0.5, transpose_degrees: 0, time_ratio: 1.0, harmony_mode: 'BarryHarris' },
				{ delay_beats: 0.5, transpose_degrees: 2, time_ratio: 1.0, harmony_mode: 'BarryHarris' },
				{ delay_beats: 0.5, transpose_degrees: 5, time_ratio: 1.0, harmony_mode: 'BarryHarris' }
			]
		},
		{
			id: 'mensuration-trinity',
			name: 'Mensuration Trinity',
			desc: '3 voices · 1× / 2× / 0.5× speeds — Ockeghem-style proportional canon',
			family: 'reactive',
			voices: [
				{ delay_beats: 0, transpose_degrees: 0, time_ratio: 1.0, harmony_mode: 'StrictCounterpoint' },
				{ delay_beats: 2, transpose_degrees: 4, time_ratio: 2.0, harmony_mode: 'BachChorale' },
				{ delay_beats: 1, transpose_degrees: -7, time_ratio: 0.5, harmony_mode: 'ContraryMotion' }
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
			<div class="section-header font-ui">
				FORMS
				<button
					class="pixel-btn"
					onclick={() => {
						showSaveForm = !showSaveForm;
						newFormName = '';
					}}
					title="Save current voice configuration as a new form"
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
			<div class="engine-status">
				<span class="status-label font-ui">ENGINE</span>
				<span class="status-pill font-code">
					<span class="pill-key">{engine.key}</span> ·
					<span class="pill-mode">{engine.mode}</span> ·
					{engine.voiceCount} voice{engine.voiceCount === 1 ? '' : 's'}{#if engine.voiceLeadingEnabled} · VL {engine.voiceLeadingStyle}{/if}
				</span>
			</div>

			<!-- TIMELINE VISUALIZATION -->
			<div class="timeline-card">
				<div class="section-header font-ui">
					ENTRY TIMELINE
					<span class="hint font-code">
						player → delayed voices ({timelineBeats} beat{timelineBeats === 1 ? '' : 's'} · {transport.beatsPerBar}/{transport.beatUnit})
					</span>
				</div>
				<div class="timeline">
					<!-- Beat / bar grid lines. Bar lines are brighter and
					     labeled `1`, `2`, … (bar number). Beat ticks within
					     a bar get a small subdivision label. -->
					{#each gridTicks as t (t.beat)}
						<div
							class="grid-line"
							class:bar-line={t.isBar}
							style="left: {(t.beat / timelineBeats) * 100}%"
						>
							<span class="grid-label font-code">
								{t.isBar ? t.barIdx + 1 : ''}
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
						<div class="track voice-track">
							<span class="track-label font-ui">V{i + 1}</span>
							<div class="track-rail">
								<div
									class="entry-dot voice-dot"
									class:transpose-pos={voice.transpose_degrees > 0}
									class:transpose-neg={voice.transpose_degrees < 0}
									style="left: {entryX(voice.delay_beats)}%"
									title="Voice {i + 1}: enters at {voice.delay_beats.toFixed(2)} beats, {transposeLabel(voice.transpose_degrees)}, {ratioLabel(voice.time_ratio)}"
								>
									{voice.time_ratio < 1
										? '◀'
										: voice.time_ratio > 1
											? '▶'
											: '◉'}
								</div>
							</div>
						</div>
					{/each}
				</div>
			</div>

			<!-- VOICE CARDS -->
			<div class="voices-card">
				<div class="section-header font-ui">
					VOICES ({engine.canonVoices.length})
					<button
						class="pixel-btn"
						disabled={engine.canonVoices.length >= 8}
						onclick={() => engine.addCanonVoice()}
						title="Add a canon voice (up to 8)"
					>
						+ Add Voice
					</button>
				</div>
				<div class="voice-grid">
					{#each engine.canonVoices as voice, i (i)}
						<div class="voice-card">
							<div class="voice-header">
								<span class="voice-label font-ui">V{i + 1}</span>
								<button
									class="pixel-btn remove-btn"
									disabled={engine.canonVoices.length <= 1}
									onclick={() => engine.removeCanonVoice(i)}
									title="Remove voice"
								>
									×
								</button>
							</div>

							<div class="voice-knobs">
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

							<div class="voice-param voice-param-mode">
								<span class="param-label font-ui">Mode</span>
								<PixelSelect
									options={MODE_OPTIONS}
									value={voice.harmony_mode ?? ''}
									placeholder="Inherit"
									small={true}
									onchange={(v) =>
										engine.updateCanonVoice(i, {
											harmony_mode: v === '' ? null : v
										})}
								/>
							</div>

							<div class="voice-param voice-param-mode">
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
		opacity: 0.6;
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
