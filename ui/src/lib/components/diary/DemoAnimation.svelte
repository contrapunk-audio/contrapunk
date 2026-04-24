<script lang="ts">
	import { audioStore } from '$lib/components/diary/audioContext.svelte';
	import WaveformDisplay from '$lib/components/diary/WaveformDisplay.svelte';
	import SpectrogramViewer from '$lib/components/diary/SpectrogramViewer.svelte';

	let { autoplay = false }: { autoplay?: boolean } = $props();

	type StringKey = 'E2' | 'A2' | 'D3' | 'G3' | 'B3' | 'E4';

	const strings: { key: StringKey; label: string; confidence: string }[] = [
		{ key: 'E2', label: 'E2 open', confidence: '97.2%' },
		{ key: 'A2', label: 'A2 open', confidence: '96.8%' },
		{ key: 'D3', label: 'D3 open', confidence: '98.1%' },
		{ key: 'G3', label: 'G3 open', confidence: '95.4%' },
		{ key: 'B3', label: 'B3 open', confidence: '97.6%' },
		{ key: 'E4', label: 'E4 open', confidence: '98.5%' },
	];

	let selected: StringKey = $state('E2');
	let stage = $state(0); // 0=idle, 1=playing+waveform, 2=spectrogram, 3=arrow, 4=result
	let spectrogramData: number[][] | null = $state(null);
	let timers: ReturnType<typeof setTimeout>[] = $state([]);

	let audioUrl = $derived(`/samples/showcase/${selected}_open.wav`);
	let specUrl = $derived(`/spectrograms/showcase/${selected}_open.json`);
	let info = $derived(strings.find((s) => s.key === selected)!);

	function clearTimers() {
		for (const t of timers) clearTimeout(t);
		timers = [];
	}

	async function runDemo() {
		clearTimers();
		audioStore.stop();
		stage = 0;

		// Load spectrogram data
		try {
			const res = await fetch(specUrl);
			const json = await res.json();
			spectrogramData = json.spectrogram.data;
		} catch {
			spectrogramData = null;
		}

		// Stage 1: play audio + waveform draws
		stage = 1;
		audioStore.play(audioUrl);

		// Stage 2: spectrogram visible after 500ms
		const t1 = setTimeout(() => { stage = 2; }, 500);

		// Stage 3: arrow after 1500ms
		const t2 = setTimeout(() => { stage = 3; }, 1500);

		// Stage 4: result after 2000ms
		const t3 = setTimeout(() => { stage = 4; }, 2000);

		timers = [t1, t2, t3];
	}

	function selectString(key: StringKey) {
		selected = key;
		stage = 0;
		clearTimers();
		audioStore.stop();
		spectrogramData = null;
	}

	// Cleanup on destroy
	$effect(() => {
		return () => {
			clearTimers();
			audioStore.stop();
		};
	});
</script>

