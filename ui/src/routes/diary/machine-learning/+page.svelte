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
	let r4Results: ModelResult[] = $state([]);
	let r5Results: ModelResult[] = $state([]);
	let loading = $state(true);

	$effect(() => {
		Promise.all([
			fetch('/training/round_01/results.json').then(r => r.json()),
			fetch('/training/round_02/results.json').then(r => r.json()),
			fetch('/training/round_03/results.json').then(r => r.json()),
			fetch('/training/round_04/results.json').then(r => r.json()),
			fetch('/training/round_05/results.json').then(r => r.json()),
		]).then(([r1, r2, r3, r4, r5]) => {
			r1Results = r1;
			r2Results = r2;
			r3Results = r3;
			r4Results = r4;
			r5Results = r5;
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
			138 positions on a guitar neck. 1,380 audio samples. Five training rounds.
			Three model architectures. The iterative journey that taught us physics
			works better than pattern matching.
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
				accuracy={bestAccuracy(r4Results)}
				delta={bestAccuracy(r4Results) - bestAccuracy(r3Results)}
				description="Added 11 physics-based harmonic ratio features via Goertzel algorithm. Hybrid CNN E2 jumped +5.2% but Pure CNN unchanged — it already learns harmonics implicitly."
				date="Apr 2, 2026"
				complete={true}
				href="/diary/machine-learning/round-4"
			/>

			<RoundCard
				number={5}
				title="Data Augmentation"
				accuracy={bestAccuracy(r5Results)}
				delta={bestAccuracy(r5Results) - bestAccuracy(r4Results)}
				description="4x training data via gain, noise, shift, stretch augmentation. Hybrid CNN hit 95.1% (new best) but Pure CNN stayed flat. Round 3's 97.3% still the overall best."
				date="Apr 3, 2026"
				complete={true}
				href="/diary/machine-learning/round-5"
			/>

			<!-- The Pivot — not a RoundCard, a special narrative link -->
			<a class="pivot-card" href="/diary/machine-learning/the-pivot">
				<div class="pivot-icon">
					<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
						<polyline points="23 6 13.5 15.5 8.5 10.5 1 18"></polyline>
						<polyline points="17 6 23 6 23 12"></polyline>
					</svg>
				</div>
				<div class="pivot-body">
					<div class="pivot-label">CONCLUSION</div>
					<div class="pivot-title">The Pivot: From ML to Physics</div>
					<div class="pivot-desc">
						Five rounds taught us the ceiling is physics-bounded.
						Research showed DSP achieves 98.5% with 1 calibration sample per string.
					</div>
				</div>
			</a>
		{/if}
	</section>

	<section class="discovery">
		<div class="label">WHAT WE DISCOVERED</div>
		<p>
			After five rounds of iterative training, our best model reached 97.3%.
			Then we discovered that a physics-based approach using the
			<strong>inharmonicity B coefficient</strong> achieves 98.5% with just one
			calibration sample per string. The full story is in
			<a href="/diary/machine-learning/the-pivot">The Pivot</a>.
		</p>
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

	/* Pivot card */
	.pivot-card {
		display: flex;
		gap: 16px;
		align-items: flex-start;
		text-decoration: none;
		background: var(--color-bg-panel);
		border: 1px solid rgba(255, 170, 51, 0.3);
		border-left: 3px solid rgba(255, 170, 51, 0.6);
		padding: 16px;
		margin-top: 8px;
	}
	.pivot-card:hover {
		background: rgba(255, 170, 51, 0.03);
		border-color: rgba(255, 170, 51, 0.5);
	}
	.pivot-icon {
		flex-shrink: 0;
		width: 40px;
		height: 40px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-accent-amber, #ffaa33);
	}
	.pivot-body {
		flex: 1;
	}
	.pivot-label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-amber, #ffaa33);
		letter-spacing: 2px;
		margin-bottom: 4px;
	}
	.pivot-title {
		font-family: var(--font-reading);
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text-primary);
	}
	.pivot-desc {
		color: var(--color-text-secondary);
		font-size: 13px;
		margin-top: 4px;
		line-height: 1.5;
	}

	/* Discovery section */
	.discovery {
		padding: 24px;
		border-bottom: 1px solid var(--color-border);
	}
	.discovery p {
		font-size: 14px;
		color: var(--color-text-secondary);
		line-height: 1.7;
		margin: 0;
	}
	.discovery strong {
		color: var(--color-text-primary);
	}
	.discovery a {
		color: var(--color-accent-cyan);
		text-decoration: none;
		border-bottom: 1px dashed var(--color-accent-cyan-dim, rgba(51, 221, 255, 0.3));
	}
	.discovery a:hover {
		border-bottom-color: var(--color-accent-cyan);
	}
</style>
