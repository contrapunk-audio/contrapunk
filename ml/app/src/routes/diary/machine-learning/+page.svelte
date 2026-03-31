<script lang="ts">
	// Round 1 results (will be loaded from JSON in future rounds)
	const round1 = {
		date: '2026-03-31',
		title: 'Round 1: Raw Baseline',
		subtitle: 'No preprocessing. No augmentation. Just raw audio → mel-spectrograms → classifiers.',
		dataset: {
			samples: 1380,
			classes: 138,
			perClass: 10,
			duration: 0.5,
			sampleRate: 48000,
			features: 'Log-mel spectrogram (64 mels, 1024 FFT, 256 hop)',
			preprocessing: 'None',
			qualityIssues: '32 clipped, 7 near-silent',
		},
		models: [
			{
				name: 'Random Forest',
				accuracy: 0.933,
				trainTime: '4.3s',
				notes: '200 trees, flattened mel-spectrogram features',
				perString: [0.861, 0.913, 0.970, 0.957, 0.935, 0.965],
			},
			{
				name: 'Hybrid CNN',
				accuracy: 0.914,
				trainTime: '84.6s',
				notes: '3 conv layers + FC, 50 epochs, Adam + cosine LR',
				perString: [0.917, 0.900, 0.913, 0.896, 0.922, 0.939],
			},
			{
				name: 'Pure CNN',
				accuracy: 0.964,
				trainTime: '302.1s',
				notes: '4 conv layers + GAP, 60 epochs, Adam + cosine LR',
				perString: [0.957, 0.970, 0.974, 0.961, 0.970, 0.952],
			},
		],
	};

	const stringNames = ['E2 (low)', 'A2', 'D3', 'G3', 'B3', 'E4 (high)'];

	const concepts = {
		melSpectrogram: {
			title: 'Mel-Spectrogram',
			content: `A mel-spectrogram converts raw audio into a 2D "image" where the x-axis is time, y-axis is frequency (on a perceptual scale), and color intensity is magnitude. The mel scale matches human hearing — we perceive the difference between 100Hz and 200Hz the same as 1000Hz and 2000Hz. For each 0.5s guitar sample at 48kHz, we get a 64×94 pixel image that a CNN can process like a photo.`,
		},
		randomForest: {
			title: 'Random Forest',
			content: `A random forest is an ensemble of decision trees. Each tree asks yes/no questions about input features: "Is the energy at 440Hz above 0.5?" Each tree sees a random subset of features, then they vote. For our guitar classifier, we flatten the 64×94 mel-spectrogram into 6,016 numbers and let 200 trees vote on which string+fret it is. RFs are fast, interpretable, and work well with small datasets — the perfect baseline.`,
		},
		hybridCnn: {
			title: 'Hybrid CNN',
			content: `The Hybrid CNN passes the spectrogram through 3 convolutional layers (learning spatial patterns like "harmonic peaks at these frequencies") followed by dense (fully connected) layers that combine those patterns into a classification. The "hybrid" part is the dense bottleneck — 64×4×4=1024 features compressed to 256 before the final 138-class output. This bottleneck can memorize training data on small datasets, which is why it slightly underperforms the Pure CNN here.`,
		},
		pureCnn: {
			title: 'Pure CNN',
			content: `The Pure CNN uses 4 convolutional layers followed by Global Average Pooling (GAP) — instead of flattening all spatial features into a dense layer, GAP averages each feature map into a single number. This means only 256 numbers feed into the classifier (vs 1024 in the Hybrid). With only 10 samples per class, fewer parameters = less overfitting. That's why Pure CNN hits 96.4% — the best architecture for our data-scarce scenario.`,
		},
		stratifiedCV: {
			title: '5-Fold Stratified Cross-Validation',
			content: `With only 10 samples per class, a single train/test split would be noisy — one unlucky split could dramatically change the results. Stratified 5-fold CV splits data into 5 equal parts, trains on 4 and validates on 1, rotating 5 times. "Stratified" means each fold preserves the class distribution (each fold gets 2 samples per class). The reported accuracy is the average across all 5 folds — a much more reliable estimate.`,
		},
		globalAvgPool: {
			title: 'Global Average Pooling',
			content: `Traditional CNNs flatten the final feature maps and feed them through dense layers. For a 64-channel 4×4 feature map, that's 1024 inputs to a dense layer — lots of parameters to overfit on. GAP instead averages each 4×4 map into a single value, giving just 64 (or 256) numbers. Dramatically fewer parameters, better generalization on small datasets. Originally proposed in the 2014 "Network in Network" paper, now standard in modern architectures.`,
		},
	};

	let expandedConcept = $state<string | null>(null);
	let selectedModel = $state<number>(2); // Pure CNN by default
	let showConfusion = $state(false);
	let activeTab = $state<'overview' | 'models' | 'results' | 'next'>('overview');

	function toggleConcept(key: string) {
		expandedConcept = expandedConcept === key ? null : key;
	}

	function pct(n: number): string {
		return (n * 100).toFixed(1) + '%';
	}

	function barWidth(n: number): string {
		return (n * 100).toFixed(1) + '%';
	}

	function barColor(n: number): string {
		if (n >= 0.95) return 'bg-emerald-500';
		if (n >= 0.90) return 'bg-cyan-500';
		if (n >= 0.85) return 'bg-amber-500';
		return 'bg-red-500';
	}
