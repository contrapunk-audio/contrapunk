<script lang="ts">
	import DiaryNav from '$lib/components/diary/DiaryNav.svelte';
	import RoundCard from '$lib/components/diary/RoundCard.svelte';

	type ModelResult = {
		name: string;
		accuracy: number;
		train_time?: number;
		per_string: number[];
		notes: string;
	};

	let r1Results: ModelResult[] = $state([]);
	let r2Results: ModelResult[] = $state([]);
	let r3Results: ModelResult[] = $state([]);
	let loading = $state(true);

	$effect(() => {
		Promise.all([
			fetch('/training/round_01/results.json').then(r => r.json()),
			fetch('/training/round_02/results.json').then(r => r.json()),
			fetch('/training/round_03/results.json').then(r => r.json()),
		]).then(([r1, r2, r3]) => {
			r1Results = r1;
			r2Results = r2;
			r3Results = r3;
			loading = false;
		}).catch(() => { loading = false; });
	});

	function bestAccuracy(results: ModelResult[]): number {
		if (!results.length) return 0;
		return Math.max(...results.map(r => r.accuracy));
	}

	function pct(n: number): string {
		return (n * 100).toFixed(1) + '%';
	}

	const crumbs = [
		{ label: 'Diary', href: '/diary' },
		{ label: 'Machine Learning' },
	];
</script>

<svelte:head>
	<title>Machine Learning - Contrapunk Diary</title>
</svelte:head>

<DiaryNav {crumbs} />

<div class="chapter">
	<header class="chapter-header">
		<div class="label">CHAPTER</div>
		<h1>Teaching a Model to Identify Guitar Positions</h1>
		<p>
			138 positions on a guitar neck. 1,380 audio samples. Three model architectures.
			We train iteratively — changing one thing at a time, measuring the impact, documenting everything.
		</p>
	</header>

	<section class="approach">
		<div class="label">THE APPROACH</div>
		<div class="steps">
			<div class="step"><div class="step-num">1</div><div class="step-text">Train on raw data</div><div class="step-sub">Establish baseline</div></div>
			<div class="step"><div class="step-num">2</div><div class="step-text">Change one thing</div><div class="step-sub">Measure impact</div></div>
			<div class="step"><div class="step-num">3</div><div class="step-text">Document result</div><div class="step-sub">Learn what matters</div></div>
			<div class="step"><div class="step-num">4</div><div class="step-text">Repeat</div><div class="step-sub">Until production-ready</div></div>
		</div>
	</section>

	<section class="rounds">
		<div class="label">TRAINING ROUNDS</div>

		{#if loading}
			<p class="loading-text">Loading results...</p>
		{:else}
			<RoundCard
				number={1}
				title="Raw Baseline"
				accuracy={bestAccuracy(r1Results)}
				description="No preprocessing. Raw mel-spectrograms into 3 classifiers. Pure CNN wins."
				date="Mar 31, 2026"
				complete={true}
				href="/diary/machine-learning/round-1"
			/>

			<RoundCard
				number={2}
				title="Onset Alignment"
				accuracy={bestAccuracy(r2Results)}
				delta={bestAccuracy(r2Results) - bestAccuracy(r1Results)}
				description="Aligned all samples to pluck onset. No accuracy change — capture tool was already triggering well."
				date="Apr 1, 2026"
				complete={true}
				href="/diary/machine-learning/round-2"
			/>

			<RoundCard
				number={3}
				title="Quality Cleanup"
				accuracy={bestAccuracy(r3Results)}
				delta={bestAccuracy(r3Results) - bestAccuracy(r2Results)}
				description="Removed 32 clipped + 7 silent samples. First measurable improvement: +0.9% Pure CNN, +1.5% Random Forest."
				date="Apr 1, 2026"
				complete={true}
				href="/diary/machine-learning/round-3"
			/>

			<RoundCard
				number={4}
				title="Goertzel Harmonics"
				accuracy={0}
				description="Physics-based harmonic ratio features for string identification."
				date=""
				complete={false}
				href=""
			/>
		{/if}
	</section>

	<section class="tools">
		<a class="tool-card" href="/diary/machine-learning/explore" style="border-color: rgba(255, 51, 136, 0.3);">
			<div class="tool-label" style="color: var(--color-accent-magenta);">EXPLORE DATA</div>
			<div class="tool-desc">Browse 138 classes, hear samples</div>
		</a>
		<a class="tool-card" href="/diary/machine-learning/playground" style="border-color: rgba(0, 204, 170, 0.3);">
			<div class="tool-label" style="color: var(--color-accent-teal);">LIVE PLAYGROUND</div>
			<div class="tool-desc">Try the model in your browser</div>
		</a>
	</section>
</div>

<style>
	.chapter {
		max-width: 800px;
		margin: 0 auto;
		padding-bottom: 64px;
	}
	.chapter-header {
		padding: 32px 24px 24px;
		border-bottom: 1px solid var(--color-border);
	}
	.label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-magenta);
		letter-spacing: 2px;
		margin-bottom: 12px;
	}
	h1 {
		font-family: var(--font-reading);
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text-primary);
		margin: 0;
	}
	.chapter-header p {
		font-size: 14px;
		color: var(--color-text-secondary);
		margin-top: 8px;
		line-height: 1.7;
		max-width: 600px;
	}
	.approach {
		padding: 24px;
		border-bottom: 1px solid var(--color-border);
	}
	.steps {
		display: flex;
		gap: 2px;
	}
	.step {
		flex: 1;
		background: var(--color-bg-panel);
		padding: 12px;
		text-align: center;
	}
	.step:first-child { border-radius: 4px 0 0 4px; }
	.step:last-child { border-radius: 0 4px 4px 0; }
	.step-num {
		font-family: var(--font-pixel);
		font-size: 18px;
		color: var(--color-accent-cyan);
	}
	.step-text {
		font-size: 12px;
		color: var(--color-text-primary);
		margin-top: 4px;
	}
	.step-sub {
		font-size: 10px;
		color: var(--color-text-dim);
		margin-top: 2px;
	}
	.rounds {
		padding: 24px;
	}
	.loading-text {
		color: var(--color-text-dim);
		font-size: 14px;
	}
	.tools {
		padding: 0 24px 24px;
		display: flex;
		gap: 12px;
	}
	.tool-card {
		flex: 1;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 16px;
		text-align: center;
		text-decoration: none;
	}
	.tool-card:hover {
		background: var(--color-widget-bg);
	}
	.tool-label {
		font-family: var(--font-pixel);
		font-size: 11px;
	}
	.tool-desc {
		color: var(--color-text-secondary);
		font-size: 11px;
		margin-top: 4px;
	}
</style>
