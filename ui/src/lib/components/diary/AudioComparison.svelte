<script lang="ts">
	import AudioPlayer from '$lib/components/diary/AudioPlayer.svelte';
	import WaveformDisplay from '$lib/components/diary/WaveformDisplay.svelte';
	import { audioStore } from '$lib/components/diary/audioContext.svelte';

	let {
		urlA,
		labelA,
		urlB,
		labelB,
		question = '',
	}: {
		urlA: string;
		labelA: string;
		urlB: string;
		labelB: string;
		question?: string;
	} = $props();

	let activeIs = $derived<'a' | 'b' | null>(
		audioStore.isPlaying(urlA) ? 'a' :
		audioStore.isPlaying(urlB) ? 'b' :
		null
	);

	function playA() {
		if (audioStore.isPlaying(urlA)) {
			audioStore.stop();
		} else {
			audioStore.play(urlA);
		}
	}

	function playB() {
		if (audioStore.isPlaying(urlB)) {
			audioStore.stop();
		} else {
			audioStore.play(urlB);
		}
	}
</script>

<div class="comparison">
	{#if question}
		<div class="question">{question}</div>
	{/if}

	<div class="ab-grid">
		<div class="ab-side" class:active={activeIs === 'a'}>
			<div class="side-label">A</div>
			<WaveformDisplay url={urlA} height={48} />
			<AudioPlayer url={urlA} label={labelA} />
		</div>

		<div class="ab-divider">
			<span class="vs-label">VS</span>
		</div>

		<div class="ab-side" class:active={activeIs === 'b'}>
			<div class="side-label">B</div>
			<WaveformDisplay url={urlB} height={48} />
			<AudioPlayer url={urlB} label={labelB} />
		</div>
	</div>
</div>

<style>
	.comparison {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 16px;
	}

	.question {
		font-family: var(--font-reading);
		font-size: 14px;
		color: var(--color-accent-cyan);
		text-align: center;
		margin-bottom: 16px;
		font-style: italic;
	}

	.ab-grid {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		gap: 12px;
		align-items: start;
	}

	.ab-side {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 8px;
		border: 1px solid transparent;
		transition: border-color 0.15s ease;
	}
	.ab-side.active {
		border-color: var(--color-accent-cyan-dim);
		background: rgba(51, 221, 255, 0.03);
	}

	.side-label {
		font-family: var(--font-ui);
		font-size: var(--font-size-sm);
		color: var(--color-text-dim);
		text-align: center;
		letter-spacing: 2px;
	}
	.ab-side.active .side-label {
		color: var(--color-accent-cyan);
	}

	.ab-divider {
		display: flex;
		align-items: center;
		justify-content: center;
		padding-top: 48px;
	}

	.vs-label {
		font-family: var(--font-ui);
		font-size: var(--font-size-xs);
		color: var(--color-text-dim);
		letter-spacing: 2px;
	}

	/* Stack on small screens */
	@media (max-width: 600px) {
		.ab-grid {
			grid-template-columns: 1fr;
			gap: 8px;
		}
		.ab-divider {
			padding-top: 0;
			padding: 4px 0;
		}
	}
</style>
