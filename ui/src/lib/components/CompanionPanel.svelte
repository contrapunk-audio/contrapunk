<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import PixelSelect from './PixelSelect.svelte';

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

	// --- Forms: each is one preset the user picks from the left rail.
	// Two families today: REACTIVE (canon-style delayed voices) and
	// HARMONIC (synchronous accompaniment via engine config). Future
	// families (looper, drone, arpeggiator) ship as additional lanes.
	type Form = {
		id: string;
		name: string;
		desc: string;
		family: 'reactive' | 'harmonic';
		voices?: Array<{ delay_beats: number; transpose_degrees: number; time_ratio: number }>;
		harmonic?: {
			mode: string;
			scale_mode?: string;
			voice_count: number;
			voice_leading_enabled?: boolean;
			voice_leading_style?: string;
		};
	};

	const TEMPLATES: Form[] = [
		{
			id: 'single-echo',
			name: 'Single Echo',
			desc: '1 voice, +1 beat unison',
			family: 'reactive',
			voices: [{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 1.0 }]
		},
		{
			id: 'round-3',
			name: '3-Voice Round',
			desc: 'Frère Jacques — 3 voices, unison, staggered',
			family: 'reactive',
			voices: [
				{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 2.0, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 3.0, transpose_degrees: 0, time_ratio: 1.0 }
			]
		},
		{
			id: 'canon-5th',
			name: 'Canon at 5th',
			desc: '2 voices, unison + a fifth above',
			family: 'reactive',
			voices: [
				{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 2.0, transpose_degrees: 4, time_ratio: 1.0 }
			]
		},
		{
			id: 'triad-stack',
			name: 'Triad Stack',
			desc: '3 voices simultaneously at unison/third/fifth',
			family: 'reactive',
			voices: [
				{ delay_beats: 0.5, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 0.5, transpose_degrees: 2, time_ratio: 1.0 },
				{ delay_beats: 0.5, transpose_degrees: 4, time_ratio: 1.0 }
			]
		},
		{
			id: 'augment',
			name: 'Augmentation Canon',
			desc: '2 voices: unison + a 2× slower copy',
			family: 'reactive',
			voices: [
				{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 1.0, transpose_degrees: -7, time_ratio: 2.0 }
			]
		},
		{
			id: 'dimin',
			name: 'Diminution Canon',
			desc: '2 voices: unison + a 2× faster copy',
			family: 'reactive',
			voices: [
				{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 0.5 }
			]
		},
		{
			id: 'bach-3',
			name: 'Bach 3-Voice',
			desc: '3 voices at unison/fifth/octave, +1/+2/+3 beats',
			family: 'reactive',
			voices: [
				{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 2.0, transpose_degrees: 4, time_ratio: 1.0 },
				{ delay_beats: 3.0, transpose_degrees: 7, time_ratio: 1.0 }
			]
		}
	];

	let selectedTemplate = $state<string | null>(null);

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
	// Show a 4-beat window. Each voice gets a horizontal track. The
	// player melody is track 0 (always at beat 0). Each canon voice
	// shows its entry as a dot at its delay offset. Diminution /
	// augmentation are hinted by a trailing arrow on the dot.
	const TIMELINE_BEATS = 4;

	function entryX(beats: number): number {
		// Map 0..TIMELINE_BEATS into 0..100% of timeline width.
		const clamped = Math.max(0, Math.min(TIMELINE_BEATS, beats));
		return (clamped / TIMELINE_BEATS) * 100;
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
			<div class="section-header font-ui">FORMS</div>
			{#each TEMPLATES as t (t.id)}
				<button
					class="template-row"
					class:active={selectedTemplate === t.id}
					onclick={() => applyTemplate(t)}
					title={t.desc}
				>
					<div class="template-name font-ui">{t.name}</div>
					<div class="template-desc font-code">{t.desc}</div>
				</button>
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
					<span class="hint font-code">player → delayed voices (4-beat window)</span>
				</div>
				<div class="timeline">
					<!-- Beat grid lines -->
					{#each Array(TIMELINE_BEATS + 1) as _, i (i)}
						<div class="grid-line" style="left: {(i / TIMELINE_BEATS) * 100}%">
							<span class="grid-label font-code">{i}</span>
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

							<div class="voice-param">
								<span class="param-label font-ui">Delay</span>
								<input
									type="range"
									min="0.25"
									max="4"
									step="0.25"
									value={voice.delay_beats}
									oninput={(e) =>
										engine.updateCanonVoice(i, {
											delay_beats: parseFloat((e.target as HTMLInputElement).value)
										})}
									class="pixel-range"
								/>
								<span class="param-readout font-code">
									{voice.delay_beats.toFixed(2)} beat{voice.delay_beats === 1 ? '' : 's'}
								</span>
							</div>

							<div class="voice-param">
								<span class="param-label font-ui">Interval</span>
								<input
									type="range"
									min="-7"
									max="7"
									step="1"
									value={voice.transpose_degrees}
									oninput={(e) =>
										engine.updateCanonVoice(i, {
											transpose_degrees: parseInt((e.target as HTMLInputElement).value, 10)
										})}
									class="pixel-range"
								/>
								<span class="param-readout font-code">{transposeLabel(voice.transpose_degrees)}</span>
							</div>

							<div class="voice-param">
								<span class="param-label font-ui">Speed</span>
								<input
									type="range"
									min="0.25"
									max="4"
									step="0.25"
									value={voice.time_ratio}
									oninput={(e) =>
										engine.updateCanonVoice(i, {
											time_ratio: parseFloat((e.target as HTMLInputElement).value)
										})}
									class="pixel-range"
								/>
								<span class="param-readout font-code">{ratioLabel(voice.time_ratio)}</span>
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
	}

	.template-row:hover {
		border-color: var(--color-accent-cyan, #33ddff);
	}

	.template-row.active {
		border-color: var(--color-accent-magenta, #ff33aa);
		background: rgba(255, 51, 170, 0.1);
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

	.param-readout {
		font-size: var(--font-size-xs);
		text-align: right;
		opacity: 0.8;
	}

	.pixel-range {
		width: 100%;
		height: 4px;
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
