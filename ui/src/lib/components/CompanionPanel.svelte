<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';

	// --- Quick-start templates: one-click canon configurations.
	// Each preset gives the user a working canon shape they can then
	// tweak per voice, instead of dialing in everything from scratch.
	const TEMPLATES: Array<{
		name: string;
		desc: string;
		voices: Array<{ delay_beats: number; transpose_degrees: number; time_ratio: number }>;
	}> = [
		{
			name: 'Single Echo',
			desc: '1-voice unison canon at +1 beat',
			voices: [{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 1.0 }]
		},
		{
			name: '3-Voice Round',
			desc: 'Frère Jacques style — 3 voices, unison',
			voices: [
				{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 2.0, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 3.0, transpose_degrees: 0, time_ratio: 1.0 }
			]
		},
		{
			name: 'Canon at 5th',
			desc: '2 voices: +1 beat unison, +2 beat fifth above',
			voices: [
				{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 2.0, transpose_degrees: 4, time_ratio: 1.0 }
			]
		},
		{
			name: 'Triad Stack',
			desc: '3 simultaneous voices at unison/third/fifth',
			voices: [
				{ delay_beats: 0.5, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 0.5, transpose_degrees: 2, time_ratio: 1.0 },
				{ delay_beats: 0.5, transpose_degrees: 4, time_ratio: 1.0 }
			]
		},
		{
			name: 'Augmentation Canon',
			desc: '2 voices: unison + 2× augmentation (half speed)',
			voices: [
				{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 1.0, transpose_degrees: -7, time_ratio: 2.0 }
			]
		},
		{
			name: 'Diminution Canon',
			desc: '2 voices: unison + 0.5× diminution (double speed)',
			voices: [
				{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 0.5 }
			]
		},
		{
			name: 'Bach 3-Voice',
			desc: '3 voices at unison/fifth/octave staggered by 1 beat',
			voices: [
				{ delay_beats: 1.0, transpose_degrees: 0, time_ratio: 1.0 },
				{ delay_beats: 2.0, transpose_degrees: 4, time_ratio: 1.0 },
				{ delay_beats: 3.0, transpose_degrees: 7, time_ratio: 1.0 }
			]
		}
	];

	async function applyTemplate(t: (typeof TEMPLATES)[number]) {
		// Apply template = replace voices + enable both gates so the
		// user hears it immediately. Template names imply user intent
		// to USE the preset, not just stage it.
		await engine.setCanonVoices(t.voices);
		if (!engine.companionEnabled) await engine.setCompanionEnabled(true);
		if (!engine.canonEnabled) await engine.setCanonEnabled(true);
	}

	// --- Time-ratio readout helpers — keep card labels self-explanatory.
	function ratioLabel(r: number): string {
		if (Math.abs(r - 1.0) < 0.01) return 'strict';
		if (r < 1.0) return `${(1 / r).toFixed(2)}× dim`;
		return `${r.toFixed(2)}× aug`;
	}

	function transposeLabel(d: number): string {
		if (d === 0) return 'unison';
		const sign = d > 0 ? '+' : '';
		return `${sign}${d}°`;
	}
</script>

<div class="companion-root">
	<!-- MASTER GATES -->
	<section class="card master">
		<div class="master-row">
			<div class="master-toggle">
				<span class="cell-label font-ui">Companion</span>
				<button
					class="pixel-btn toggle-btn"
					class:toggle-on={engine.companionEnabled}
					onclick={() => engine.setCompanionEnabled(!engine.companionEnabled)}
				>
					{engine.companionEnabled ? 'ON' : 'OFF'}
				</button>
			</div>
			<div class="master-toggle">
				<span class="cell-label font-ui">Canon</span>
				<button
					class="pixel-btn toggle-btn"
					class:toggle-on={engine.canonEnabled}
					disabled={!engine.companionEnabled}
					onclick={() => engine.setCanonEnabled(!engine.canonEnabled)}
				>
					{engine.canonEnabled ? 'ON' : 'OFF'}
				</button>
			</div>
		</div>
		{#if !engine.companionEnabled}
			<div class="hint font-ui">Enable Companion to use Canon and other delayed-voice lanes.</div>
		{:else if !engine.canonEnabled}
			<div class="hint font-ui">Enable Canon to fire delayed voices. Transport must be playing.</div>
		{:else}
			<div class="hint font-ui">
				Canon active. {engine.canonVoices.length} voice{engine.canonVoices.length === 1 ? '' : 's'} configured.
			</div>
		{/if}
	</section>

	<!-- TEMPLATES -->
	<section class="card">
		<header class="section-header font-ui">TEMPLATES</header>
		<div class="template-grid">
			{#each TEMPLATES as t (t.name)}
				<button
					class="template-btn pixel-btn"
					onclick={() => applyTemplate(t)}
					title={t.desc}
				>
					<span class="template-name font-ui">{t.name}</span>
					<span class="template-desc font-code">{t.desc}</span>
				</button>
			{/each}
		</div>
	</section>

	<!-- VOICES -->
	<section class="card">
		<header class="section-header font-ui">
			VOICES ({engine.canonVoices.length})
			<button
				class="pixel-btn"
				disabled={engine.canonVoices.length >= 8}
				onclick={() => engine.addCanonVoice()}
				title="Add a canon voice (up to 8)"
			>
				+ Add Voice
			</button>
		</header>
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
						<span class="param-label font-ui">Transp</span>
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
						<span class="param-label font-ui">Time</span>
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
				</div>
			{/each}
		</div>
	</section>

	<!-- FOOTER -->
	<section class="footer-hint font-ui">
		Canon voices fire relative to a phrase anchor. Phrase resets after 2 beats of silence.
		Transpose routes through the engine's modal-interchange path when enabled — out-of-scale
		input borrows from a parallel mode rather than emitting bare unison.
	</section>
</div>

<style>
	.companion-root {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 8px;
		overflow-y: auto;
	}

	.card {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		padding: 8px;
	}

	.master-row {
		display: flex;
		gap: 16px;
		align-items: center;
	}

	.master-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
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

	.template-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 6px;
	}

	.template-btn {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 2px;
		padding: 6px 8px;
		text-align: left;
		white-space: normal;
	}

	.template-name {
		color: var(--color-accent-cyan, #33ddff);
		font-size: var(--font-size-sm);
	}

	.template-desc {
		font-size: var(--font-size-xs);
		opacity: 0.7;
		line-height: 1.3;
	}

	.voice-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 8px;
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
		grid-template-columns: 3em 1fr 5em;
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

	.hint {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		opacity: 0.7;
		margin-top: 6px;
	}

	.footer-hint {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		opacity: 0.6;
		line-height: 1.4;
		padding: 4px 8px;
	}
</style>
