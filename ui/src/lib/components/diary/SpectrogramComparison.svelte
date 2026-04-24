<script lang="ts">
	import SpectrogramViewer from '$lib/components/diary/SpectrogramViewer.svelte';
	import AudioPlayer from '$lib/components/diary/AudioPlayer.svelte';

	let {
		urlA,
		urlB,
		labelA,
		labelB,
		audioUrlA = '',
		audioUrlB = '',
		question = '',
	}: {
		urlA: string;
		urlB: string;
		labelA: string;
		labelB: string;
		audioUrlA?: string;
		audioUrlB?: string;
		question?: string;
	} = $props();

	type SpectrogramJson = {
		label: string;
		string_name: string;
		midi_note: number;
		sample_rate: number;
		spectrogram: {
			n_mels: number;
			n_fft: number;
			hop_length: number;
			time_frames: number;
			data: number[][];
		};
		waveform: number[];
		rms_envelope: number[];
	};

	let dataA: number[][] | null = $state(null);
	let dataB: number[][] | null = $state(null);
	let loading = $state(true);
	let error = $state('');

	$effect(() => {
		loading = true;
		error = '';
		dataA = null;
		dataB = null;

		Promise.all([
			fetch(urlA).then(r => {
				if (!r.ok) throw new Error(`Failed to load ${urlA}`);
				return r.json() as Promise<SpectrogramJson>;
			}),
			fetch(urlB).then(r => {
				if (!r.ok) throw new Error(`Failed to load ${urlB}`);
				return r.json() as Promise<SpectrogramJson>;
			}),
		])
			.then(([a, b]) => {
				dataA = a.spectrogram.data;
				dataB = b.spectrogram.data;
				loading = false;
			})
			.catch((e) => {
				error = e instanceof Error ? e.message : 'Failed to load spectrograms';
				loading = false;
			});
	});
</script>

<div class="spec-comparison">
	{#if question}
		<div class="spec-question">{question}</div>
	{/if}

	{#if loading}
		<div class="spec-loading">Loading spectrograms...</div>
	{:else if error}
		<div class="spec-error">{error}</div>
	{:else if dataA && dataB}
		<div class="spec-grid">
			<div class="spec-side">
				<SpectrogramViewer data={dataA} label={labelA} audioUrl={audioUrlA} />
				{#if audioUrlA}
					<AudioPlayer url={audioUrlA} label={labelA} />
				{/if}
			</div>

			<div class="spec-divider">
				<span class="spec-vs">VS</span>
			</div>

			<div class="spec-side">
				<SpectrogramViewer data={dataB} label={labelB} audioUrl={audioUrlB} />
				{#if audioUrlB}
					<AudioPlayer url={audioUrlB} label={labelB} />
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.spec-comparison {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 16px;
	}

	.spec-question {
		font-family: var(--font-reading);
		font-size: 14px;
		color: var(--color-accent-cyan);
		text-align: center;
		margin-bottom: 16px;
		font-style: italic;
	}

	.spec-loading,
	.spec-error {
		font-family: var(--font-ui);
		font-size: 10px;
		color: var(--color-text-dim);
		text-align: center;
		padding: 24px;
	}

	.spec-error {
		color: var(--color-accent-magenta);
	}

	.spec-grid {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		gap: 12px;
		align-items: start;
	}

	.spec-side {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.spec-divider {
		display: flex;
		align-items: center;
		justify-content: center;
		padding-top: 48px;
	}

	.spec-vs {
		font-family: var(--font-ui);
		font-size: var(--font-size-xs);
		color: var(--color-text-dim);
		letter-spacing: 2px;
	}

	/* Stack on small screens */
	@media (max-width: 600px) {
		.spec-grid {
			grid-template-columns: 1fr;
			gap: 8px;
		}
		.spec-divider {
			padding-top: 0;
			padding: 4px 0;
		}
	}
</style>