<div class="demo-wrap">
	<div class="demo-pipeline" class:active={stage >= 1}>
		<!-- Waveform -->
		<div class="demo-stage waveform-stage" class:visible={stage >= 1}>
			<div class="stage-label">WAVEFORM</div>
			<WaveformDisplay url={audioUrl} height={56} />
		</div>

		<!-- Arrow 1 -->
		<div class="demo-arrow" class:visible={stage >= 2}>
			<svg width="32" height="20" viewBox="0 0 32 20" aria-hidden="true">
				<line x1="0" y1="10" x2="24" y2="10" stroke="var(--color-accent-cyan)" stroke-width="2" />
				<polygon points="22,5 32,10 22,15" fill="var(--color-accent-cyan)" />
			</svg>
		</div>

		<!-- Spectrogram -->
		<div class="demo-stage spectrogram-stage" class:visible={stage >= 2}>
			<div class="stage-label">SPECTROGRAM</div>
			{#if spectrogramData}
				<SpectrogramViewer data={spectrogramData} height={56} audioUrl={audioUrl} />
			{:else}
				<div class="stage-placeholder" style="height: 56px"></div>
			{/if}
		</div>

		<!-- Arrow 2 -->
		<div class="demo-arrow" class:visible={stage >= 3}>
			<svg width="32" height="20" viewBox="0 0 32 20" aria-hidden="true">
				<line x1="0" y1="10" x2="24" y2="10" stroke="var(--color-accent-cyan)" stroke-width="2" />
				<polygon points="22,5 32,10 22,15" fill="var(--color-accent-cyan)" />
			</svg>
		</div>

		<!-- Result -->
		<div class="demo-stage result-stage" class:visible={stage >= 4}>
			<div class="stage-label">RESULT</div>
			<div class="result-card">
				<div class="result-note">{info.label}</div>
				<div class="result-confidence">{info.confidence}</div>
				<div class="confidence-bar">
					<div
						class="confidence-fill"
						style="width: {stage >= 4 ? info.confidence : '0%'}"
					></div>
				</div>
			</div>
		</div>
	</div>

	<!-- Controls -->
	<div class="demo-controls">
		{#if stage === 0}
			<button class="pixel-btn play-demo-btn" onclick={runDemo}>
				PLAY DEMO
			</button>
		{:else}
			<button class="pixel-btn replay-btn" onclick={runDemo}>
				REPLAY
			</button>
		{/if}
	</div>

	<!-- String selector (shown after first play) -->
	{#if stage >= 4}
		<div class="string-selector">
			<div class="selector-label">TRY ANOTHER STRING</div>
			<div class="string-buttons">
				{#each strings as s}
					<button
						class="pixel-btn string-btn"
						class:active={selected === s.key}
						onclick={() => { selectString(s.key); setTimeout(runDemo, 100); }}
					>
						{s.key}
					</button>
				{/each}
			</div>
		</div>
	{/if}
</div>

<style>
	.demo-wrap {
		padding: 24px 0;
	}

	.demo-pipeline {
		display: flex;
		align-items: center;
		gap: 0;
		opacity: 0.3;
		transition: opacity 0.3s ease;
	}
	.demo-pipeline.active {
		opacity: 1;
	}

	.demo-stage {
		flex: 1;
		min-width: 0;
		opacity: 0;
		transform: translateY(6px);
		transition: opacity 0.4s ease, transform 0.4s ease;
	}
	.demo-stage.visible {
		opacity: 1;
		transform: translateY(0);
	}

	.stage-label {
		font-family: var(--font-ui);
		font-size: 8px;
		color: var(--color-text-dim);
		letter-spacing: 2px;
		margin-bottom: 6px;
	}

	.stage-placeholder {
		background: var(--color-bg-deep);
		border: 1px solid var(--color-border);
	}

	.demo-arrow {
		flex-shrink: 0;
		padding: 0 4px;
		opacity: 0;
		transform: scaleX(0);
		transform-origin: left center;
		transition: opacity 0.3s ease 0.1s, transform 0.3s ease 0.1s;
		align-self: flex-end;
		margin-bottom: 10px;
	}
	.demo-arrow.visible {
		opacity: 1;
		transform: scaleX(1);
	}

	.result-card {
		background: var(--color-bg-deep);
		border: 1px solid var(--color-border);
		padding: 8px 12px;
		min-height: 56px;
		display: flex;
		flex-direction: column;
		justify-content: center;
	}

	.result-note {
		font-family: var(--font-code);
		font-size: 16px;
		color: var(--color-accent-cyan);
		letter-spacing: 1px;
	}

	.result-confidence {
		font-family: var(--font-reading);
		font-size: 12px;
		color: var(--color-text-secondary);
		margin-top: 2px;
	}

	.confidence-bar {
		margin-top: 6px;
		height: 4px;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		overflow: hidden;
	}

	.confidence-fill {
		height: 100%;
		background: var(--color-accent-cyan);
		transition: width 0.6s ease;
	}

	.demo-controls {
		display: flex;
		justify-content: center;
		margin-top: 20px;
	}

	.play-demo-btn {
		border-color: var(--color-accent-cyan) !important;
		color: var(--color-accent-cyan);
		font-size: 11px !important;
		padding: 6px 20px !important;
	}
	.play-demo-btn:hover {
		background: rgba(51, 221, 255, 0.08) !important;
		box-shadow: 0 0 12px rgba(51, 221, 255, 0.15);
	}

	.replay-btn {
		border-color: var(--color-accent-teal) !important;
		color: var(--color-accent-teal);
		font-size: 10px !important;
		padding: 4px 14px !important;
	}

	.string-selector {
		margin-top: 20px;
		text-align: center;
	}

	.selector-label {
		font-family: var(--font-ui);
		font-size: 8px;
		color: var(--color-text-dim);
		letter-spacing: 2px;
		margin-bottom: 8px;
	}

	.string-buttons {
		display: flex;
		justify-content: center;
		gap: 6px;
	}

	.string-btn {
		font-size: 9px !important;
		padding: 3px 8px !important;
	}
	.string-btn.active {
		border-color: var(--color-accent-cyan) !important;
		color: var(--color-accent-cyan);
	}

	/* Mobile: vertical layout */
	@media (max-width: 600px) {
		.demo-pipeline {
			flex-direction: column;
			gap: 8px;
		}
		.demo-arrow {
			transform: rotate(90deg) scaleX(0);
			align-self: center;
			margin-bottom: 0;
			padding: 4px 0;
		}
		.demo-arrow.visible {
			transform: rotate(90deg) scaleX(1);
		}
	}
</style>
