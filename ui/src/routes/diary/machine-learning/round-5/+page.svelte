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
	let r5Results: ModelResult[] = $state([]);
	let loading = $state(true);
	let selectedModel = $state(2); // Pure CNN by default

	const stringNames = ['E2 (low)', 'A2', 'D3', 'G3', 'B3', 'E4 (high)'];

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
		{ label: 'Round 5' },
	];
</script>

<svelte:head>
	<title>Round 5: Data Augmentation - Contrapunk Diary</title>
</svelte:head>

<DiaryNav {crumbs} />

<div class="round-page">
	<!-- Hero -->
	<header class="hero">
		<div class="round-label">ROUND 05</div>
		<h1>Data Augmentation</h1>
		<p class="hero-sub">
			With only 10 samples per class, the models are starved for data. We generated
			3 augmented copies per sample -- gain variation, noise injection, time shift,
			time stretch -- growing the training set from 1,380 to 5,520 samples. The Hybrid
			CNN hit a new personal best (95.1%), but the Pure CNN stayed flat. Round 3's
			97.3% remains the overall best.
		</p>
	</header>

	{#if loading}
		<p class="loading-text">Loading results...</p>
	{:else if r1Results.length && r3Results.length && r5Results.length}
		{@const r3Best = bestAccuracy(r3Results)}
		{@const r5Best = bestAccuracy(r5Results)}
		{@const delta = r5Best - r3Best}
		{@const hybridR1 = r1Results[1].accuracy}
		{@const hybridR5 = r5Results[1].accuracy}

		<StatBar stats={[
			{ value: pct(r5Best), label: 'Best Accuracy', color: 'var(--color-accent-cyan)' },
			{ value: deltaPct(r3Best, r5Best), label: 'vs Round 3 (best)', color: deltaColor(delta) },
			{ value: '5,520', label: 'Training Samples', color: 'var(--color-accent-teal)' },
			{ value: deltaPct(hybridR1, hybridR5), label: 'Hybrid CNN vs R1', color: 'var(--color-accent-teal)' },
		]} />

		<!-- Comparison Banner -->
		<div class="comparison-banner">
			<div class="comp-item">
				<div class="comp-label">ROUND 3 (BEST)</div>
				<div class="comp-value">{pct(r3Best)}</div>
				<div class="comp-sub">Quality-cleaned</div>
			</div>
			<div class="comp-arrow">
				<svg width="24" height="16" viewBox="0 0 24 16" fill="none">
					<path d="M0 8h20M14 2l6 6-6 6" stroke="var(--color-text-dim)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
				</svg>
			</div>
			<div class="comp-item">
				<div class="comp-label">ROUND 5</div>
				<div class="comp-value">{pct(r5Best)}</div>
				<div class="comp-sub">Augmented (4x data)</div>
			</div>
			<div class="comp-item delta">
				<div class="comp-label">DELTA</div>
				<div class="comp-value" style:color={deltaColor(delta)}>{deltaPct(r3Best, r5Best)}</div>
				<div class="comp-sub">Did not beat Round 3</div>
			</div>
		</div>

		<!-- Station 1: What Changed -->
		<section class="station">
			<div class="station-label">STATION 1</div>
			<h2>What Changed</h2>
			<p>
				We generated 3 augmented copies of every training sample using four
				label-preserving transformations. Each copy receives 1-2 randomly
				chosen augmentations:
			</p>
			<div class="augmentation-grid">
				<div class="aug-card">
					<div class="aug-name">Gain Variation</div>
					<div class="aug-desc">Scale amplitude by a random factor between 0.7x and 1.3x</div>
					<div class="aug-why">Simulates different picking dynamics</div>
				</div>
				<div class="aug-card">
					<div class="aug-name">Noise Injection</div>
					<div class="aug-desc">Add Gaussian noise at 5% of signal RMS</div>
					<div class="aug-why">Simulates microphone/preamp noise</div>
				</div>
				<div class="aug-card">
					<div class="aug-name">Time Shift</div>
					<div class="aug-desc">Shift onset by +/-10ms (+/-480 samples at 48kHz)</div>
					<div class="aug-why">Simulates trigger timing variation</div>
				</div>
				<div class="aug-card">
					<div class="aug-name">Time Stretch</div>
					<div class="aug-desc">Pitch-invariant stretch/compress by 0.95x to 1.05x</div>
					<div class="aug-why">Simulates subtle tempo variation</div>
				</div>
			</div>
			<div class="dataset-summary">
				<div class="ds-item">
					<div class="ds-value">1,380</div>
					<div class="ds-label">Original samples</div>
				</div>
				<div class="ds-arrow">x4</div>
				<div class="ds-item">
					<div class="ds-value">5,520</div>
					<div class="ds-label">Total training pool</div>
				</div>
				<div class="ds-arrow">=</div>
				<div class="ds-item">
					<div class="ds-value">~40</div>
					<div class="ds-label">Samples per class</div>
				</div>
			</div>

			<div class="insight-box">
				<div class="insight-label">DELIBERATELY EXCLUDED</div>
				<p>
					<strong>Pitch shifting</strong> was deliberately excluded from the augmentation
					pipeline. Even a 0.5 semitone shift would make one fret position sound like
					another -- a fundamental frequency shift of ~3% is enough to cross fret
					boundaries. Similarly, <strong>large time stretch</strong> was avoided because
					it alters the harmonic content too much, and <strong>SpecAugment frequency
					masking</strong> was excluded because it can mask the exact harmonics that
					distinguish one string from another. These augmentations are standard in speech
					recognition but dangerous for a pitch-and-timbre classification task.
				</p>
			</div>
		</section>

		<!-- Station 2: Leakage Prevention -->
		<section class="station">
			<div class="station-label">STATION 2</div>
			<h2>Leakage Prevention</h2>
			<p>
				Data augmentation introduces a subtle but critical risk:
				<ConceptInline term="data leakage">
					{#snippet children()}
						Data leakage occurs when information from the validation or test set
						influences the training process. With augmentation, leakage happens if
						an augmented copy of a training sample ends up in the validation fold.
						The model has essentially "seen" that sample (in a slightly modified
						form) during training, so its validation accuracy is artificially
						inflated. The model appears to generalize but is actually memorizing.
						The fix is simple: augmented copies must always stay in the same
						cross-validation fold as their original sample.
					{/snippet}
				</ConceptInline>.
				If an augmented copy of a training sample leaks into the validation fold,
				the model has effectively seen that sample during training, and validation
				accuracy becomes meaningless.
			</p>
			<ul class="findings">
				<li>
					<span class="finding-icon num">1</span>
					<div>
						<strong>Augmented copies always stay in the same CV fold as their original.</strong>
						The 5-fold stratified split is performed on the original 1,380 samples first.
						Augmented copies are generated only for the training folds, never for
						validation. This guarantees zero leakage.
					</div>
				</li>
				<li>
					<span class="finding-icon num">2</span>
					<div>
						<strong>Validation is performed exclusively on original samples.</strong>
						Even though the training set grows to 5,520 samples, the validation set
						in each fold contains only unaugmented originals. This ensures we are
						measuring true generalization to real recordings, not the model's ability
						to handle synthetic variations.
					</div>
				</li>
				<li>
					<span class="finding-icon num">3</span>
					<div>
						<strong>Stratification preserves class balance across folds.</strong>
						Each of the 138 classes has approximately 10 original samples. Stratified
						splitting ensures each fold has proportional representation, so no class
						is over- or under-represented in validation.
					</div>
				</li>
			</ul>
		</section>

		<!-- Station 3: Augmentation Examples -->
		<section class="station">
			<div class="station-label">STATION 3</div>
			<h2>Augmentation Examples</h2>
			<p>
				The spectrograms below show original samples alongside their augmented copies.
				The augmentations are subtle by design -- just enough variation to help the
				model generalize without distorting the underlying harmonic content.
			</p>
			<div class="viz-item">
				<div class="viz-label">Original vs augmented spectrogram pairs</div>
				<img
					src="/training/round_05/augmentation_examples.png"
					alt="Spectrogram pairs showing original recordings alongside augmented copies with gain variation, noise injection, time shift, and time stretch applied."
					class="viz-img"
				/>
			</div>
			<p class="viz-caption">
				Each augmentation is barely visible in the spectrogram -- gain variation adjusts
				overall brightness, noise adds a faint haze, time shift moves the onset, and
				time stretch subtly widens or narrows the temporal structure.
			</p>
		</section>

		<!-- Station 4: Results Comparison -->
		<section class="station">
			<div class="station-label">STATION 4</div>
			<h2>Results Comparison</h2>
			<p>
				All five rounds compared side-by-side. Select a model to see its
				string-by-string breakdown across every round.
			</p>

			<!-- Model selector tabs -->
			<div class="model-tabs">
				{#each r5Results as model, i}
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

			<!-- Five-round comparison table -->
			<div class="model-compare">
				<div class="mc-header">
					<span class="mc-col"></span>
					<span class="mc-col">R1</span>
					<span class="mc-col">R2</span>
					<span class="mc-col">R3</span>
					<span class="mc-col">R4</span>
					<span class="mc-col">R5</span>
					<span class="mc-col">R1-R5</span>
				</div>
				<div class="mc-row overall">
					<span class="mc-col mc-label">Overall</span>
					<span class="mc-col">{pct(r1Results[selectedModel].accuracy)}</span>
					<span class="mc-col">{pct(r2Results[selectedModel].accuracy)}</span>
					<span class="mc-col">{pct(r3Results[selectedModel].accuracy)}</span>
					<span class="mc-col">{pct(r4Results[selectedModel].accuracy)}</span>
					<span class="mc-col">{pct(r5Results[selectedModel].accuracy)}</span>
					<span class="mc-col" style:color={deltaColor(r5Results[selectedModel].accuracy - r1Results[selectedModel].accuracy)}>
						{deltaPct(r1Results[selectedModel].accuracy, r5Results[selectedModel].accuracy)}
					</span>
				</div>
				{#each stringNames as name, i}
					{@const d = r5Results[selectedModel].per_string[i] - r1Results[selectedModel].per_string[i]}
					<div class="mc-row">
						<span class="mc-col mc-label">{name}</span>
						<span class="mc-col">{pct(r1Results[selectedModel].per_string[i])}</span>
						<span class="mc-col">{pct(r2Results[selectedModel].per_string[i])}</span>
						<span class="mc-col">{pct(r3Results[selectedModel].per_string[i])}</span>
						<span class="mc-col">{pct(r4Results[selectedModel].per_string[i])}</span>
						<span class="mc-col">{pct(r5Results[selectedModel].per_string[i])}</span>
						<span class="mc-col" style:color={deltaColor(d)}>{deltaPct(r1Results[selectedModel].per_string[i], r5Results[selectedModel].per_string[i])}</span>
					</div>
				{/each}
			</div>

			<!-- Per-string bars for Round 5 -->
			<div class="string-bars">
				<div class="bars-title">Per-String Accuracy -- {r5Results[selectedModel].name} (Round 5)</div>
				{#each stringNames as name, i}
					{@const acc = r5Results[selectedModel].per_string[i]}
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
				<div class="viz-label">Round 1 through Round 5 accuracy across all models</div>
				<img
					src="/training/round_05/round_05_comparison.png"
					alt="Bar chart comparing all five rounds of training accuracy for Random Forest, Hybrid CNN, and Pure CNN."
					class="viz-img"
				/>
			</div>

			<div class="table-note">
				Hybrid CNN jumped to 95.1% -- a new personal best for that architecture,
				+3.1% over its Round 1 baseline. But the Pure CNN stayed at 96.4%, identical
				to Round 4. Round 3's 97.3% (from quality cleanup alone) remains the overall
				best result across all rounds and all models.
			</div>
		</section>

		<!-- Station 5: Training Curves -->
		<section class="station">
			<div class="station-label">STATION 5</div>
			<h2>Training Curves</h2>
			<p>
				The training curves reveal how each CNN architecture responds to
				the 4x increase in training data. The Hybrid CNN shows clear benefit
				from augmentation with less overfitting, while the Pure CNN's curves
				barely change.
			</p>
			<div class="viz-item">
				<div class="viz-label">Hybrid CNN training curves (augmented)</div>
				<img
					src="/training/round_05/training_curves_hybrid_cnn.png"
					alt="Training and validation loss/accuracy curves for the Hybrid CNN with augmented data, showing improved generalization."
					class="viz-img"
				/>
			</div>
			<div class="viz-item">
				<div class="viz-label">Pure CNN training curves (augmented)</div>
				<img
					src="/training/round_05/training_curves_pure_cnn.png"
					alt="Training and validation loss/accuracy curves for the Pure CNN with augmented data, showing minimal change from previous rounds."
					class="viz-img"
				/>
			</div>
		</section>

		<!-- Station 6: What We Learned -->
		<section class="station">
			<div class="station-label">STATION 6</div>
			<h2>What We Learned</h2>
			<ul class="learnings">
				<li>
					<strong>Augmentation helps models with more parameters.</strong>
					The Hybrid CNN gained +3.1% from Round 1, its biggest single-round
					improvement. With more parameters to tune, it benefits most from seeing
					more training examples -- even synthetic ones. The lean Pure CNN, with
					fewer parameters and stronger inductive biases from its architecture,
					does not need the extra data.
				</li>
				<li>
					<strong>The Pure CNN's 96.4% did not budge.</strong>
					Four times the training data, zero improvement for the strongest model.
					This is a strong signal that the Pure CNN's accuracy ceiling is not
					data-bounded. It has already extracted everything it can from the
					spectrogram representation.
				</li>
				<li>
					<strong>Round 3's 97.3% remains the overall best.</strong>
					Quality cleanup (removing 39 bad samples) gave a bigger boost than
					either harmonic features or 4x augmentation. The best thing we did
					for accuracy was remove data, not add it.
				</li>
				<li>
					<strong>The limit is not data quantity -- it is something more fundamental.</strong>
					We have tried better features (Round 4: Goertzel harmonics) and more
					data (Round 5: augmentation). Neither broke through 97%. The remaining
					errors are concentrated on string pairs with overlapping fundamentals
					(E2/A2, D3/G3) where the mel-spectrogram representation itself may be
					the bottleneck.
				</li>
			</ul>
		</section>

		<!-- Station 7: The Question -->
		<section class="station">
			<div class="station-label">STATION 7</div>
			<h2>The Question</h2>
			<div class="question-box">
				<p>
					If more data and more features do not break through 97%, maybe we need
					a fundamentally different approach. The mel-spectrogram collapses the
					raw frequency spectrum into perceptual bands, which is great for human
					hearing but may blur the fine harmonic structure that distinguishes
					overlapping string-fret combinations.
				</p>
				<p>
					Five rounds of iterative improvement have taught us where the ceiling is
					and what does not move it. The next step is not another incremental change
					-- it is a re-examination of the core representation and classification
					strategy.
				</p>
			</div>
			<div class="nav-links">
				<a href="/diary/machine-learning/round-4" class="nav-link prev">
					&lt;- Round 4: Goertzel Harmonics
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

	/* Augmentation grid */
	.augmentation-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
		margin: 16px 0;
	}
	.aug-card {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 16px;
	}
	.aug-name {
		font-family: var(--font-ui);
		font-size: 11px;
		color: var(--color-accent-cyan);
		letter-spacing: 1px;
		margin-bottom: 6px;
	}
	.aug-desc {
		font-size: 13px;
		color: var(--color-text-primary);
		line-height: 1.5;
	}
	.aug-why {
		font-size: 11px;
		color: var(--color-text-dim);
		margin-top: 6px;
		font-style: italic;
	}

	/* Dataset summary */
	.dataset-summary {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0;
		margin: 20px 0;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 16px;
	}
	.ds-item {
		flex: 1;
		text-align: center;
	}
	.ds-value {
		font-family: var(--font-code);
		font-size: 22px;
		color: var(--color-accent-cyan);
	}
	.ds-label {
		font-size: 10px;
		color: var(--color-text-dim);
		margin-top: 4px;
	}
	.ds-arrow {
		font-family: var(--font-ui);
		font-size: 16px;
		color: var(--color-text-dim);
		padding: 0 16px;
		flex-shrink: 0;
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
		font-family: var(--font-ui);
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
	.insight-box strong {
		color: var(--color-text-primary);
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

	/* Question box */
	.question-box {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-accent-magenta);
		border-left-width: 3px;
		padding: 20px;
		margin-bottom: 24px;
	}
	.question-box p {
		font-size: 14px;
		color: var(--color-text-secondary);
		line-height: 1.8;
		margin: 0 0 12px 0;
	}
	.question-box p:last-child {
		margin-bottom: 0;
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
