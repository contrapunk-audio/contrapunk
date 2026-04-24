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
	let r3Results: ModelResult[] = $state([]);
	let loading = $state(true);
	let selectedModel = $state(2); // Pure CNN by default

	const stringNames = ['E2 (low)', 'A2', 'D3', 'G3', 'B3', 'E4 (high)'];

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
		{ label: 'Round 3' },
	];
</script>

<svelte:head>
	<title>Round 3: Quality Cleanup - Contrapunk Diary</title>
</svelte:head>

<DiaryNav {crumbs} />

<div class="round-page">
	<!-- Hero -->
	<header class="hero">
		<div class="round-label">ROUND 03</div>
		<h1>Quality Cleanup</h1>
		<p class="hero-sub">
			The first round with measurable improvement. Removed 32 clipped samples and
			7 near-silent recordings from 1,380 total, leaving 1,341 clean samples.
			Data quality beats data quantity -- removing 39 bad samples helped more than
			aligning 1,380.
		</p>
	</header>

	{#if loading}
		<p class="loading-text">Loading results...</p>
	{:else if r1Results.length && r2Results.length && r3Results.length}
		{@const r2Best = bestAccuracy(r2Results)}
		{@const r3Best = bestAccuracy(r3Results)}
		{@const delta = r3Best - r2Best}

		<StatBar stats={[
			{ value: pct(r3Best), label: 'Best Accuracy', color: 'var(--color-accent-cyan)' },
			{ value: deltaPct(r2Best, r3Best), label: 'vs Round 2', color: 'var(--color-accent-teal)' },
			{ value: '1,341', label: 'Clean Samples', color: 'var(--color-accent-teal)' },
			{ value: '39', label: 'Removed', color: 'var(--color-accent-magenta)' },
		]} />

		<!-- Comparison Banner -->
		<div class="comparison-banner">
			<div class="comp-item">
				<div class="comp-label">ROUND 2</div>
				<div class="comp-value">{pct(r2Best)}</div>
				<div class="comp-sub">Onset-aligned</div>
			</div>
			<div class="comp-arrow">
				<svg width="24" height="16" viewBox="0 0 24 16" fill="none">
					<path d="M0 8h20M14 2l6 6-6 6" stroke="var(--color-text-dim)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
				</svg>
			</div>
			<div class="comp-item">
				<div class="comp-label">ROUND 3</div>
				<div class="comp-value">{pct(r3Best)}</div>
				<div class="comp-sub">Quality-cleaned</div>
			</div>
			<div class="comp-item delta">
				<div class="comp-label">DELTA</div>
				<div class="comp-value" style:color={deltaColor(delta)}>{deltaPct(r2Best, r3Best)}</div>
				<div class="comp-sub">First measurable gain</div>
			</div>
		</div>

		<!-- Station 1: What Changed -->
		<section class="station">
			<div class="station-label">STATION 1</div>
			<h2>What Changed</h2>
			<p>
				From the original 1,380 samples, we identified and removed two categories of
				problematic recordings:
			</p>
			<div class="removal-grid">
				<div class="removal-card">
					<div class="removal-num">32</div>
					<div class="removal-type">Clipped Samples</div>
					<div class="removal-detail">Peak amplitude exceeding 0.99</div>
				</div>
				<div class="removal-card">
					<div class="removal-num">7</div>
					<div class="removal-type">Near-Silent Samples</div>
					<div class="removal-detail">RMS below 0.005</div>
				</div>
			</div>
			<p>
				<strong>Clipped samples</strong> are recordings where the signal was too loud for
				the analog-to-digital converter. When a signal exceeds the maximum representable
				value, the waveform's peaks get
				<ConceptInline term="hard-clipped">
					{#snippet children()}
						Hard clipping occurs when a signal exceeds the maximum range of the recording
						system. Instead of following its natural curve, the waveform is flattened at
						the ceiling. This creates sharp discontinuities in the waveform that generate
						harmonics not present in the original signal. In a spectrogram, clipping
						manifests as bright horizontal lines across many frequency bins -- false
						spectral content that does not correspond to any real harmonic of the note.
						The classifier sees these artifacts as features and tries to learn from them,
						but since clipping varies randomly with playing dynamics, it adds noise rather
						than signal.
					{/snippet}
				</ConceptInline>
				-- flattened at the ceiling. This distorts the harmonic content, creating
				false spectral energy across frequency bins that does not correspond to any real
				harmonic of the note. The spectrogram becomes contaminated with artifacts the
				model tries to learn from but cannot generalize.
			</p>
			<p>
				<strong>Near-silent samples</strong> are recordings where the string was barely
				plucked or the capture triggered on ambient noise. With an RMS below 0.005,
				these spectrograms are mostly noise floor. They teach the model nothing useful
				about what a note at that position sounds like -- instead, the model is forced
				to memorize random noise patterns to classify them correctly, which hurts
				generalization.
			</p>
			<div class="viz-item">
				<div class="viz-label">Removed samples: clipped (left) and near-silent (right)</div>
				<img
					src="/training/round_03/removed_samples.png"
					alt="Visualization showing the 32 clipped samples with peaks exceeding 0.99 and 7 near-silent samples with RMS below 0.005 that were removed from the dataset"
					class="viz-img"
				/>
			</div>
			<p class="viz-caption">
				The 39 samples removed from the dataset. Clipped samples show distorted waveforms
				with flattened peaks. Silent samples show near-zero amplitude with mostly noise floor.
			</p>
		</section>

		<!-- Station 2: Why It Helped -->
		<section class="station">
			<div class="station-label">STATION 2</div>
			<h2>Why It Helped</h2>
			<p>
				This is the first round with measurable improvement across all three models.
				Round 2 (onset alignment) produced zero change. Here, removing just 39 samples
				out of 1,380 -- less than 3% of the data -- produced clear gains.
			</p>
			<ul class="findings">
				<li>
					<span class="finding-icon num">1</span>
					<div>
						<strong>Clipped samples had distorted harmonic content that confused the model.</strong>
						When a signal clips, the flat-topped waveform generates spurious harmonics
						across the frequency spectrum. The spectrogram shows bright energy at
						frequencies that have nothing to do with the fundamental pitch or its natural
						overtones. The model sees these artifacts and tries to incorporate them into
						its learned features, but since clipping is random and inconsistent, these
						features hurt rather than help classification.
					</div>
				</li>
				<li>
					<span class="finding-icon num">2</span>
					<div>
						<strong>Silent samples were pure noise the model had to memorize instead of learn from.</strong>
						A near-silent spectrogram is dominated by the noise floor of the recording
						chain. There are no meaningful harmonic patterns to extract. When these
						samples appear in training, the model must memorize their random noise
						patterns to classify them correctly, wasting model capacity on information
						that cannot generalize to real playing.
					</div>
				</li>
				<li>
					<span class="finding-icon num">3</span>
					<div>
						<strong>Random Forest benefited most (+1.5%) because flattened features are more sensitive to noise.</strong>
						The Random Forest operates on flattened spectrogram pixels -- every pixel
						is an independent feature with no spatial context. A few corrupted pixels
						from clipping artifacts can mislead individual decision trees. The CNNs,
						with their spatial convolution filters, are somewhat more robust to
						localized noise because they aggregate information from neighboring regions.
					</div>
				</li>
				<li>
					<span class="finding-icon num">4</span>
					<div>
						<strong>Data quality beats data quantity.</strong>
						Removing 39 bad samples helped more than aligning 1,380. This is a common
						pattern in machine learning: a small number of noisy or mislabeled samples
						can disproportionately drag down accuracy because the model wastes capacity
						trying to fit data that contradicts the true patterns.
					</div>
				</li>
			</ul>
		</section>

		<!-- Station 3: Results Comparison -->
		<section class="station">
			<div class="station-label">STATION 3</div>
			<h2>Results Comparison</h2>
			<p>
				All three rounds compared side-by-side. Select a model to see its
				string-by-string breakdown across all rounds.
			</p>

			<!-- Model selector tabs -->
			<div class="model-tabs">
				{#each r3Results as model, i}
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

			<!-- Three-round comparison table -->
			<div class="model-compare">
				<div class="mc-header">
					<span class="mc-col"></span>
					<span class="mc-col">Round 1</span>
					<span class="mc-col">Round 2</span>
					<span class="mc-col">Round 3</span>
					<span class="mc-col">R1 to R3</span>
				</div>
				<div class="mc-row overall">
					<span class="mc-col mc-label">Overall</span>
					<span class="mc-col">{pct(r1Results[selectedModel].accuracy)}</span>
					<span class="mc-col">{pct(r2Results[selectedModel].accuracy)}</span>
					<span class="mc-col">{pct(r3Results[selectedModel].accuracy)}</span>
					<span class="mc-col" style:color={deltaColor(r3Results[selectedModel].accuracy - r1Results[selectedModel].accuracy)}>
						{deltaPct(r1Results[selectedModel].accuracy, r3Results[selectedModel].accuracy)}
					</span>
				</div>
				{#each stringNames as name, i}
					{@const d = r3Results[selectedModel].per_string[i] - r1Results[selectedModel].per_string[i]}
					<div class="mc-row">
						<span class="mc-col mc-label">{name}</span>
						<span class="mc-col">{pct(r1Results[selectedModel].per_string[i])}</span>
						<span class="mc-col">{pct(r2Results[selectedModel].per_string[i])}</span>
						<span class="mc-col">{pct(r3Results[selectedModel].per_string[i])}</span>
						<span class="mc-col" style:color={deltaColor(d)}>{deltaPct(r1Results[selectedModel].per_string[i], r3Results[selectedModel].per_string[i])}</span>
					</div>
				{/each}
			</div>

			<!-- Per-string bars for Pure CNN Round 3 -->
			<div class="string-bars">
				<div class="bars-title">Per-String Accuracy -- {r3Results[selectedModel].name} (Round 3)</div>
				{#each stringNames as name, i}
					{@const acc = r3Results[selectedModel].per_string[i]}
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

			<!-- Side-by-side accuracy chart -->
			<div class="viz-item">
				<div class="viz-label">Round 1 vs Round 2 vs Round 3 accuracy across all models</div>
				<img
					src="/training/round_03/comparison_bars.png"
					alt="Bar chart comparing Round 1, Round 2, and Round 3 accuracy for Random Forest, Hybrid CNN, and Pure CNN. Round 3 shows improvement across all models."
					class="viz-img"
				/>
			</div>
		</section>

		<!-- Station 4: What We Learned -->
		<section class="station">
			<div class="station-label">STATION 4</div>
			<h2>What We Learned</h2>
			<ul class="learnings">
				<li>
					<strong>Quality matters more than alignment.</strong>
					Round 2 aligned all 1,380 samples to their pluck onset with zero accuracy
					impact. Round 3 removed just 39 samples and every model improved. The
					32 clipped samples were actively harmful -- their distorted harmonics were
					polluting the learned features.
				</li>
				<li>
					<strong>A small number of bad samples can disproportionately hurt accuracy.</strong>
					39 samples is 2.8% of the dataset. Removing them gave the Pure CNN a +0.9%
					boost and the Random Forest +1.5%. These samples were not just unhelpful --
					they were teaching the model incorrect patterns.
				</li>
				<li>
					<strong>Feature-level models are more sensitive to data quality.</strong>
					The Random Forest, which operates on flattened pixel values with no spatial
					context, gained the most from cleanup. The CNNs, with their spatial pooling,
					were partially robust to the noise but still benefited.
				</li>
				<li>
					<strong>Next: harmonic features for string identification.</strong>
					With quality issues resolved, the next frontier is the E2/A2 confusion.
					These strings share overlapping fundamental frequencies at certain fret
					positions. Physics-based harmonic ratio features (via Goertzel filters)
					should help distinguish them by their overtone structure rather than relying
					solely on the spectrogram.
				</li>
			</ul>
			<div class="nav-links">
				<a href="/diary/machine-learning/round-2" class="nav-link prev">
					&lt;- Round 2: Onset Alignment
				</a>
				<a href="/diary/machine-learning/round-4" class="nav-link next">
					Round 4: Goertzel Harmonics ->
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
		font-family: var(--font-ui);
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
		flex-shrink: 0;
		display: flex;
		align-items: center;
	}
	.comp-label {
		font-family: var(--font-ui);
		font-size: var(--font-size-xs);
		color: var(--color-accent-magenta);
		letter-spacing: 2px;
		margin-bottom: 4px;
	}
	.comp-value {
		font-family: var(--font-code);
		font-size: 22px;
		color: var(--color-accent-cyan);
	}
	.comp-sub {
		font-size: 10px;
		color: var(--color-text-dim);
		margin-top: 4px;
	}

	/* Removal grid */
	.removal-grid {
		display: flex;
		gap: 12px;
		margin: 16px 0;
	}
	.removal-card {
		flex: 1;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 16px;
		text-align: center;
	}
	.removal-num {
		font-family: var(--font-code);
		font-size: 28px;
		color: var(--color-accent-magenta);
	}
	.removal-type {
		font-family: var(--font-reading);
		font-size: 13px;
		color: var(--color-text-primary);
		font-weight: 600;
		margin-top: 4px;
	}
	.removal-detail {
		font-size: 11px;
		color: var(--color-text-dim);
		margin-top: 4px;
	}

	/* Stations */
	.station {
		padding: 24px;
		border-bottom: 1px solid var(--color-border);
	}
	.station-label {
		font-family: var(--font-ui);
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
	.station strong {
		color: var(--color-text-primary);
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
		font-family: var(--font-code);
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
		font-family: var(--font-ui);
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
		font-family: var(--font-code);
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
		font-family: var(--font-ui);
		font-size: 9px;
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
		font-family: var(--font-ui);
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

	/* Learnings list */
	.learnings {
		list-style: none;
		padding: 0;
		margin: 0 0 24px 0;
	}
	.learnings li {
		padding: 10px 0;
		border-bottom: 1px solid var(--color-border);
		font-size: 13px;
		color: var(--color-text-secondary);
		line-height: 1.7;
	}
	.learnings li:last-child {
		border-bottom: none;
	}
	.learnings strong {
		color: var(--color-text-primary);
		display: block;
		margin-bottom: 4px;
	}

	/* Navigation links */
	.nav-links {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		margin-top: 16px;
	}
	.nav-link {
		font-family: var(--font-ui);
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
