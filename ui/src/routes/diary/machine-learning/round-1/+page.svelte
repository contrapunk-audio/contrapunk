<script lang="ts">
	import DiaryNav from '$lib/components/diary/DiaryNav.svelte';
	import StatBar from '$lib/components/diary/StatBar.svelte';
	import ConceptInline from '$lib/components/diary/ConceptInline.svelte';
	import AudioComparison from '$lib/components/diary/AudioComparison.svelte';
	import AudioPlayer from '$lib/components/diary/AudioPlayer.svelte';
	import ModelDiagram from '$lib/components/diary/ModelDiagram.svelte';
	import SpectrogramViewer from '$lib/components/diary/SpectrogramViewer.svelte';
	import SpectrogramComparison from '$lib/components/diary/SpectrogramComparison.svelte';
	import PerStringBars from '$lib/components/diary/PerStringBars.svelte';
	import FretHeatmap from '$lib/components/diary/FretHeatmap.svelte';
	import StringConfusion from '$lib/components/diary/StringConfusion.svelte';

	type SpectrogramJson = {
		label: string;
		spectrogram: { data: number[][] };
	};

	let overviewSpecData: number[][] | null = $state(null);

	$effect(() => {
		fetch('/spectrograms/showcase/E2_open.json')
			.then(r => r.json() as Promise<SpectrogramJson>)
			.then(d => { overviewSpecData = d.spectrogram.data; })
			.catch(() => { /* silent */ });
	});

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

	// Maps selectedModel index to JSON keys in fret_accuracy.json / confusion_data.json
	const modelKeys = ['random_forest', 'hybrid_cnn', 'pure_cnn'];

	type FretAccuracyData = {
		models: Record<string, { grid: number[][]; counts: number[][] }>;
	};

	type ConfusionData = {
		models: Record<string, { string_confusion: number[][] }>;
	};

	let fretData = $state<FretAccuracyData | null>(null);
	let confusionData = $state<ConfusionData | null>(null);

	let currentFretGrid: number[][] = $derived(
		fretData?.models[modelKeys[selectedModel]]?.grid ?? []
	);
	let currentFretCounts: number[][] = $derived(
		fretData?.models[modelKeys[selectedModel]]?.counts ?? []
	);
	let currentStringConfusion: number[][] = $derived(
		confusionData?.models[modelKeys[selectedModel]]?.string_confusion ?? []
	);

	$effect(() => {
		fetch('/training/round_01/results.json')
			.then(r => r.json())
			.then(data => {
				results = data;
				loading = false;
			})
			.catch(() => { loading = false; });
	});

	$effect(() => {
		fetch('/training/round_01/fret_accuracy.json')
			.then(r => r.json())
			.then(data => { fretData = data; })
			.catch(() => { /* silent */ });
	});

	$effect(() => {
		fetch('/training/round_01/confusion_data.json')
			.then(r => r.json())
			.then(data => { confusionData = data; })
			.catch(() => { /* silent */ });
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

			{#if overviewSpecData}
				<div class="overview-spectrogram">
					<SpectrogramViewer
						data={overviewSpecData}
						label="What the model sees: a mel-spectrogram"
						height={200}
					/>
				</div>
			{/if}

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

			<!-- Hear the Data: A/B comparison -->
			<div class="hear-section">
				<div class="hear-heading">Hear the Data</div>
				<p class="hear-desc">
					The same note -- A2 (110 Hz) -- played on two different strings.
					The classifier must tell them apart from the spectrogram alone.
				</p>
				<AudioComparison
					urlA="/samples/confused_pairs/A2_on_E_string_fret5.wav"
					labelA="A2 on E string (fret 5)"
					urlB="/samples/confused_pairs/A2_on_A_string_open.wav"
					labelB="A2 on A string (open)"
					question="Can you hear the difference?"
				/>
			</div>

			<!-- See the Data: spectrogram comparison -->
			<div class="hear-section">
				<div class="hear-heading">See the Data</div>
				<p class="hear-desc">
					The same A2 pair as above, but rendered as mel-spectrograms -- the 64x94
					images the model actually sees. Notice how the harmonic patterns differ
					even though both are the same pitch.
				</p>
				<SpectrogramComparison
					urlA="/spectrograms/confused_pairs/A2_E2_string_fret5.json"
					urlB="/spectrograms/confused_pairs/A2_A2_string_open.json"
					labelA="A2 on E string (fret 5)"
					labelB="A2 on A string (open)"
					audioUrlA="/samples/confused_pairs/A2_on_E_string_fret5.wav"
					audioUrlB="/samples/confused_pairs/A2_on_A_string_open.wav"
					question="Same pitch, different harmonic fingerprints"
				/>
			</div>

			<!-- Showcase samples -->
			<div class="hear-section">
				<div class="hear-heading">Sample Sounds</div>
				<p class="hear-desc">
					A few recordings from across the guitar neck. Each is a single 0.5s pluck
					captured DI through the Audient iD14.
				</p>
				<div class="sample-grid">
					<AudioPlayer url="/samples/showcase/E2_open.wav" label="E2 open string (low E)" />
					<AudioPlayer url="/samples/showcase/A2_open.wav" label="A2 open string" />
					<AudioPlayer url="/samples/showcase/E4_open.wav" label="E4 open string (high E)" />
					<AudioPlayer url="/samples/showcase/G3_fret12.wav" label="G3 fret 12 (harmonic region)" />
				</div>
			</div>
		</section>

		<!-- Station 2: The Models -->
		<section class="station">
			<div class="station-label">STATION 2</div>
			<h2>How the Models Work</h2>
			<p>
				Three architectures, each taking the same mel-spectrogram input. The question:
				does the architecture matter more than the data at this scale? Select a model
				below to explore its architecture, reasoning, and data flow.
			</p>

			<!-- Model selector tabs -->
			<div class="model-selector-tabs">
				{#each results as model, i}
					<button
						class="model-selector-tab"
						class:active={selectedModel === i}
						onclick={() => selectedModel = i}
					>
						<span class="selector-name">{model.name}</span>
						<span class="selector-stats">
							<span class="selector-acc" style:color={barColor(model.accuracy)}>{pct(model.accuracy)}</span>
							{#if i === 0}
								<span class="selector-meta">200 trees | 6,016 features</span>
							{:else if i === 1}
								<span class="selector-meta">~265K params</span>
							{:else}
								<span class="selector-meta">~35K classifier params</span>
							{/if}
						</span>
						{#if model.train_time}
							<span class="selector-time">{model.train_time.toFixed(1)}s train</span>
						{/if}
					</button>
				{/each}
			</div>

			<!-- Random Forest detail -->
			{#if selectedModel === 0}
				<div class="model-detail">
					<div class="model-detail-section">
						<div class="model-detail-label">WHY WE CHOSE IT</div>
						<p>
							The reliable baseline. Random Forests handle small datasets well, are fast to train,
							and tell us if the features themselves are informative. If RF gets 90%+, we know the
							mel-spectrogram contains enough information to distinguish fret positions -- and we
							do not need a neural network just to prove the signal exists.
						</p>
					</div>

					<div class="model-detail-section">
						<div class="model-detail-label">HOW IT WORKS</div>
						<ModelDiagram model="random-forest" />
						<div class="step-list">
							<div class="step">
								<span class="step-num">1</span>
								<div>
									<strong>Audio to image.</strong> Each 0.5-second recording (24,000 raw samples at
									48kHz) is converted into a
									<ConceptInline term="mel-spectrogram">
										{#snippet children()}
											A mel-spectrogram converts raw audio into a 2D "image" where the x-axis
											is time, y-axis is frequency (on a perceptual scale), and brightness is
											loudness. The mel scale matches human hearing -- we perceive the difference
											between 100Hz and 200Hz the same as 1000Hz and 2000Hz.
										{/snippet}
									</ConceptInline>
									-- a 64x94 grid of numbers representing frequency content over time.
								</div>
							</div>
							<div class="step">
								<span class="step-num">2</span>
								<div>
									<strong>Flatten the grid.</strong> The 64x94 grid is unrolled into a single list
									of 6,016 numbers, left-to-right, top-to-bottom. The spatial structure is
									destroyed -- the model cannot know that pixel (3,5) is "next to" pixel (3,6).
								</div>
							</div>
							<div class="step">
								<span class="step-num">3</span>
								<div>
									<strong>200 decision trees vote.</strong> Each tree sees the same 6,016 numbers
									but asks different questions in a different order ("Is feature #2041 above 0.3?
									Then go left..."). Each tree makes its own prediction. The final answer is
									whichever class gets the most votes.
								</div>
							</div>
						</div>
					</div>

					<div class="model-detail-section">
						<div class="model-detail-label">THE KEY DIFFERENCE</div>
						<p>
							The Random Forest treats the spectrogram as 6,016 independent numbers. It has
							<strong>no spatial awareness</strong> -- it cannot learn that "energy at frequency
							band 12 <em>next to</em> energy at frequency band 13" forms a harmonic peak. It
							can only learn thresholds on individual pixel values. The fact that it still reaches
							93% tells us the raw features are very informative.
						</p>
					</div>

					<div class="model-io-box">
						<div class="io-col">
							<div class="io-label" style="color: #33ddff;">DATA IN</div>
							<p>
								A 64x94 grid of numbers, flattened to 6,016. Each row is a frequency band (low
								to high). Each column is a time step (~5ms). Values are loudness in decibels.
							</p>
						</div>
						<div class="io-col">
							<div class="io-label" style="color: #00ccaa;">DATA OUT</div>
							<p>
								138 vote counts (one per string+fret position). The class with the most votes
								wins. No softmax -- Random Forests output discrete votes, not probabilities.
							</p>
						</div>
					</div>
				</div>

			<!-- Hybrid CNN detail -->
			{:else if selectedModel === 1}
				<div class="model-detail">
					<div class="model-detail-section">
						<div class="model-detail-label">WHY WE CHOSE IT</div>
						<p>
							The standard approach for audio classification. Convolutional layers learn spatial
							patterns in spectrograms -- harmonic peaks, attack shapes, frequency relationships.
							The dense bottleneck gives maximum flexibility but can memorize training data. This
							is the architecture you would find in most audio-ML tutorials.
						</p>
					</div>

					<div class="model-detail-section">
						<div class="model-detail-label">HOW IT WORKS</div>
						<ModelDiagram model="hybrid-cnn" />
						<div class="step-list">
							<div class="step">
								<span class="step-num">1</span>
								<div>
									<strong>Convolutional feature extraction.</strong> Three convolutional layers
									slide small filters across the spectrogram. Think of each filter as a pattern
									detector -- one might look for "energy at the 2nd harmonic", another for "sharp
									attack transient", another for "frequency decay over time". After the first
									convolution, we have 16 different "views" of the spectrogram, each highlighting
									different patterns.
								</div>
							</div>
							<div class="step">
								<span class="step-num">2</span>
								<div>
									<strong>Pooling shrinks the image.</strong> After each convolution,
									<ConceptInline term="max pooling">
										{#snippet children()}
											Max pooling takes a 2x2 window and keeps only the brightest pixel,
											discarding the rest. This halves the image in each dimension. It makes
											the model tolerant to small shifts -- if a harmonic peak moves one pixel
											left or right, the pooled output is the same.
										{/snippet}
									</ConceptInline>
									halves the spatial dimensions. The spectrogram goes from 64x94 to 32x47 to
									16x23, then adaptive pooling forces it down to 4x4. Each step trades spatial
									detail for more abstract pattern detection.
								</div>
							</div>
							<div class="step">
								<span class="step-num">3</span>
								<div>
									<strong>The dense bottleneck.</strong> The 64 feature maps of size 4x4 are
									flattened into 1,024 numbers, then compressed through a 256-unit dense layer
									with
									<ConceptInline term="dropout">
										{#snippet children()}
											During training, dropout randomly sets 30% of neurons to zero on each
											forward pass. This forces the network to not rely on any single neuron
											and acts as regularization -- a defense against memorizing the training
											data. At test time, all neurons are active.
										{/snippet}
									</ConceptInline>
									(0.3). This is where most of the ~265K parameters live -- and where
									overfitting is most likely with only 10 samples per class.
								</div>
							</div>
							<div class="step">
								<span class="step-num">4</span>
								<div>
									<strong>Classification.</strong> A final linear layer maps 256 features to
									138 class scores.
								</div>
							</div>
						</div>
					</div>

					<div class="model-detail-section">
						<div class="model-detail-label">THE KEY DIFFERENCE</div>
						<p>
							The Hybrid CNN <strong>sees spatial patterns</strong> that the Random Forest misses --
							it knows that adjacent frequency bands forming a harmonic series matter. But it has a
							<strong>1,024 to 256 dense bottleneck</strong> with ~265K learnable parameters. With
							only 10 samples per class, this bottleneck can memorize the training data instead of
							learning generalizable patterns.
						</p>
					</div>

					<div class="model-io-box">
						<div class="io-col">
							<div class="io-label" style="color: #33ddff;">DATA IN</div>
							<p>
								A 64x94 grid of numbers, shaped as a 1-channel "image" (1 x 64 x 94). Each
								row is a frequency band. Each column is a time step. Values are log-magnitude
								in decibels.
							</p>
						</div>
						<div class="io-col">
							<div class="io-label" style="color: #00ccaa;">DATA OUT</div>
							<p>
								138 numbers (one per string+fret position). The highest number is the raw
								prediction. We apply
								<ConceptInline term="softmax">
									{#snippet children()}
										Softmax converts raw scores into probabilities that sum to 1.0. A score
										vector like [2.1, 0.3, -1.0] becomes [0.82, 0.14, 0.04]. The relative
										differences are preserved but now interpretable as confidence levels.
									{/snippet}
								</ConceptInline>
								to convert to probabilities.
							</p>
						</div>
					</div>
				</div>

			<!-- Pure CNN detail -->
			{:else}
				<div class="model-detail">
					<div class="model-detail-section">
						<div class="model-detail-label">WHY WE CHOSE IT</div>
						<p>
							The hypothesis: fewer parameters equals better generalization on small data.
							<ConceptInline term="Global Average Pooling">
								{#snippet children()}
									Traditional CNNs flatten the final feature maps and feed them through dense
									layers. For a 256-channel 8x11 feature map, that would be 22,528 inputs to
									a dense layer -- lots of parameters to overfit on. GAP instead averages each
									8x11 map into a single value, giving just 256 numbers. Dramatically fewer
									parameters, better generalization on small datasets.
								{/snippet}
							</ConceptInline>
							replaces the dense bottleneck, reducing the classifier from ~265K to ~35K parameters.
							With only 10 samples per class, this matters.
						</p>
					</div>

					<div class="model-detail-section">
						<div class="model-detail-label">HOW IT WORKS</div>
						<ModelDiagram model="pure-cnn" />
						<div class="step-list">
							<div class="step">
								<span class="step-num">1</span>
								<div>
									<strong>Deeper feature extraction.</strong> Four convolutional layers (vs three
									in the Hybrid) progressively build up from simple patterns to complex ones.
									The first layer detects edges and peaks. The second combines those into harmonic
									patterns. The third recognizes attack shapes and decay curves. The fourth layer
									sees high-level "fingerprints" of specific string+fret combinations.
								</div>
							</div>
							<div class="step">
								<span class="step-num">2</span>
								<div>
									<strong>Pooling shrinks aggressively.</strong> Three rounds of max-pooling
									reduce the spatial dimensions from 64x94 down to 8x11. Each pooling step
									makes the features more invariant to small timing or pitch shifts in the
									original recording.
								</div>
							</div>
							<div class="step">
								<span class="step-num">3</span>
								<div>
									<strong>Global Average Pooling -- the key.</strong> Instead of flattening 256
									feature maps of 8x11 into 22,528 numbers, GAP averages each map into a
									single number. 256 feature maps become 256 numbers. Each number answers one
									question: "How much of pattern X was present anywhere in the spectrogram?"
									The spatial location is discarded, but the presence is preserved.
								</div>
							</div>
							<div class="step">
								<span class="step-num">4</span>
								<div>
									<strong>Minimal classifier.</strong> Just 256 numbers through dropout (0.4)
									into a single linear layer producing 138 scores. That is 256 x 138 = ~35K
									parameters -- compared to 265K in the Hybrid. Less to memorize, more to
									generalize.
								</div>
							</div>
						</div>
					</div>

					<div class="model-detail-section">
						<div class="model-detail-label">THE KEY DIFFERENCE</div>
						<p>
							The Pure CNN <strong>sees spatial patterns AND has minimal parameters</strong>.
							The convolutional layers (shared feature extractors, ~389K params) learn reusable
							pattern detectors that generalize well. The classifier head is just 256 numbers to
							138 classes -- dramatically simpler than the Hybrid's 1,024 to 256 to 138 pipeline.
							Best of both worlds for small data.
						</p>
					</div>

					<div class="model-io-box">
						<div class="io-col">
							<div class="io-label" style="color: #33ddff;">DATA IN</div>
							<p>
								A 64x94 grid of numbers, shaped as a 1-channel "image" (1 x 64 x 94). Each
								row is a frequency band (low to high). Each column is a time step (~5ms). The
								values are loudness in decibels.
							</p>
						</div>
						<div class="io-col">
							<div class="io-label" style="color: #00ccaa;">DATA OUT</div>
							<p>
								138 numbers (one per string+fret position). The highest number is the
								prediction. We convert to a probability using softmax. With 96.2% accuracy,
								the correct class is typically the clear winner.
							</p>
						</div>
					</div>
				</div>
			{/if}

			<div class="insight-box">
				<div class="insight-label">WHY PURE CNN WINS</div>
				<p>
					With only 10 samples per class, the model with the fewest learnable parameters
					in its classifier head wins. The Pure CNN uses Global Average Pooling to reduce
					256 feature maps to 256 numbers, then a single linear layer to 138 classes.
					That is around 35K parameters in the classifier vs around 265K in the Hybrid CNN.
					The convolutional layers in both models have similar parameter counts -- the
					difference is entirely in how they go from spatial features to a classification
					decision.
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
				<PerStringBars data={results[selectedModel].per_string} />
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
				Interactive charts for the {results[selectedModel].name} model.
				Select a model above to switch between them.
			</p>

			<div class="viz-grid">
				<div class="viz-item">
					<div class="viz-label">Per-String Accuracy</div>
					<PerStringBars data={results[selectedModel].per_string} height={180} />
				</div>
				{#if currentFretGrid.length > 0}
					<div class="viz-item">
						<div class="viz-label">Fret Accuracy Heatmap</div>
						<FretHeatmap
							grid={currentFretGrid}
							counts={currentFretCounts.length > 0 ? currentFretCounts : undefined}
						/>
					</div>
				{/if}
				{#if currentStringConfusion.length > 0}
					<div class="viz-item">
						<div class="viz-label">String Confusion Matrix</div>
						<p class="viz-desc">
							Which strings get confused with which? Diagonal (teal) = correct predictions.
							Off-diagonal (magenta) = misclassifications.
						</p>
						<StringConfusion matrix={currentStringConfusion} />
					</div>
				{/if}
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

	/* Model selector tabs (Station 2) */
	.model-selector-tabs {
		display: flex;
		gap: 2px;
		margin-bottom: 20px;
	}
	.model-selector-tab {
		flex: 1;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 12px 10px;
		cursor: pointer;
		text-align: center;
		font-family: var(--font-reading);
		display: flex;
		flex-direction: column;
		gap: 4px;
		align-items: center;
	}
	.model-selector-tab.active {
		border-color: var(--color-accent-cyan);
		background: rgba(51, 221, 255, 0.05);
	}
	.selector-name {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text-primary);
	}
	.model-selector-tab.active .selector-name {
		color: var(--color-accent-cyan);
	}
	.selector-stats {
		display: flex;
		flex-direction: column;
		gap: 2px;
		align-items: center;
	}
	.selector-acc {
		font-family: var(--font-code);
		font-size: 15px;
	}
	.selector-meta {
		font-size: 10px;
		color: var(--color-text-dim);
	}
	.selector-time {
		font-size: 10px;
		color: var(--color-text-dim);
	}

	/* Model detail content */
	.model-detail {
		margin-bottom: 16px;
	}
	.model-detail-section {
		margin-bottom: 20px;
	}
	.model-detail-label {
		font-family: var(--font-ui);
		font-size: var(--font-size-xs);
		color: var(--color-accent-teal);
		letter-spacing: 2px;
		text-transform: uppercase;
		margin-bottom: 8px;
	}

	/* Step-by-step explanation list */
	.step-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-top: 12px;
	}
	.step {
		display: flex;
		gap: 12px;
		font-size: 13px;
		color: var(--color-text-secondary);
		line-height: 1.7;
	}
	.step-num {
		flex-shrink: 0;
		width: 22px;
		height: 22px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		font-family: var(--font-code);
		font-size: 10px;
		color: var(--color-accent-cyan);
		margin-top: 2px;
	}
	.step strong {
		color: var(--color-text-primary);
	}

	/* Data in / Data out box */
	.model-io-box {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1px;
		background: var(--color-border);
		border: 1px solid var(--color-border);
		margin-top: 16px;
	}
	.io-col {
		background: var(--color-bg-panel);
		padding: 12px 14px;
	}
	.io-col p {
		font-size: 12px;
		color: var(--color-text-secondary);
		line-height: 1.6;
		margin: 0;
	}
	.io-label {
		font-family: var(--font-ui);
		font-size: var(--font-size-xs);
		letter-spacing: 2px;
		margin-bottom: 6px;
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
		font-family: var(--font-code);
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
		font-family: var(--font-ui);
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
	.viz-desc {
		font-size: 12px;
		color: var(--color-text-dim);
		line-height: 1.5;
		margin: 0 0 12px 0;
	}

	/* Hear the Data section */
	.hear-section {
		margin-top: 24px;
	}
	.hear-heading {
		font-family: var(--font-ui);
		font-size: var(--font-size-xs);
		color: var(--color-accent-teal);
		letter-spacing: 2px;
		text-transform: uppercase;
		margin-bottom: 8px;
	}
	.hear-desc {
		font-size: 13px;
		color: var(--color-text-secondary);
		line-height: 1.6;
		margin: 0 0 12px 0;
	}
	.sample-grid {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	/* Next link */
	.next-link {
		margin-top: 16px;
	}
	.next-link a {
		display: inline-block;
		font-family: var(--font-ui);
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

	/* Overview spectrogram */
	.overview-spectrogram {
		margin: 16px 0;
	}
</style>
