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

	let results: ModelResult[] = $state([]);
	let loading = $state(true);
	let selectedModel = $state(2); // Pure CNN by default

	const stringNames = ['E2 (low)', 'A2', 'D3', 'G3', 'B3', 'E4 (high)'];

	$effect(() => {
		fetch('/training/round_01/results.json')
			.then(r => r.json())
			.then(data => {
				results = data;
				loading = false;
			})
			.catch(() => { loading = false; });
	});

	function pct(n: number): string {
		return (n * 100).toFixed(1) + '%';
	}

	function barColor(n: number): string {
		if (n >= 0.95) return 'var(--color-accent-teal)';
		if (n >= 0.90) return 'var(--color-accent-cyan)';
		if (n >= 0.85) return 'var(--color-accent-amber)';
		return 'var(--color-accent-magenta)';
	}

	function bestModel(r: ModelResult[]): ModelResult | null {
		if (!r.length) return null;
		return r.reduce((a, b) => a.accuracy > b.accuracy ? a : b);
	}

	const crumbs = [
		{ label: 'Diary', href: '/diary' },
		{ label: 'Machine Learning', href: '/diary/machine-learning' },
		{ label: 'Round 1' },
	];
</script>

<svelte:head>
	<title>Round 1: Raw Baseline - Contrapunk Diary</title>
</svelte:head>

<DiaryNav {crumbs} />