</script>

<svelte:head>
	<title>ML Diary — Contrapunk</title>
</svelte:head>

<div class="min-h-screen bg-gray-950 text-gray-100">
	<div class="max-w-5xl mx-auto px-6 py-12">
		<!-- Header -->
		<div class="mb-4">
			<a href="/" class="inline-flex items-center gap-2 text-cyan-400 hover:text-cyan-300 text-sm transition-colors">
				&larr; Back to Pipeline
			</a>
		</div>

		<header class="mb-12">
			<h1 class="text-4xl font-bold text-cyan-400 mb-2">Machine Learning Diary</h1>
			<p class="text-lg text-gray-400">
				How we're training a guitar string+fret classifier — every decision, every result, explained.
			</p>
			<p class="text-sm text-gray-600 mt-2">
				This is a living document. Each training round adds a new entry with what changed, why, and what we learned.
			</p>
		</header>

		<!-- The Approach -->
		<section class="mb-12 bg-gray-900 border border-gray-800 rounded-xl p-8">
			<h2 class="text-2xl font-bold text-cyan-300 mb-4">The Approach: Iterative Improvement</h2>
			<p class="text-gray-300 leading-relaxed mb-4">
				Instead of preprocessing everything, tuning every hyperparameter, and training once — we train on
				<strong class="text-cyan-400">raw data first</strong>, then improve one thing at a time. Each round
				documents exactly what changed and how it affected accuracy.
			</p>
			<p class="text-gray-300 leading-relaxed mb-6">
				This matters because most ML tutorials skip straight to the final pipeline. You never see <em>why</em>
				onset alignment matters, or <em>how much</em> augmentation actually helps. Our diary shows the real,
				messy, iterative process of building an ML system.
			</p>

			<div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
				<div class="bg-gray-950 border border-emerald-500/20 rounded-lg p-4 text-center">
					<div class="text-3xl font-bold text-emerald-400">1,380</div>
					<div class="text-sm text-gray-500">samples captured</div>
				</div>
				<div class="bg-gray-950 border border-cyan-500/20 rounded-lg p-4 text-center">
					<div class="text-3xl font-bold text-cyan-400">138</div>
					<div class="text-sm text-gray-500">classes (6 strings &times; 23 frets)</div>
				</div>
				<div class="bg-gray-950 border border-amber-500/20 rounded-lg p-4 text-center">
					<div class="text-3xl font-bold text-amber-400">10</div>
					<div class="text-sm text-gray-500">samples per class</div>
				</div>
			</div>
		</section>

		<!-- Round 1 Diary Entry -->
		<section class="mb-12">
			<div class="flex items-center gap-4 mb-6">
				<div class="flex-shrink-0 w-12 h-12 rounded-full bg-cyan-500/10 border-2 border-cyan-500/40 flex items-center justify-center">
					<span class="text-cyan-400 font-bold text-lg">1</span>
				</div>
				<div>
					<h2 class="text-2xl font-bold text-white">{round1.title}</h2>
					<p class="text-sm text-gray-500">{round1.date} &mdash; {round1.subtitle}</p>
				</div>
			</div>

			<!-- Tab Navigation -->
			<div class="flex gap-1 mb-6 bg-gray-900 rounded-lg p-1 border border-gray-800">
				{#each [
					{ key: 'overview', label: 'Overview' },
					{ key: 'models', label: 'How the Models Work' },
					{ key: 'results', label: 'Results & Analysis' },
					{ key: 'next', label: 'What\'s Next' },
				] as tab}
					<button
						class="flex-1 py-2 px-4 rounded-md text-sm font-medium transition-colors {activeTab === tab.key
							? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/30'
							: 'text-gray-500 hover:text-gray-300 border border-transparent'}"
						onclick={() => activeTab = tab.key as typeof activeTab}
					>
						{tab.label}
					</button>
				{/each}
			</div>

			<!-- Tab: Overview -->
			{#if activeTab === 'overview'}
				<div class="space-y-6">
					<div class="bg-gray-900 border border-gray-800 rounded-xl p-6">
						<h3 class="text-lg font-semibold text-cyan-300 mb-4">What We Did</h3>
						<ol class="space-y-3 text-gray-300">
							<li class="flex gap-3">
								<span class="flex-shrink-0 w-6 h-6 rounded-full bg-cyan-500/10 text-cyan-400 text-xs font-bold flex items-center justify-center">1</span>
								<span>Captured 1,380 guitar samples (6 strings &times; 23 frets &times; 10 plucks) using Audient iD14 DI input</span>
							</li>
							<li class="flex gap-3">
								<span class="flex-shrink-0 w-6 h-6 rounded-full bg-cyan-500/10 text-cyan-400 text-xs font-bold flex items-center justify-center">2</span>
								<span>Extracted log-mel spectrograms (64 mel bands, 0.5s windows) &mdash; turning audio into "images"</span>
							</li>
							<li class="flex gap-3">
								<span class="flex-shrink-0 w-6 h-6 rounded-full bg-cyan-500/10 text-cyan-400 text-xs font-bold flex items-center justify-center">3</span>
								<span>Trained 3 different classifiers on the raw spectrograms with 5-fold cross-validation</span>
							</li>
							<li class="flex gap-3">
								<span class="flex-shrink-0 w-6 h-6 rounded-full bg-cyan-500/10 text-cyan-400 text-xs font-bold flex items-center justify-center">4</span>
								<span>Measured accuracy per-string, per-fret to find where the model struggles</span>
							</li>
						</ol>
					</div>

					<div class="bg-gray-900 border border-gray-800 rounded-xl p-6">
						<h3 class="text-lg font-semibold text-cyan-300 mb-4">Dataset Details</h3>
						<div class="grid grid-cols-2 gap-4 text-sm">
							{#each Object.entries({
								'Guitar': 'Ibanez Artcore AG85 (hollow body)',
								'Interface': 'Audient iD14 (DI input)',
								'Sample Rate': '48,000 Hz',
								'Duration': '0.5s per sample',
								'Features': round1.dataset.features,
								'Preprocessing': round1.dataset.preprocessing,
								'Quality Issues': round1.dataset.qualityIssues,
								'Evaluation': '5-fold stratified cross-validation',
							}) as [key, value]}
								<div class="text-gray-500">{key}</div>
								<div class="text-gray-300">{value}</div>
							{/each}
						</div>
					</div>

					<!-- Concept cards -->
					<div class="bg-gray-900 border border-gray-800 rounded-xl p-6">
						<h3 class="text-lg font-semibold text-cyan-300 mb-4">Key Concepts Used</h3>
						<div class="space-y-2">
							{#each [
								['melSpectrogram', concepts.melSpectrogram],
								['stratifiedCV', concepts.stratifiedCV],
							] as [key, concept]}
								<button
									class="w-full text-left bg-gray-950 border border-gray-800 rounded-lg p-4 hover:border-cyan-500/30 transition-colors"
									onclick={() => toggleConcept(key)}
								>
									<div class="flex items-center justify-between">
										<span class="font-medium text-cyan-400">{concept.title}</span>
										<span class="text-gray-600 text-sm">{expandedConcept === key ? '−' : '+'}</span>
									</div>
									{#if expandedConcept === key}
										<p class="mt-3 text-sm text-gray-400 leading-relaxed">{concept.content}</p>
									{/if}
								</button>
							{/each}
						</div>
					</div>
				</div>
			{/if}

			<!-- Tab: Models -->
			{#if activeTab === 'models'}
				<div class="space-y-6">
					{#each [
						['randomForest', concepts.randomForest, 'Model 1'],
						['hybridCnn', concepts.hybridCnn, 'Model 2'],
						['pureCnn', concepts.pureCnn, 'Model 3'],
					] as [key, concept, label]}
						<div class="bg-gray-900 border border-gray-800 rounded-xl p-6">
							<div class="flex items-center gap-3 mb-3">
								<span class="text-xs font-mono text-gray-600">{label}</span>
								<h3 class="text-lg font-semibold text-cyan-300">{concept.title}</h3>
								<span class="ml-auto text-lg font-bold {
									round1.models[['randomForest', 'hybridCnn', 'pureCnn'].indexOf(key)].accuracy >= 0.95
										? 'text-emerald-400'
										: round1.models[['randomForest', 'hybridCnn', 'pureCnn'].indexOf(key)].accuracy >= 0.90
										? 'text-cyan-400'
										: 'text-amber-400'
								}">
									{pct(round1.models[['randomForest', 'hybridCnn', 'pureCnn'].indexOf(key)].accuracy)}
								</span>
							</div>
							<p class="text-sm text-gray-400 leading-relaxed mb-4">{concept.content}</p>

							<!-- Model architecture diagram -->
							{#if key === 'randomForest'}
								<div class="bg-gray-950 rounded-lg p-4 font-mono text-xs text-gray-500 overflow-x-auto">
									<pre>Audio (24,000 samples)
  → Mel-spectrogram (64 × 94)
  → Flatten (6,016 features)
  → 200 Decision Trees (vote)
  → Predicted class (1 of 138)</pre>
								</div>
							{:else if key === 'hybridCnn'}
								<div class="bg-gray-950 rounded-lg p-4 font-mono text-xs text-gray-500 overflow-x-auto">
									<pre>Mel-spectrogram (1 × 64 × 94)
  → Conv 16 filters → BN → ReLU → MaxPool
  → Conv 32 filters → BN → ReLU → MaxPool
  → Conv 64 filters → BN → ReLU → AdaptiveAvgPool(4×4)
  → Flatten (1,024) → Dense 256 → ReLU → Dropout(0.3)
  → Dense 138 (output)</pre>
								</div>
							{:else}
								<div class="bg-gray-950 rounded-lg p-4 font-mono text-xs text-gray-500 overflow-x-auto">
									<pre>Mel-spectrogram (1 × 64 × 94)
  → Conv 32 filters → BN → ReLU → MaxPool
  → Conv 64 filters → BN → ReLU → MaxPool
  → Conv 128 filters → BN → ReLU → MaxPool
  → Conv 256 filters → BN → ReLU → GAP(1×1)
  → Flatten (256) → Dropout(0.4)
  → Dense 138 (output)</pre>
								</div>
							{/if}

							<div class="mt-4 flex items-center gap-4 text-xs text-gray-600">
								<span>Train time: {round1.models[['randomForest', 'hybridCnn', 'pureCnn'].indexOf(key)].trainTime}</span>
								<span>&bull;</span>
								<span>{round1.models[['randomForest', 'hybridCnn', 'pureCnn'].indexOf(key)].notes}</span>
							</div>
						</div>
					{/each}

					<!-- Why Pure CNN wins explanation -->
					<div class="bg-gray-900 border border-amber-500/20 rounded-xl p-6">
						<h3 class="text-lg font-semibold text-amber-300 mb-3">Why does the Pure CNN beat the others?</h3>
						<p class="text-sm text-gray-400 leading-relaxed mb-3">
							With only <strong class="text-white">10 samples per class</strong>, the model with the
							fewest learnable parameters in its classifier head wins. The Pure CNN uses Global Average
							Pooling to reduce 256 feature maps to 256 numbers &mdash; then a single linear layer to
							138 classes. That's ~35K parameters in the classifier.
						</p>
						<p class="text-sm text-gray-400 leading-relaxed">
							The Hybrid CNN flattens to 1,024 features then uses a 256-neuron hidden layer &mdash;
							~265K parameters just in the classifier. More parameters + less data = more overfitting.
							The Random Forest avoids overfitting differently (bagging), but flattening destroys the 2D
							spatial structure of the spectrogram that convolutions exploit.
						</p>

						<button
							class="mt-4 w-full text-left bg-gray-950 border border-gray-800 rounded-lg p-4 hover:border-amber-500/30 transition-colors"
							onclick={() => toggleConcept('globalAvgPool')}
						>
							<div class="flex items-center justify-between">
								<span class="font-medium text-amber-400">Learn: {concepts.globalAvgPool.title}</span>
								<span class="text-gray-600 text-sm">{expandedConcept === 'globalAvgPool' ? '−' : '+'}</span>
							</div>
							{#if expandedConcept === 'globalAvgPool'}
								<p class="mt-3 text-sm text-gray-400 leading-relaxed">{concepts.globalAvgPool.content}</p>
							{/if}
						</button>
					</div>
				</div>
			{/if}

			<!-- Tab: Results -->
			{#if activeTab === 'results'}
				<div class="space-y-6">
					<!-- Summary table -->
					<div class="bg-gray-900 border border-gray-800 rounded-xl p-6">
						<h3 class="text-lg font-semibold text-cyan-300 mb-4">Overall Accuracy</h3>
						<div class="space-y-3">
							{#each round1.models as model, i}
								<button
									class="w-full text-left p-3 rounded-lg transition-colors {selectedModel === i
										? 'bg-cyan-500/10 border border-cyan-500/30'
										: 'bg-gray-950 border border-gray-800 hover:border-gray-700'}"
									onclick={() => selectedModel = i}
								>
									<div class="flex items-center justify-between mb-2">
										<span class="font-medium text-white">{model.name}</span>
										<span class="font-bold {model.accuracy >= 0.95
											? 'text-emerald-400'
											: model.accuracy >= 0.90
											? 'text-cyan-400'
											: 'text-amber-400'}">{pct(model.accuracy)}</span>
									</div>
									<div class="w-full bg-gray-800 rounded-full h-2">
										<div
											class="h-2 rounded-full transition-all duration-500 {barColor(model.accuracy)}"
											style="width: {barWidth(model.accuracy)}"
										></div>
									</div>
								</button>
							{/each}
						</div>
					</div>

					<!-- Per-string accuracy -->
					<div class="bg-gray-900 border border-gray-800 rounded-xl p-6">
						<h3 class="text-lg font-semibold text-cyan-300 mb-4">
							Per-String Accuracy &mdash; {round1.models[selectedModel].name}
						</h3>
						<div class="space-y-3">
							{#each stringNames as name, i}
								{@const acc = round1.models[selectedModel].perString[i]}
								<div class="flex items-center gap-4">
									<span class="w-20 text-sm text-gray-400 font-mono">{name}</span>
									<div class="flex-1 bg-gray-800 rounded-full h-3">
										<div
											class="h-3 rounded-full transition-all duration-500 {barColor(acc)}"
											style="width: {barWidth(acc)}"
										></div>
									</div>
									<span class="w-14 text-right text-sm font-bold {
										acc >= 0.95 ? 'text-emerald-400' : acc >= 0.90 ? 'text-cyan-400' : 'text-amber-400'
									}">{pct(acc)}</span>
								</div>
							{/each}
						</div>
					</div>

					<!-- Visualizations -->
					<div class="bg-gray-900 border border-gray-800 rounded-xl p-6">
						<h3 class="text-lg font-semibold text-cyan-300 mb-4">Visualizations</h3>

						<div class="space-y-6">
							<!-- Per-string bar chart -->
							<div>
								<h4 class="text-sm font-medium text-gray-400 mb-2">Per-String Accuracy</h4>
								<img
									src="/training/round_01/per_string_{round1.models[selectedModel].name.toLowerCase().replace(' ', '_')}.png"
									alt="{round1.models[selectedModel].name} per-string accuracy"
									class="w-full rounded-lg border border-gray-800"
								/>
							</div>

							<!-- Fret heatmap -->
							<div>
								<h4 class="text-sm font-medium text-gray-400 mb-2">Per-Fret Accuracy Heatmap</h4>
								<img
									src="/training/round_01/fret_heatmap_{round1.models[selectedModel].name.toLowerCase().replace(' ', '_')}.png"
									alt="{round1.models[selectedModel].name} fret heatmap"
									class="w-full rounded-lg border border-gray-800"
								/>
							</div>

							<!-- Confusion matrix (expandable) -->
							<div>
								<button
									class="flex items-center gap-2 text-sm font-medium text-gray-400 hover:text-cyan-300 transition-colors"
									onclick={() => showConfusion = !showConfusion}
								>
									<span>{showConfusion ? '▼' : '▶'} Confusion Matrix (138x138)</span>
								</button>
								{#if showConfusion}
									<img
										src="/training/round_01/confusion_{round1.models[selectedModel].name.toLowerCase().replace(' ', '_')}.png"
										alt="{round1.models[selectedModel].name} confusion matrix"
										class="w-full rounded-lg border border-gray-800 mt-2"
									/>
								{/if}
							</div>

							<!-- Training curves (CNN only) -->
							{#if selectedModel > 0}
								<div>
									<h4 class="text-sm font-medium text-gray-400 mb-2">Training Curves</h4>
									<img
										src="/training/round_01/training_curves_{round1.models[selectedModel].name.toLowerCase().replace(' ', '_')}.png"
										alt="{round1.models[selectedModel].name} training curves"
										class="w-full rounded-lg border border-gray-800"
									/>
								</div>
							{/if}
						</div>
					</div>

					<!-- Observations -->
					<div class="bg-gray-900 border border-gray-800 rounded-xl p-6">
						<h3 class="text-lg font-semibold text-cyan-300 mb-4">What We Learned</h3>
						<ul class="space-y-3 text-sm text-gray-300">
							<li class="flex gap-3">
								<span class="text-emerald-400 mt-0.5">&#10003;</span>
								<span><strong class="text-white">Raw mel-spectrograms work surprisingly well</strong> &mdash;
								96.4% accuracy on 138 classes with zero preprocessing shows the audio signal is rich enough
								for classification even without feature engineering.</span>
							</li>
							<li class="flex gap-3">
								<span class="text-emerald-400 mt-0.5">&#10003;</span>
								<span><strong class="text-white">Low E string is hardest</strong> &mdash;
								RF gets only 86.1% on E2. The thick wound string has more complex harmonics and the
								frets are physically closer together in the low register.</span>
							</li>
							<li class="flex gap-3">
								<span class="text-emerald-400 mt-0.5">&#10003;</span>
								<span><strong class="text-white">Pure CNN &gt; RF &gt; Hybrid CNN</strong> &mdash;
								Global Average Pooling beats the dense bottleneck when data is scarce.
								The RF's strong showing (93.3%) validates that the features are genuinely informative.</span>
							</li>
							<li class="flex gap-3">
								<span class="text-amber-400 mt-0.5">!</span>
								<span><strong class="text-white">32 clipped samples didn't hurt much</strong> &mdash;
								we left quality issues in intentionally. If cleaning them improves accuracy in Round 2,
								we'll know the impact precisely.</span>
							</li>
						</ul>
					</div>
				</div>
			{/if}

			<!-- Tab: What's Next -->
			{#if activeTab === 'next'}
				<div class="space-y-6">
					<div class="bg-gray-900 border border-gray-800 rounded-xl p-6">
						<h3 class="text-lg font-semibold text-cyan-300 mb-4">Planned Rounds</h3>
						<p class="text-sm text-gray-400 mb-6">
							Each round changes exactly one thing so we can measure its impact. No magic &mdash;
							just systematic, documented improvement.
						</p>

						<div class="space-y-4">
							{#each [
								{
									round: 2,
									title: 'Onset Alignment',
									description: 'Trim silence before the note attack so every spectrogram starts at the pluck. Currently, some samples have 50-100ms of silence at the start, wasting spectrogram resolution.',
									expectedImpact: 'Should improve fret discrimination (the attack transient is where most fret-specific information lives).',
								},
								{
									round: 3,
									title: 'Quality Cleanup',
									description: 'Remove or re-record the 32 clipped samples and 7 near-silent samples. Clipping distorts the harmonic content; silence teaches the model nothing useful.',
									expectedImpact: 'Small accuracy bump, but more importantly: cleaner confusion matrix.',
								},
								{
									round: 4,
									title: 'Noise Samples',
									description: 'Capture ambient noise, string muting, pick scrapes, finger slides, accidental touches. Add a 139th "noise" class so the model can reject non-note input.',
									expectedImpact: 'Critical for real-world use — without noise rejection, every sound triggers a classification.',
								},
								{
									round: 5,
									title: 'Goertzel Harmonic Features',
									description: 'Add harmonic ratio features (h2/h1, h3/h1...) as extra input channels. These are the physics-based fingerprint of each string.',
									expectedImpact: 'Should dramatically improve string identification (same-note-different-string confusion).',
								},
								{
									round: 6,
									title: 'Data Augmentation',
									description: 'Gain variation, noise injection, time jitter. NOT pitch shifting (would corrupt fret labels).',
									expectedImpact: 'More training diversity → better generalization, especially for low-sample classes.',
								},
							] as plan}
								<div class="bg-gray-950 border border-gray-800 rounded-lg p-4">
									<div class="flex items-center gap-3 mb-2">
										<span class="flex-shrink-0 w-7 h-7 rounded-full bg-gray-800 text-gray-500 text-xs font-bold flex items-center justify-center">
											{plan.round}
										</span>
										<h4 class="font-medium text-white">{plan.title}</h4>
									</div>
									<p class="text-sm text-gray-400 ml-10 mb-2">{plan.description}</p>
									<p class="text-sm text-cyan-400/70 ml-10 italic">{plan.expectedImpact}</p>
								</div>
							{/each}
						</div>
					</div>

					<div class="bg-gray-900 border border-emerald-500/20 rounded-xl p-6">
						<h3 class="text-lg font-semibold text-emerald-300 mb-3">The Goal</h3>
						<p class="text-sm text-gray-400 leading-relaxed">
							A Pure Rust inference engine running inside Contrapunk that identifies which string and fret
							you're playing from audio input &mdash; in real-time, with no external dependencies, working
							in both native and WASM builds. The model weights get embedded directly into the binary
							via <code class="text-emerald-400">include_bytes!</code>. Target: &gt;98% accuracy with
							&lt;10ms inference time.
						</p>
					</div>
				</div>
			{/if}
		</section>

		<!-- Future round entries will go here -->
		<section class="border-t border-gray-800 pt-8 mb-12">
			<div class="flex items-center gap-4 opacity-40">
				<div class="flex-shrink-0 w-12 h-12 rounded-full bg-gray-900 border-2 border-gray-800 border-dashed flex items-center justify-center">
					<span class="text-gray-600 font-bold text-lg">2</span>
				</div>
				<div>
					<h2 class="text-xl font-bold text-gray-600">Round 2: Onset Alignment</h2>
					<p class="text-sm text-gray-700">Coming soon...</p>
				</div>
			</div>
		</section>

		<footer class="text-center text-gray-600 text-sm py-8">
			<p>Contrapunk &mdash; Open-source MIDI harmony generator with ML guitar classification</p>
		</footer>
	</div>
</div>
