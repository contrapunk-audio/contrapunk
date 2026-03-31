<script lang="ts">
	import DiaryNav from '$lib/components/diary/DiaryNav.svelte';
	import StatBar from '$lib/components/diary/StatBar.svelte';
	import ConceptInline from '$lib/components/diary/ConceptInline.svelte';

	type ModelResult = {
		name: string;
		accuracy: number;
		train_time?: number;
		per_string: number[];
		notes: string;
		training_curves?: boolean;
	};

	let r1Results: ModelResult[] = $state([]);
	let r2Results: ModelResult[] = $state([]);
	let loading = $state(true);
	let selectedModel = $state(2); // Pure CNN by default

	const stringNames = ['E2 (low)', 'A2', 'D3', 'G3', 'B3', 'E4 (high)'];

	$effect(() => {
		Promise.all([
			fetch('/training/round_01/results.json').then(r => r.json()),
			fetch('/training/round_02/results.json').then(r => r.json()),
		]).then(([r1, r2]) => {
			r1Results = r1;
			r2Results = r2;
			loading = false;
		}).catch(() => { loading = false; });
	});

	function pct(n: number): string {
		return (n * 100).toFixed(1) + '%';
	}

	function deltaPct(a: number, b: number): string {
		const d = b - a;
		const sign = d > 0 ? '+' : '';
		return sign + (d * 100).toFixed(1) + '%';
	}

	function barColor(n: number): string {
		if (n >= 0.95) return 'var(--color-accent-teal)';
		if (n >= 0.90) return 'var(--color-accent-cyan)';
		if (n >= 0.85) return 'var(--color-accent-amber)';
		return 'var(--color-accent-magenta)';
	}

	function deltaColor(d: number): string {
		if (d > 0.005) return 'var(--color-accent-teal)';
		if (d < -0.005) return 'var(--color-accent-magenta)';
		return 'var(--color-text-dim)';
	}

	function bestAccuracy(results: ModelResult[]): number {
		if (!results.length) return 0;
		return Math.max(...results.map(r => r.accuracy));
	}

	const crumbs = [
		{ label: 'Diary', href: '/diary' },
		{ label: 'Machine Learning', href: '/diary/machine-learning' },
		{ label: 'Round 2' },
	];
</script>

<svelte:head>
	<title>Round 2: Onset Alignment - Contrapunk Diary</title>
</svelte:head>

<DiaryNav {crumbs} />