<div class="round-page">
	<!-- Hero -->
	<header class="hero">
		<div class="round-label">ROUND 1</div>
		<h1>Raw Baseline</h1>
		<p class="hero-sub">
			No preprocessing. No augmentation. Raw audio turned into mel-spectrograms
			and fed into three different classifiers. The goal: establish what accuracy
			we get with zero effort on the data pipeline.
		</p>
	</header>

	{#if loading}
		<p class="loading-text">Loading results...</p>
	{:else if results.length}
		{@const best = bestModel(results)}
		<StatBar stats={[
			{ value: pct(best?.accuracy ?? 0), label: 'Best Accuracy', color: 'var(--color-accent-cyan)' },
			{ value: '1,380', label: 'Samples', color: 'var(--color-accent-teal)' },
			{ value: '138', label: 'Classes', color: 'var(--color-accent-magenta)' },
			{ value: '10', label: 'Per Class', color: 'var(--color-text-primary)' },
		]} />

		<!-- Station 1: Hear the Data -->
		<section class="station">
			<div class="station-label">STATION 1</div>
			<h2>The Data</h2>
			<p>
				1,380 guitar samples captured through an Audient iD14 DI input. Each of 138 positions
				(6 strings x 23 frets) recorded 10 times as individual plucks. Every sample is 0.5 seconds
				at 48kHz -- 24,000 raw audio samples per recording.
			</p>
			<p>
				Each audio clip is converted into a
				<ConceptInline term="mel-spectrogram">
					{#snippet children()}
						A mel-spectrogram converts raw audio into a 2D "image" where the x-axis is time,
						y-axis is frequency (on a perceptual scale), and color intensity is magnitude.
						The mel scale matches human hearing -- we perceive the difference between 100Hz and
						200Hz the same as 1000Hz and 2000Hz. For each 0.5s guitar sample at 48kHz, we get
						a 64x94 pixel image that a CNN can process like a photo.
					{/snippet}
				</ConceptInline>
				-- a 64x94 pixel image that represents the frequency content over time. The classifier
				never hears the audio directly; it sees these images.
			</p>
			<div class="detail-grid">
				<div class="detail-item">
					<span class="detail-key">Guitar</span>
					<span class="detail-val">Ibanez Artcore AG85 (hollow body)</span>
				</div>
				<div class="detail-item">
					<span class="detail-key">Interface</span>
					<span class="detail-val">Audient iD14 (DI input)</span>
				</div>
				<div class="detail-item">
					<span class="detail-key">Sample Rate</span>
					<span class="detail-val">48,000 Hz</span>
				</div>
				<div class="detail-item">
					<span class="detail-key">Duration</span>
					<span class="detail-val">0.5s per sample</span>
				</div>
				<div class="detail-item">
					<span class="detail-key">Features</span>
					<span class="detail-val">Log-mel spectrogram (64 mels, 1024 FFT, 256 hop)</span>
				</div>
				<div class="detail-item">
					<span class="detail-key">Preprocessing</span>
					<span class="detail-val">None</span>
				</div>
				<div class="detail-item">
					<span class="detail-key">Quality Issues</span>
					<span class="detail-val">32 clipped, 7 near-silent</span>
				</div>
				<div class="detail-item">
					<span class="detail-key">Evaluation</span>
					<span class="detail-val">
						<ConceptInline term="5-fold stratified cross-validation">
							{#snippet children()}
								With only 10 samples per class, a single train/test split would be noisy --
								one unlucky split could dramatically change results. Stratified 5-fold CV
								splits data into 5 equal parts, trains on 4 and tests on 1, rotating 5 times.
								"Stratified" means each fold preserves the class distribution. The reported
								accuracy is the average across all 5 folds.
							{/snippet}
						</ConceptInline>
					</span>
				</div>
			</div>
		</section>

		<!-- Station 2: The Models -->
		<section class="station">
			<div class="station-label">STATION 2</div>
			<h2>The Models</h2>
			<p>
				Three architectures, each taking the same mel-spectrogram input. The question:
				does the architecture matter more than the data at this scale?
			</p>

			{#each results as model, i}
				<div class="model-card" class:selected={selectedModel === i}>
					<button class="model-header" onclick={() => selectedModel = i}>
						<span class="model-name">{model.name}</span>
						<span class="model-acc" style:color={barColor(model.accuracy)}>{pct(model.accuracy)}</span>
					</button>
					<div class="model-notes">{model.notes}</div>
					{#if model.train_time}
						<div class="model-time">Train time: {model.train_time.toFixed(1)}s</div>
					{/if}

					{#if i === 0}
						<pre class="arch-diagram">Audio (24,000 samples)
  -> Mel-spectrogram (64 x 94)
  -> Flatten (6,016 features)
  -> 200 Decision Trees (vote)
  -> Predicted class (1 of 138)</pre>
					{:else if i === 1}
						<pre class="arch-diagram">Mel-spectrogram (1 x 64 x 94)
  -> Conv 16 filters -> BN -> ReLU -> MaxPool
  -> Conv 32 filters -> BN -> ReLU -> MaxPool
  -> Conv 64 filters -> BN -> ReLU -> AdaptiveAvgPool(4x4)
  -> Flatten (1,024) -> Dense 256 -> ReLU -> Dropout(0.3)
  -> Dense 138 (output)</pre>
					{:else}
						<pre class="arch-diagram">Mel-spectrogram (1 x 64 x 94)
  -> Conv 32 filters -> BN -> ReLU -> MaxPool
  -> Conv 64 filters -> BN -> ReLU -> MaxPool
  -> Conv 128 filters -> BN -> ReLU -> MaxPool
  -> Conv 256 filters -> BN -> ReLU -> GAP(1x1)
  -> Flatten (256) -> Dropout(0.4)
  -> Dense 138 (output)</pre>
					{/if}
				</div>
			{/each}

			<div class="insight-box">
				<div class="insight-label">WHY PURE CNN WINS</div>
				<p>
					With only 10 samples per class, the model with the fewest learnable parameters
					in its classifier head wins. The Pure CNN uses
					<ConceptInline term="Global Average Pooling">
						{#snippet children()}
							Traditional CNNs flatten the final feature maps and feed them through dense layers.
							For a 64-channel 4x4 feature map, that is 1024 inputs to a dense layer -- lots of
							parameters to overfit on. GAP instead averages each 4x4 map into a single value,
							giving just 256 numbers. Dramatically fewer parameters, better generalization on
							small datasets.
						{/snippet}
					</ConceptInline>
					to reduce 256 feature maps to 256 numbers, then a single linear layer to 138 classes.
					That is around 35K parameters in the classifier vs around 265K in the Hybrid CNN.
				</p>
			</div>
		</section>

		<!-- Station 3: The Results -->
		<section class="station">
			<div class="station-label">STATION 3</div>
			<h2>The Results</h2>

			<!-- Model selector tabs -->
			<div class="model-tabs">
				{#each results as model, i}
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

			<!-- Per-string accuracy bars -->
			<div class="string-bars">
				<div class="bars-title">Per-String Accuracy -- {results[selectedModel].name}</div>
				{#each stringNames as name, i}
					{@const acc = results[selectedModel].per_string[i]}
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

			<!-- Overall comparison -->
			<div class="comparison">
				<div class="bars-title">Overall Accuracy Comparison</div>
				{#each results as model, i}
					<div class="bar-row">
						<span class="bar-label">{model.name}</span>
						<div class="bar-track">
							<div
								class="bar-fill"
								style="width: {(model.accuracy * 100).toFixed(1)}%; background: {barColor(model.accuracy)}"
							></div>
						</div>
						<span class="bar-value" style:color={barColor(model.accuracy)}>{pct(model.accuracy)}</span>
					</div>
				{/each}
			</div>
		</section>

		<!-- Station 4: What We Learned -->
		<section class="station">
			<div class="station-label">STATION 4</div>
			<h2>What We Learned</h2>
			<ul class="findings">
				<li>
					<span class="finding-icon check">OK</span>
					<div>
						<strong>Raw mel-spectrograms work surprisingly well.</strong>
						96.2% accuracy on 138 classes with zero preprocessing shows the audio signal
						is rich enough for classification even without feature engineering.
					</div>
				</li>
				<li>
					<span class="finding-icon check">OK</span>
					<div>
						<strong>Low E string is hardest.</strong>
						The Random Forest gets only 86.1% on E2. The thick wound string has more
						complex harmonics and the frets are physically closer together in the low register.
					</div>
				</li>
				<li>
					<span class="finding-icon check">OK</span>
					<div>
						<strong>Pure CNN > RF > Hybrid CNN.</strong>
						Global Average Pooling beats the dense bottleneck when data is scarce.
						The RF's strong 93.3% validates that the features are genuinely informative.
					</div>
				</li>
				<li>
					<span class="finding-icon warn">!!</span>
					<div>
						<strong>32 clipped samples left in intentionally.</strong>
						If cleaning them improves accuracy in a later round, we will know the
						impact precisely. No guessing -- measure everything.
					</div>
				</li>
			</ul>
		</section>

		<!-- Station 5: Visualizations -->
		<section class="station">
			<div class="station-label">STATION 5</div>
			<h2>Visualizations</h2>
			<p>
				Training artifacts generated from the {results[selectedModel].name} model.
				Select a model above to switch between them.
			</p>

			<div class="viz-grid">
				<div class="viz-item">
					<div class="viz-label">Per-String Accuracy Chart</div>
					<img
						src="/training/round_01/per_string_{results[selectedModel].name.toLowerCase().replace(/ /g, '_')}.png"
						alt="{results[selectedModel].name} per-string accuracy chart"
						class="viz-img"
					/>
				</div>
				<div class="viz-item">
					<div class="viz-label">Per-Fret Accuracy Heatmap</div>
					<img
						src="/training/round_01/fret_heatmap_{results[selectedModel].name.toLowerCase().replace(/ /g, '_')}.png"
						alt="{results[selectedModel].name} fret accuracy heatmap"
						class="viz-img"
					/>
				</div>
			</div>
		</section>

		<!-- Station 6: Next Steps -->
		<section class="station">
			<div class="station-label">STATION 6</div>
			<h2>Next Steps</h2>
			<p>
				The baseline is set. Now we improve one thing at a time.
			</p>
			<div class="next-link">
				<a href="/diary/machine-learning/round-2">
					Round 2: Onset Alignment ->
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

	/* Detail grid */
	.detail-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1px;
		background: var(--color-border);
		border: 1px solid var(--color-border);
		margin-top: 16px;
	}
	.detail-item {
		background: var(--color-bg-panel);
		padding: 10px 12px;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.detail-key {
		font-size: 10px;
		color: var(--color-text-dim);
		text-transform: uppercase;
		letter-spacing: 1px;
	}
	.detail-val {
		font-size: 13px;
		color: var(--color-text-primary);
	}

	/* Model cards */
	.model-card {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 16px;
		margin-bottom: 8px;
	}
	.model-card.selected {
		border-color: rgba(51, 221, 255, 0.3);
	}
	.model-header {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		width: 100%;
		background: none;
		border: none;
		padding: 0;
		cursor: pointer;
		text-align: left;
	}
	.model-name {
		font-family: var(--font-reading);
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text-primary);
	}
	.model-acc {
		font-family: var(--font-pixel);
		font-size: 16px;
	}
	.model-notes {
		font-size: 12px;
		color: var(--color-text-dim);
		margin-top: 4px;
	}
	.model-time {
		font-size: 11px;
		color: var(--color-text-dim);
		margin-top: 4px;
	}
	.arch-diagram {
		background: var(--color-bg-deep);
		border: 1px solid var(--color-border);
		padding: 12px 16px;
		margin-top: 12px;
		font-family: monospace;
		font-size: 12px;
		color: var(--color-text-dim);
		line-height: 1.6;
		overflow-x: auto;
		white-space: pre;
	}

	/* Insight box */
	.insight-box {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-accent-amber);
		border-left-width: 3px;
		padding: 16px;
		margin-top: 16px;
	}
	.insight-label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-amber);
		letter-spacing: 2px;
		margin-bottom: 8px;
	}
	.insight-box p {
		font-size: 13px;
		color: var(--color-text-secondary);
		line-height: 1.7;
		margin: 0;
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

	/* Accuracy bars */
	.string-bars, .comparison {
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

	/* Findings list */
	.findings {
		list-style: none;
		padding: 0;
		margin: 0;
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
		font-size: 8px;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		margin-top: 2px;
	}
	.finding-icon.check {
		color: var(--color-accent-teal);
	}
	.finding-icon.warn {
		color: var(--color-accent-amber);
	}

	/* Visualizations */
	.viz-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: 16px;
		margin-top: 16px;
	}
	.viz-item {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 12px;
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

	/* Next link */
	.next-link {
		margin-top: 16px;
	}
	.next-link a {
		display: inline-block;
		font-family: var(--font-pixel);
		font-size: var(--font-size-sm);
		color: var(--color-accent-cyan);
		text-decoration: none;
		padding: 12px 20px;
		border: 1px solid var(--color-accent-cyan-dim);
	}
	.next-link a:hover {
		background: rgba(51, 221, 255, 0.05);
		border-color: var(--color-accent-cyan);
	}
</style>
