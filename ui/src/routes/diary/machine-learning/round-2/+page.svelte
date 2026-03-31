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

	function worstStringName(results: ModelResult[], idx: number): string {
		if (!results.length || !results[idx]) return '';
		const perString = results[idx].per_string;
		const minIdx = perString.indexOf(Math.min(...perString));
		return stringNames[minIdx];
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
			Hypothesis: every audio sample has some silence before the pluck. If we detect
			the exact moment the string starts vibrating and trim everything before it,
			the spectrogram should be cleaner and the classifier should perform better.
			Result: zero measurable change. Here is why that is actually informative.
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
			<div class="comp-arrow">
				<svg width="24" height="16" viewBox="0 0 24 16" fill="none">
					<path d="M0 8h20M14 2l6 6-6 6" stroke="var(--color-text-dim)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
				</svg>
			</div>
			<div class="comp-item">
				<div class="comp-label">ROUND 2</div>
				<div class="comp-value">{pct(r2Best)}</div>
				<div class="comp-sub">Onset-aligned</div>
			</div>
			<div class="comp-item delta">
				<div class="comp-label">DELTA</div>
				<div class="comp-value" style:color={deltaColor(delta)}>{deltaPct(r1Best, r2Best)}</div>
				<div class="comp-sub">No measurable change</div>
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
				Every sample starts with some silence before the pluck. Onset detection
				finds the exact moment the string starts vibrating and trims everything
				before it. The idea is simple: if every spectrogram starts at the attack
				transient instead of at a random amount of pre-silence, the patterns should
				be more consistent and easier for the model to learn.
			</p>
			<p>
				The process for each of the 1,380 samples:
			</p>
			<ol class="process-steps">
				<li>Compute short-time energy in sliding 5ms windows across the raw audio</li>
				<li>Find the first frame where energy exceeds 10% of the sample's peak energy</li>
				<li>Trim everything before that point (with a small 2ms safety margin)</li>
				<li>Re-extract the mel-spectrogram from the trimmed audio</li>
				<li>Retrain all three models on the new spectrograms</li>
			</ol>
			<p>
				This is an
				<ConceptInline term="onset detection algorithm">
					{#snippet children()}
						Onset detection identifies the precise moment a note begins by tracking
						how rapidly the signal's energy increases. We use energy-based onset
						detection: compute the short-time energy in sliding windows, then find
						the point where energy rises sharply above a threshold. The threshold
						is set relative to each sample's peak energy, making it adaptive to
						different playing dynamics. Once the onset is found, we trim everything
						before it and re-extract the mel-spectrogram from the aligned audio.
					{/snippet}
				</ConceptInline>
				-- the same technique that DAWs use for snap-to-transient editing, applied
				automatically to every sample in the dataset.
			</p>
		</section>

		<!-- Station 2: Onset Distribution -->
		<section class="station">
			<div class="station-label">STATION 2</div>
			<h2>Where Were the Onsets?</h2>
			<p>
				Before looking at model accuracy, the first question is: how much silence
				was there to trim? If the capture tool was already triggering close to the
				pluck, there would be very little misalignment to fix.
			</p>
			<div class="viz-item">
				<div class="viz-label">Detected onset position across all 1,380 samples</div>
				<img
					src="/training/round_02/onset_distribution.png"
					alt="Histogram showing detected onset times clustered tightly around 20ms, with most samples between 15ms and 30ms"
					class="viz-img"
				/>
			</div>
			<p class="viz-caption">
				Median onset was 21.8ms from the start of recording. The tight clustering
				shows that the capture tool was already triggering within ~20ms of the pluck --
				most samples had very little pre-silence to trim.
			</p>
			<div class="insight-box">
				<div class="insight-label">WHAT THIS MEANS</div>
				<p>
					At 48kHz with a hop length of 256, 21.8ms corresponds to roughly 4 spectrogram
					columns. That is a tiny temporal shift -- barely visible when comparing
					spectrograms side by side. The capture tool was already doing a good job of
					starting close to the pluck.
				</p>
			</div>
		</section>

		<!-- Station 3: Before/After Spectrograms -->
		<section class="station">
			<div class="station-label">STATION 3</div>
			<h2>Before and After</h2>
			<p>
				The spectrogram pairs below show the same sample before and after onset
				alignment. Look at the left edge: in the "before" version there is a narrow
				band of silence (dark area), and in the "after" version the harmonic content
				starts immediately. The difference is subtle because there was not much
				silence to remove.
			</p>
			<div class="viz-item">
				<div class="viz-label">Raw vs onset-aligned spectrograms (6 sample pairs)</div>
				<img
					src="/training/round_02/before_after_spectrograms.png"
					alt="Six pairs of spectrograms showing raw audio on the left and onset-aligned audio on the right. The aligned versions start slightly earlier with harmonic content but are otherwise nearly identical."
					class="viz-img"
				/>
			</div>
			<p class="viz-caption">
				Each pair shows the same note. Top row: raw spectrogram as captured. Bottom
				row: after trimming to the detected onset. The spectral content is identical --
				only the start position shifts by a few columns.
			</p>
		</section>

		<!-- Station 4: Model-by-Model Comparison -->
		<section class="station">
			<div class="station-label">STATION 4</div>
			<h2>Per-Model Comparison</h2>
			<p>
				All three architectures were retrained on the onset-aligned spectrograms
				with the same hyperparameters as Round 1. Select a model to see the
				string-by-string breakdown.
			</p>

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

			<p class="table-note">
				E2 remains the weakest string across all models in both rounds: its per-string
				accuracy barely moved. The low E string's fundamental frequencies overlap with
				the A2 string's lower frets, creating confusion that onset alignment cannot
				resolve -- this is a spectral similarity problem, not a timing problem.
			</p>
		</section>

		<!-- Station 5: Accuracy Comparison Chart -->
		<section class="station">
			<div class="station-label">STATION 5</div>
			<h2>Side-by-Side Accuracy</h2>
			<p>
				A direct visual comparison of all three models across both rounds. The bars
				are nearly identical in height, confirming that onset alignment produced no
				meaningful accuracy shift for any architecture.
			</p>
			<div class="viz-item">
				<div class="viz-label">Round 1 vs Round 2 accuracy across all models</div>
				<img
					src="/training/round_02/comparison_bars.png"
					alt="Bar chart comparing Round 1 and Round 2 accuracy for Random Forest, Hybrid CNN, and Pure CNN. All pairs are nearly identical."
					class="viz-img"
				/>
			</div>
		</section>

		<!-- Station 6: Why Zero Impact -->
		<section class="station">
			<div class="station-label">STATION 6</div>
			<h2>Why Zero Impact</h2>
			<p>
				A null result is still a result. Three factors explain why onset alignment
				made no measurable difference:
			</p>
			<ul class="findings">
				<li>
					<span class="finding-icon num">1</span>
					<div>
						<strong>The capture tool already triggered within ~20ms of the pluck.</strong>
						That is less than 2 time frames in the spectrogram. There simply was not
						enough misalignment across samples for trimming to matter. If onset times
						had been spread across 50-200ms, this experiment would likely have shown
						a clear improvement.
					</div>
				</li>
				<li>
					<span class="finding-icon num">2</span>
					<div>
						<strong>CNNs with
						<ConceptInline term="pooling layers have built-in translation invariance">
							{#snippet children()}
								Max pooling and average pooling operations reduce spatial dimensions by
								summarizing local regions. A MaxPool(2x2) takes the maximum value from
								each 2x2 block. This makes the network partially invariant to small
								translations -- if a feature shifts by a few pixels, the pooled output
								remains similar. With 3-4 pooling layers stacked, the Pure CNN can
								tolerate the kind of small temporal shifts (~4 columns) that onset
								alignment was meant to fix. This is why CNNs are robust to slight
								positional variations without explicit alignment.
							{/snippet}
						</ConceptInline>.</strong>
						The Pure CNN has 3 MaxPool layers, reducing the 94-pixel time axis to roughly
						11 pixels. A 20ms shift at 48kHz with hop=256 is about 4 spectrogram columns --
						well within what pooling can absorb without losing discriminative features.
					</div>
				</li>
				<li>
					<span class="finding-icon num">3</span>
					<div>
						<strong>The discriminative features are not onset-dependent.</strong>
						String and fret identity comes from harmonic content -- which frequencies are
						present and their relative strengths. This information is encoded in the
						vertical structure of the spectrogram (frequency axis), not the horizontal
						position (time axis). The steady-state portion of the note carries the same
						harmonic signature whether the spectrogram starts at 0ms or 20ms.
					</div>
				</li>
			</ul>
			<div class="takeaway-box">
				<div class="takeaway-label">TAKEAWAY</div>
				<p>
					This result tells us: the bottleneck is not data alignment.
					The capture tool's built-in onset detection is already good enough, and the
					CNN's pooling layers handle the remaining variation. Time to look at the data
					itself -- are there clipped samples, near-silent recordings, or other quality
					issues dragging accuracy down?
				</p>
			</div>
		</section>

		<!-- Station 7: What We Learned -->
		<section class="station">
			<div class="station-label">STATION 7</div>
			<h2>What We Learned</h2>
			<ul class="learnings">
				<li>
					<strong>Onset alignment does not help when capture is already well-timed.</strong>
					The median 21.8ms pre-silence was too small to matter, and the tight distribution
					means there were no severe outliers pulling accuracy down.
				</li>
				<li>
					<strong>This validates the capture tool's quality.</strong>
					The threshold-based trigger in the recording pipeline is doing its job. We do not
					need to add an alignment step to the preprocessing pipeline.
				</li>
				<li>
					<strong>Not every preprocessing step helps.</strong>
					It is tempting to assume that "cleaner data = better accuracy" but this experiment
					shows that some cleaning has no effect because the data was already clean enough
					in that specific dimension.
				</li>
				<li>
					<strong>Next: clean up quality issues, add harmonic features.</strong>
					With alignment ruled out, the next round will remove the 32 clipped samples and
					7 near-silent recordings identified during data analysis, and explore whether
					harmonic ratio features can help with the E2/A2 confusion.
				</li>
			</ul>
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
		flex-shrink: 0;
		display: flex;
		align-items: center;
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

	/* Process steps */
	.process-steps {
		padding-left: 20px;
		margin: 0 0 16px 0;
	}
	.process-steps li {
		font-size: 13px;
		color: var(--color-text-secondary);
		line-height: 1.7;
		margin-bottom: 4px;
	}
	.process-steps li::marker {
		color: var(--color-accent-cyan);
		font-family: var(--font-pixel);
		font-size: 10px;
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

	/* Table note */
	.table-note {
		font-size: 13px;
		color: var(--color-text-dim);
		line-height: 1.7;
		margin-top: 8px;
		padding: 12px 16px;
		border-left: 2px solid var(--color-accent-amber);
		background: var(--color-bg-panel);
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
		font-family: var(--font-pixel);
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

	/* Insight box */
	.insight-box {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-accent-cyan-dim);
		border-left-width: 3px;
		border-left-color: var(--color-accent-cyan);
		padding: 16px;
		margin-top: 16px;
	}
	.insight-label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-cyan);
		letter-spacing: 2px;
		margin-bottom: 8px;
	}
	.insight-box p {
		font-size: 13px;
		color: var(--color-text-secondary);
		line-height: 1.7;
		margin: 0;
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