<div class="round-page">
	<!-- Hero -->
	<header class="hero">
		<div class="round-label">ROUND 2</div>
		<h1>Onset Alignment</h1>
		<p class="hero-sub">
			The hypothesis: aligning every sample to the exact moment of the pluck
			will improve classification by ensuring spectrograms start at the attack transient.
			The result: no measurable change.
		</p>
	</header>

	{#if loading}
		<p class="loading-text">Loading results...</p>
	{:else if r1Results.length && r2Results.length}
		{@const r1Best = bestAccuracy(r1Results)}
		{@const r2Best = bestAccuracy(r2Results)}
		{@const delta = r2Best - r1Best}

		<!-- Comparison Banner -->
		<div class="comparison-banner">
			<div class="comp-item">
				<div class="comp-label">ROUND 1</div>
				<div class="comp-value">{pct(r1Best)}</div>
				<div class="comp-sub">Raw baseline</div>
			</div>
			<div class="comp-arrow">-></div>
			<div class="comp-item">
				<div class="comp-label">ROUND 2</div>
				<div class="comp-value">{pct(r2Best)}</div>
				<div class="comp-sub">Onset-aligned</div>
			</div>
			<div class="comp-item delta">
				<div class="comp-label">DELTA</div>
				<div class="comp-value" style:color={deltaColor(delta)}>{deltaPct(r1Best, r2Best)}</div>
				<div class="comp-sub">Change in best accuracy</div>
			</div>
		</div>

		<StatBar stats={[
			{ value: pct(r2Best), label: 'Best Accuracy', color: 'var(--color-accent-cyan)' },
			{ value: deltaPct(r1Best, r2Best), label: 'vs Round 1', color: deltaColor(delta) },
			{ value: '21.8ms', label: 'Median Onset', color: 'var(--color-accent-teal)' },
			{ value: '0', label: 'Impact', color: 'var(--color-text-dim)' },
		]} />

		<!-- Station 1: What Changed -->
		<section class="station">
			<div class="station-label">STATION 1</div>
			<h2>What Changed</h2>
			<p>
				In Round 1, audio recording started when the capture tool detected a signal above
				threshold. But depending on playing dynamics and the string's sustain characteristics,
				there could be varying amounts of silence or pre-ring before the actual pluck attack.
			</p>
			<p>
				For Round 2, every sample was processed through an
				<ConceptInline term="onset detection algorithm">
					{#snippet children()}
						Onset detection identifies the precise moment a note begins. We use energy-based
						onset detection: compute the short-time energy in sliding windows, then find the
						point where energy rises sharply. The threshold is set relative to each sample's
						peak energy, making it adaptive to different playing dynamics. Once the onset is
						found, we trim everything before it and re-extract the mel-spectrogram.
					{/snippet}
				</ConceptInline>
				that finds the exact moment of the pluck attack, trims everything before it,
				and re-extracts the mel-spectrogram from the aligned audio.
			</p>
			<p>
				The expectation was that consistent alignment would improve classification,
				especially for fret discrimination where the attack transient carries the
				most distinctive information.
			</p>
		</section>

		<!-- Station 2: Onset Distribution -->
		<section class="station">
			<div class="station-label">STATION 2</div>
			<h2>Onset Distribution</h2>
			<p>
				The onset detection found that the median onset was at 21.8ms from the start
				of each recording. Most samples had their pluck very close to the beginning --
				the capture tool was already triggering well.
			</p>
			<div class="viz-item">
				<div class="viz-label">Onset Timing Distribution Across All Samples</div>
				<img
					src="/training/round_02/onset_distribution.png"
					alt="Distribution of detected onset times across all 1,380 samples"
					class="viz-img"
				/>
			</div>
			<p class="viz-caption">
				The tight clustering around 20ms explains why alignment had no impact --
				there was very little misalignment to correct in the first place.
			</p>
		</section>

		<!-- Station 3: Model-by-Model Comparison -->
		<section class="station">
			<div class="station-label">STATION 3</div>
			<h2>Model-by-Model Comparison</h2>

			<!-- Model selector tabs -->
			<div class="model-tabs">
				{#each r2Results as model, i}
					<button
						class="model-tab"
						class:active={selectedModel === i}
						onclick={() => selectedModel = i}
					>
						{model.name}
						<span class="tab-acc">{pct(model.accuracy)}</span>
					</button>
				{/each}
			</div>

			<!-- Per-model comparison table -->
			<div class="model-compare">
				<div class="mc-header">
					<span class="mc-col"></span>
					<span class="mc-col">Round 1</span>
					<span class="mc-col">Round 2</span>
					<span class="mc-col">Delta</span>
				</div>
				<div class="mc-row overall">
					<span class="mc-col mc-label">Overall</span>
					<span class="mc-col">{pct(r1Results[selectedModel].accuracy)}</span>
					<span class="mc-col">{pct(r2Results[selectedModel].accuracy)}</span>
					<span class="mc-col" style:color={deltaColor(r2Results[selectedModel].accuracy - r1Results[selectedModel].accuracy)}>
						{deltaPct(r1Results[selectedModel].accuracy, r2Results[selectedModel].accuracy)}
					</span>
				</div>
				{#each stringNames as name, i}
					{@const d = r2Results[selectedModel].per_string[i] - r1Results[selectedModel].per_string[i]}
					<div class="mc-row">
						<span class="mc-col mc-label">{name}</span>
						<span class="mc-col">{pct(r1Results[selectedModel].per_string[i])}</span>
						<span class="mc-col">{pct(r2Results[selectedModel].per_string[i])}</span>
						<span class="mc-col" style:color={deltaColor(d)}>{deltaPct(r1Results[selectedModel].per_string[i], r2Results[selectedModel].per_string[i])}</span>
					</div>
				{/each}
			</div>

			<!-- Per-string bars for Round 2 -->
			<div class="string-bars">
				<div class="bars-title">Per-String Accuracy -- {r2Results[selectedModel].name} (Round 2)</div>
				{#each stringNames as name, i}
					{@const acc = r2Results[selectedModel].per_string[i]}
					<div class="bar-row">
						<span class="bar-label">{name}</span>
						<div class="bar-track">
							<div
								class="bar-fill"
								style="width: {(acc * 100).toFixed(1)}%; background: {barColor(acc)}"
							></div>
						</div>
						<span class="bar-value" style:color={barColor(acc)}>{pct(acc)}</span>
					</div>
				{/each}
			</div>
		</section>

		<!-- Station 4: Before/After Spectrograms -->
		<section class="station">
			<div class="station-label">STATION 4</div>
			<h2>Before and After</h2>
			<p>
				Visual comparison of spectrograms before and after onset alignment.
				The difference is subtle -- most spectrograms shift by only a few pixels.
			</p>
			<div class="viz-item">
				<div class="viz-label">Before/After Spectrogram Comparison</div>
				<img
					src="/training/round_02/before_after_spectrograms.png"
					alt="Spectrograms before and after onset alignment"
					class="viz-img"
				/>
			</div>
			<div class="viz-item">
				<div class="viz-label">Accuracy Comparison Across Models</div>
				<img
					src="/training/round_02/comparison_bars.png"
					alt="Round 1 vs Round 2 accuracy comparison"
					class="viz-img"
				/>
			</div>
		</section>

		<!-- Station 5: Why Zero Impact -->
		<section class="station">
			<div class="station-label">STATION 5</div>
			<h2>Why Zero Impact</h2>
			<p>
				Three factors explain why onset alignment made no measurable difference:
			</p>
			<ul class="findings">
				<li>
					<span class="finding-icon num">1</span>
					<div>
						<strong>The capture tool was already triggering well.</strong>
						Median onset at 21.8ms means the tool was already starting recording
						very close to the pluck. There was minimal misalignment to correct.
					</div>
				</li>
				<li>
					<span class="finding-icon num">2</span>
					<div>
						<strong>CNN
						<ConceptInline term="pooling layers provide translation invariance">
							{#snippet children()}
								Max pooling and average pooling operations reduce spatial dimensions by
								summarizing local regions. A MaxPool(2x2) takes the maximum value from
								each 2x2 block. This makes the network partially invariant to small
								translations -- if a feature shifts by a few pixels, the pooled output
								is similar. With 3-4 pooling layers, the Pure CNN can tolerate the
								small temporal shifts that onset alignment was meant to fix.
							{/snippet}
						</ConceptInline>.</strong>
						The Pure CNN has 3 MaxPool layers, reducing the 94-pixel time axis to roughly
						11 pixels. A 20ms shift at 48kHz with hop=256 is about 4 spectrogram columns --
						within what pooling can absorb.
					</div>
				</li>
				<li>
					<span class="finding-icon num">3</span>
					<div>
						<strong>The discriminative features are not onset-dependent.</strong>
						String and fret identity comes from harmonic content (which frequencies are
						present and their relative strengths), not from the exact timing of the attack.
						The steady-state portion of the note carries enough information.
					</div>
				</li>
			</ul>
			<div class="takeaway-box">
				<div class="takeaway-label">TAKEAWAY</div>
				<p>
					Not every preprocessing step helps. The value of this round is knowing that
					onset alignment is not a factor for this dataset and capture setup. One fewer
					thing to worry about in the pipeline.
				</p>
			</div>
		</section>

		<!-- Station 6: Next Steps -->
		<section class="station">
			<div class="station-label">STATION 6</div>
			<h2>Next Steps</h2>
			<p>
				Onset alignment ruled out. Next: remove the 32 clipped and 7 near-silent
				samples to see if data quality cleaning has a measurable impact.
			</p>
			<div class="nav-links">
				<a href="/diary/machine-learning/round-1" class="nav-link prev">
					&lt;- Round 1: Raw Baseline
				</a>
				<a href="/diary/machine-learning" class="nav-link next">
					Back to Overview ->
				</a>
			</div>
		</section>
	{/if}
</div>

<style>
	.round-page {
		max-width: 800px;
		margin: 0 auto;
		padding-bottom: 64px;
	}

	.hero {
		padding: 32px 24px 24px;
		border-bottom: 1px solid var(--color-border);
	}
	.round-label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-magenta);
		letter-spacing: 3px;
		margin-bottom: 8px;
	}
	.hero h1 {
		font-family: var(--font-reading);
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text-primary);
		margin: 0;
	}
	.hero-sub {
		font-size: 14px;
		color: var(--color-text-secondary);
		margin-top: 8px;
		line-height: 1.7;
		max-width: 600px;
	}

	.loading-text {
		color: var(--color-text-dim);
		font-size: 14px;
		padding: 24px;
	}

	/* Comparison Banner */
	.comparison-banner {
		display: flex;
		align-items: center;
		gap: 0;
		border-bottom: 1px solid var(--color-border);
	}
	.comp-item {
		flex: 1;
		padding: 20px 16px;
		text-align: center;
		border-right: 1px solid var(--color-border);
	}
	.comp-item:last-child {
		border-right: none;
	}
	.comp-item.delta {
		background: var(--color-bg-panel);
	}
	.comp-arrow {
		padding: 0 12px;
		font-family: monospace;
		font-size: 18px;
		color: var(--color-text-dim);
		flex-shrink: 0;
	}
	.comp-label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-magenta);
		letter-spacing: 2px;
		margin-bottom: 4px;
	}
	.comp-value {
		font-family: var(--font-pixel);
		font-size: 22px;
		color: var(--color-accent-cyan);
	}
	.comp-sub {
		font-size: 10px;
		color: var(--color-text-dim);
		margin-top: 4px;
	}

	/* Stations */
	.station {
		padding: 24px;
		border-bottom: 1px solid var(--color-border);
	}
	.station-label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-magenta);
		letter-spacing: 2px;
		margin-bottom: 12px;
	}
	.station h2 {
		font-family: var(--font-reading);
		font-size: 20px;
		font-weight: 700;
		color: var(--color-text-primary);
		margin: 0 0 12px 0;
	}
	.station p {
		font-size: 14px;
		color: var(--color-text-secondary);
		line-height: 1.7;
		margin: 0 0 12px 0;
	}

	/* Model tabs */
	.model-tabs {
		display: flex;
		gap: 2px;
		margin-bottom: 20px;
	}
	.model-tab {
		flex: 1;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 10px 8px;
		cursor: pointer;
		text-align: center;
		font-size: 12px;
		color: var(--color-text-secondary);
		font-family: var(--font-reading);
	}
	.model-tab.active {
		border-color: var(--color-accent-cyan);
		color: var(--color-accent-cyan);
		background: rgba(51, 221, 255, 0.05);
	}
	.tab-acc {
		display: block;
		font-family: var(--font-pixel);
		font-size: 14px;
		margin-top: 4px;
	}

	/* Model comparison table */
	.model-compare {
		border: 1px solid var(--color-border);
		margin-bottom: 24px;
	}
	.mc-header {
		display: flex;
		background: var(--color-bg-panel);
		border-bottom: 1px solid var(--color-border);
	}
	.mc-header .mc-col {
		font-family: var(--font-pixel);
		font-size: 8px;
		color: var(--color-text-dim);
		letter-spacing: 1px;
	}
	.mc-row {
		display: flex;
		border-bottom: 1px solid var(--color-border);
	}
	.mc-row:last-child {
		border-bottom: none;
	}
	.mc-row.overall {
		background: var(--color-bg-panel);
	}
	.mc-col {
		flex: 1;
		padding: 8px 12px;
		font-size: 12px;
		color: var(--color-text-secondary);
		font-family: monospace;
	}
	.mc-col.mc-label {
		font-family: var(--font-reading);
		color: var(--color-text-primary);
		font-size: 12px;
	}

	/* Accuracy bars */
	.string-bars {
		margin-bottom: 24px;
	}
	.bars-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin-bottom: 12px;
	}
	.bar-row {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-bottom: 8px;
	}
	.bar-label {
		width: 100px;
		font-size: 12px;
		color: var(--color-text-secondary);
		font-family: monospace;
		text-align: right;
		flex-shrink: 0;
	}
	.bar-track {
		flex: 1;
		height: 14px;
		background: var(--color-bg-deep);
		border: 1px solid var(--color-border);
		position: relative;
		overflow: hidden;
	}
	.bar-fill {
		height: 100%;
		transition: width 0.3s ease;
	}
	.bar-value {
		width: 50px;
		font-family: var(--font-pixel);
		font-size: 11px;
		text-align: right;
		flex-shrink: 0;
	}

	/* Visualizations */
	.viz-item {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 12px;
		margin-bottom: 16px;
	}
	.viz-label {
		font-size: 11px;
		color: var(--color-text-dim);
		text-transform: uppercase;
		letter-spacing: 1px;
		margin-bottom: 8px;
	}
	.viz-img {
		width: 100%;
		border: 1px solid var(--color-border);
		display: block;
	}
	.viz-caption {
		font-size: 12px;
		color: var(--color-text-dim);
		font-style: italic;
		margin-top: 8px;
	}

	/* Findings list */
	.findings {
		list-style: none;
		padding: 0;
		margin: 0 0 16px 0;
	}
	.findings li {
		display: flex;
		gap: 12px;
		padding: 12px 0;
		border-bottom: 1px solid var(--color-border);
		font-size: 13px;
		color: var(--color-text-secondary);
		line-height: 1.6;
	}
	.findings li:last-child {
		border-bottom: none;
	}
	.findings strong {
		color: var(--color-text-primary);
	}
	.finding-icon {
		flex-shrink: 0;
		font-family: var(--font-pixel);
		font-size: 10px;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		margin-top: 2px;
	}
	.finding-icon.num {
		color: var(--color-accent-cyan);
	}

	/* Takeaway box */
	.takeaway-box {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-accent-teal);
		border-left-width: 3px;
		padding: 16px;
		margin-top: 16px;
	}
	.takeaway-label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-teal);
		letter-spacing: 2px;
		margin-bottom: 8px;
	}
	.takeaway-box p {
		font-size: 13px;
		color: var(--color-text-secondary);
		line-height: 1.7;
		margin: 0;
	}

	/* Navigation links */
	.nav-links {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		margin-top: 16px;
	}
	.nav-link {
		font-family: var(--font-pixel);
		font-size: var(--font-size-sm);
		color: var(--color-accent-cyan);
		text-decoration: none;
		padding: 12px 20px;
		border: 1px solid var(--color-accent-cyan-dim);
	}
	.nav-link:hover {
		background: rgba(51, 221, 255, 0.05);
		border-color: var(--color-accent-cyan);
	}
</style>
