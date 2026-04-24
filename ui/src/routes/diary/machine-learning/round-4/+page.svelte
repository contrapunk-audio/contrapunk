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
	let r4Results: ModelResult[] = $state([]);
	let loading = $state(true);
	let selectedModel = $state(2); // Pure CNN by default

	const stringNames = ['E2 (low)', 'A2', 'D3', 'G3', 'B3', 'E4 (high)'];

	$effect(() => {
		Promise.all([
			fetch('/training/round_01/results.json').then(r => r.json()),
			fetch('/training/round_02/results.json').then(r => r.json()),
			fetch('/training/round_03/results.json').then(r => r.json()),
			fetch('/training/round_04/results.json').then(r => r.json()),
		]).then(([r1, r2, r3, r4]) => {
			r1Results = r1;
			r2Results = r2;
			r3Results = r3;
			r4Results = r4;
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
		{ label: 'Round 4' },
	];
</script>

<svelte:head>
	<title>Round 4: Goertzel Harmonics - Contrapunk Diary</title>
</svelte:head>

<DiaryNav {crumbs} />

<div class="round-page">
	<!-- Hero -->
	<header class="hero">
		<div class="round-label">ROUND 04</div>
		<h1>Goertzel Harmonics</h1>
		<p class="hero-sub">
			Each guitar string has a unique harmonic fingerprint -- the ratio of overtone
			amplitudes relative to the fundamental. We extracted these using the Goertzel
			algorithm and fused them with the spectrogram in a two-branch neural network.
			The result: harmonic features help weaker models on hard strings, but the
			Pure CNN already captures this information from the spectrogram alone.
		</p>
	</header>

	{#if loading}
		<p class="loading-text">Loading results...</p>
	{:else if r1Results.length && r3Results.length && r4Results.length}
		{@const r3Best = bestAccuracy(r3Results)}
		{@const r4Best = bestAccuracy(r4Results)}
		{@const delta = r4Best - r3Best}

		<StatBar stats={[
			{ value: pct(r4Best), label: 'Best Accuracy', color: 'var(--color-accent-cyan)' },
			{ value: deltaPct(r3Best, r4Best), label: 'vs Round 3', color: deltaColor(delta) },
			{ value: '11', label: 'Goertzel Features', color: 'var(--color-accent-teal)' },
			{ value: '+5.2%', label: 'Hybrid CNN E2', color: 'var(--color-accent-teal)' },
		]} />

		<!-- Comparison Banner -->
		<div class="comparison-banner">
			<div class="comp-item">
				<div class="comp-label">ROUND 3</div>
				<div class="comp-value">{pct(r3Best)}</div>
				<div class="comp-sub">Quality-cleaned</div>
			</div>
			<div class="comp-arrow">
				<svg width="24" height="16" viewBox="0 0 24 16" fill="none">
					<path d="M0 8h20M14 2l6 6-6 6" stroke="var(--color-text-dim)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
				</svg>
			</div>
			<div class="comp-item">
				<div class="comp-label">ROUND 4</div>
				<div class="comp-value">{pct(r4Best)}</div>
				<div class="comp-sub">Goertzel harmonics</div>
			</div>
			<div class="comp-item delta">
				<div class="comp-label">DELTA</div>
				<div class="comp-value" style:color={deltaColor(delta)}>{deltaPct(r3Best, r4Best)}</div>
				<div class="comp-sub">Best model unchanged</div>
			</div>
		</div>

		<!-- Station 1: What Changed -->
		<section class="station">
			<div class="station-label">STATION 1</div>
			<h2>What Changed</h2>
			<p>
				The mel-spectrogram captures pitch (which fret) well but struggles with
				string identity for overlapping pitches. Two different string-fret
				combinations can produce the same fundamental frequency -- for example,
				E2 fret 5 and A2 open are both 110 Hz. What distinguishes them is their
				<ConceptInline term="harmonic fingerprint">
					{#snippet children()}
						Every vibrating string produces not just the fundamental frequency but
						also a series of overtones at integer multiples (2x, 3x, 4x...). The
						relative amplitudes of these overtones -- the harmonic ratios -- are
						determined by the string's physical properties: wound vs plain, thick
						vs thin, steel vs nylon. A wound E2 string produces different harmonic
						ratios than a plain B3 string even when playing the same pitch. This
						is what gives each string its characteristic timbre and what allows a
						trained ear (or a classifier) to tell them apart.
					{/snippet}
				</ConceptInline>
				-- the unique pattern of overtone amplitudes determined by the string's
				physical properties.
			</p>
			<p>
				We used the
				<ConceptInline term="Goertzel algorithm">
					{#snippet children()}
						The Goertzel algorithm computes the energy at a single specific
						frequency, unlike the FFT which computes energy at all frequencies.
						When you only need a handful of frequency bins (like the first 10
						harmonics of a known fundamental), Goertzel is more efficient than a
						full FFT. For each sample, we compute the fundamental frequency from
						the fret label, then use Goertzel filters at the first 10 harmonic
						frequencies (1x, 2x, ... 10x the fundamental) to extract their
						amplitudes. This gives us 9 harmonic ratios (H2/H1 through H10/H1),
						plus spectral centroid and inharmonicity -- 11 features total.
					{/snippet}
				</ConceptInline>
				to extract 11 physics-based features from each sample:
			</p>
			<div class="feature-grid">
				<div class="feature-card">
					<div class="feature-num">9</div>
					<div class="feature-type">Harmonic Ratios</div>
					<div class="feature-detail">H2/H1 through H10/H1</div>
				</div>
				<div class="feature-card">
					<div class="feature-num">1</div>
					<div class="feature-type">Spectral Centroid</div>
					<div class="feature-detail">Brightness measure</div>
				</div>
				<div class="feature-card">
					<div class="feature-num">1</div>
					<div class="feature-type">Inharmonicity</div>
					<div class="feature-detail">Overtone stretch factor</div>
				</div>
			</div>
			<p>
				These 11 features were fused with the spectrogram: concatenated as extra
				columns for the Random Forest, or fed as a second input branch for the
				CNN architectures (two-branch fusion network).
			</p>
		</section>

		<!-- Station 2: The Physics -->
		<section class="station">
			<div class="station-label">STATION 2</div>
			<h2>The Physics</h2>
			<p>
				Not all strings are created equal. The Goertzel analysis revealed that
				each string has a distinct harmonic profile:
			</p>
			<ul class="findings">
				<li>
					<span class="finding-icon num">1</span>
					<div>
						<strong>G3 has the highest H2 ratio (0.76).</strong>
						The G3 string on a standard acoustic guitar is often the thickest
						unwound (plain) string. Its second harmonic is nearly as strong as
						the fundamental, giving it that characteristic bright, ringing quality
						that makes it easy to classify.
					</div>
				</li>
				<li>
					<span class="finding-icon num">2</span>
					<div>
						<strong>E2 and G3 have the highest inharmonicity.</strong>
						Real strings are not ideal -- their overtones are slightly sharp
						relative to perfect integer multiples. This
						<ConceptInline term="inharmonicity">
							{#snippet children()}
								Inharmonicity is the deviation of a string's overtones from perfect
								harmonic ratios. An ideal string would have overtones at exactly 2x,
								3x, 4x the fundamental. Real strings, due to stiffness, produce
								overtones that are slightly sharper (higher) than these ideal values.
								Thicker, stiffer strings (like the wound E2) have more inharmonicity
								than thin, flexible strings (like the plain E4). This measurable
								physical property is a strong cue for string identification.
							{/snippet}
						</ConceptInline>
						is most pronounced on thicker strings and provides an additional
						dimension for distinguishing strings with overlapping fundamentals.
					</div>
				</li>
				<li>
					<span class="finding-icon num">3</span>
					<div>
						<strong>Higher strings (B3, E4) have cleaner harmonic series.</strong>
						Their overtones align closely with integer multiples, making them
						easier to classify from the spectrogram alone. This explains why
						the Pure CNN already handles them well without extra features.
					</div>
				</li>
			</ul>
			<div class="viz-item">
				<div class="viz-label">Round 4 per-string accuracy deltas vs Round 1</div>
				<img
					src="/training/round_04/round_04_delta.png"
					alt="Bar chart showing per-string accuracy changes from Round 1 to Round 4. Hybrid CNN E2 shows the largest improvement at +5.2%."
					class="viz-img"
				/>
			</div>
			<p class="viz-caption">
				The delta chart shows where harmonic features had the most impact. The Hybrid CNN's
				E2 string jumped +5.2%, while the Pure CNN's E2 stayed flat -- it already extracts
				this information from the spectrogram.
			</p>
		</section>

		<!-- Station 3: Results Comparison -->
		<section class="station">
			<div class="station-label">STATION 3</div>
			<h2>Results Comparison</h2>
			<p>
				All four rounds compared side-by-side. Select a model to see its
				string-by-string breakdown across all rounds.
			</p>

			<!-- Model selector tabs -->
			<div class="model-tabs">
				{#each r4Results as model, i}
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

			<!-- Four-round comparison table -->
			<div class="model-compare">
				<div class="mc-header">
					<span class="mc-col"></span>
					<span class="mc-col">Round 1</span>
					<span class="mc-col">Round 2</span>
					<span class="mc-col">Round 3</span>
					<span class="mc-col">Round 4</span>
					<span class="mc-col">R1 to R4</span>
				</div>
				<div class="mc-row overall">
					<span class="mc-col mc-label">Overall</span>
					<span class="mc-col">{pct(r1Results[selectedModel].accuracy)}</span>
					<span class="mc-col">{pct(r2Results[selectedModel].accuracy)}</span>
					<span class="mc-col">{pct(r3Results[selectedModel].accuracy)}</span>
					<span class="mc-col">{pct(r4Results[selectedModel].accuracy)}</span>
					<span class="mc-col" style:color={deltaColor(r4Results[selectedModel].accuracy - r1Results[selectedModel].accuracy)}>
						{deltaPct(r1Results[selectedModel].accuracy, r4Results[selectedModel].accuracy)}
					</span>
				</div>
				{#each stringNames as name, i}
					{@const d = r4Results[selectedModel].per_string[i] - r1Results[selectedModel].per_string[i]}
					<div class="mc-row">
						<span class="mc-col mc-label">{name}</span>
						<span class="mc-col">{pct(r1Results[selectedModel].per_string[i])}</span>
						<span class="mc-col">{pct(r2Results[selectedModel].per_string[i])}</span>
						<span class="mc-col">{pct(r3Results[selectedModel].per_string[i])}</span>
						<span class="mc-col">{pct(r4Results[selectedModel].per_string[i])}</span>
						<span class="mc-col" style:color={deltaColor(d)}>{deltaPct(r1Results[selectedModel].per_string[i], r4Results[selectedModel].per_string[i])}</span>
					</div>
				{/each}
			</div>

			<!-- Per-string bars for Round 4 -->
			<div class="string-bars">
				<div class="bars-title">Per-String Accuracy -- {r4Results[selectedModel].name} (Round 4)</div>
				{#each stringNames as name, i}
					{@const acc = r4Results[selectedModel].per_string[i]}
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

			<div class="viz-item">
				<div class="viz-label">Round 1 vs Round 4 accuracy across all models</div>
				<img
					src="/training/round_04/round_04_comparison.png"
					alt="Bar chart comparing Round 1 and Round 4 accuracy for Random Forest, Hybrid CNN, and Pure CNN."
					class="viz-img"
				/>
			</div>

			<div class="table-note">
				The key finding: Hybrid CNN E2 jumped from 81.3% to 86.5% (+5.2%) with
				harmonic features, but Pure CNN E2 stayed at 94.3% -- it was already
				extracting this information from the raw spectrogram. Harmonic features
				help weaker models catch up but do not push the frontier.
			</div>
		</section>

		<!-- Station 4: What We Learned -->
		<section class="station">
			<div class="station-label">STATION 4</div>
			<h2>What We Learned</h2>
			<ul class="learnings">
				<li>
					<strong>Harmonic features help weaker models on hard strings.</strong>
					The Hybrid CNN's E2 accuracy jumped +5.2% (81.3% to 86.5%) with Goertzel
					features. The Random Forest's E2 gained +2.2%. These models lacked the
					depth to extract harmonic ratios from the spectrogram on their own, so
					providing them explicitly helped significantly.
				</li>
				<li>
					<strong>The Pure CNN already captures harmonic information.</strong>
					With 4 convolutional layers and enough depth, the Pure CNN learns to
					extract harmonic ratio patterns directly from the spectrogram's vertical
					frequency structure. Adding explicit harmonic features gave it nothing new
					to work with, resulting in a negligible +0.2% overall change.
				</li>
				<li>
					<strong>The ceiling is not about features -- it is about data quantity.</strong>
					With only 10 samples per class, even a model that perfectly extracts all
					available features is limited by the statistical reliability of its
					training signal. More features cannot compensate for thin data.
				</li>
				<li>
					<strong>Next: data augmentation to increase effective training size.</strong>
					If the limit is data quantity rather than feature coverage, the next
					logical step is to generate augmented copies of the training data --
					gain variation, noise injection, time shift -- to give the models more
					examples to learn from without capturing new recordings.
				</li>
			</ul>
			<div class="nav-links">
				<a href="/diary/machine-learning/round-3" class="nav-link prev">
					&lt;- Round 3: Quality Cleanup
				</a>
				<a href="/diary/machine-learning/round-5" class="nav-link next">
					Round 5: Data Augmentation ->
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

	/* Feature grid */
	.feature-grid {
		display: flex;
		gap: 12px;
		margin: 16px 0;
	}
	.feature-card {
		flex: 1;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 16px;
		text-align: center;
	}
	.feature-num {
		font-family: var(--font-code);
		font-size: 28px;
		color: var(--color-accent-cyan);
	}
	.feature-type {
		font-family: var(--font-reading);
		font-size: 13px;
		color: var(--color-text-primary);
		font-weight: 600;
		margin-top: 4px;
	}
	.feature-detail {
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
