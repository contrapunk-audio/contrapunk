<script lang="ts">
	const steps = [
		{ number: 1, title: 'Raw Data Explorer', href: '/raw-data', description: 'Inspect waveforms, amplitudes, and sample distributions' },
		{ number: 2, title: 'Onset Alignment', href: '/onset', description: 'Align note onsets for consistent training windows' },
		{ number: 3, title: 'Feature Extraction', href: '/features', description: 'Extract spectral and temporal features from audio' },
		{ number: 4, title: 'Normalization', href: '/normalization', description: 'Normalize features for consistent model input' },
		{ number: 5, title: 'Augmentation', href: '/augmentation', description: 'Apply data augmentation to expand training set' },
		{ number: 6, title: 'Dataset Validation', href: '/validation', description: 'Validate dataset integrity and class balance' },
		{ number: 7, title: 'Training', href: '/training', description: 'Train the classifier model' },
		{ number: 8, title: 'Model Comparison', href: '/comparison', description: 'Compare model architectures and hyperparameters' },
		{ number: 9, title: 'Ensemble', href: '/ensemble', description: 'Combine models for improved accuracy' },
		{ number: 10, title: 'Live Demo', href: '/live', description: 'Real-time guitar string and fret classification' },
	];

	const diary = {
		title: 'ML Diary',
		href: '/diary/machine-learning',
		description: 'The full training journey — every round, every decision, every result explained.',
	};
</script>

<svelte:head>
	<title>Contrapunk ML Pipeline</title>
</svelte:head>

<div class="min-h-screen bg-gray-950 text-gray-100">
	<div class="max-w-6xl mx-auto px-6 py-16">
		<header class="text-center mb-16">
			<h1 class="text-5xl font-bold text-cyan-400 mb-4">Contrapunk ML Pipeline</h1>
			<p class="text-xl text-gray-400">Guitar String+Fret Classifier — Visual Learning Environment</p>
		</header>

		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			{#each steps as step}
				<a
					href={step.href}
					class="group block bg-gray-900 border border-gray-800 rounded-xl p-6 hover:border-cyan-500/50 hover:bg-gray-900/80 transition-all duration-200"
				>
					<div class="flex items-start gap-4">
						<span class="flex-shrink-0 w-10 h-10 rounded-lg bg-cyan-500/10 text-cyan-400 font-bold text-lg flex items-center justify-center border border-cyan-500/20">
							{step.number}
						</span>
						<div>
							<h2 class="text-lg font-semibold text-cyan-300 group-hover:text-cyan-200 transition-colors">
								{step.title}
							</h2>
							<p class="text-sm text-gray-500 mt-1">{step.description}</p>
						</div>
					</div>
				</a>
			{/each}
		</div>

		<!-- ML Diary -->
		<div class="mt-10">
			<a
				href={diary.href}
				class="group block bg-gradient-to-r from-cyan-950/40 to-gray-900 border border-cyan-500/30 rounded-xl p-6 hover:border-cyan-400/50 transition-all duration-200"
			>
				<div class="flex items-center gap-4">
					<span class="flex-shrink-0 w-12 h-12 rounded-lg bg-cyan-500/10 text-cyan-400 font-bold text-2xl flex items-center justify-center border border-cyan-500/20">
						&#128214;
					</span>
					<div>
						<h2 class="text-xl font-semibold text-cyan-300 group-hover:text-cyan-200 transition-colors">
							{diary.title}
						</h2>
						<p class="text-sm text-gray-400 mt-1">{diary.description}</p>
					</div>
				</div>
			</a>
		</div>

		<footer class="text-center mt-16 text-gray-600 text-sm">
			<p>Contrapunk — ML pipeline visualization for guitar classification</p>
		</footer>
	</div>
</div>
