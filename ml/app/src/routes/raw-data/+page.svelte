<svelte:head>
	<title>Raw Data Explorer — Contrapunk ML</title>
</svelte:head>

<div class="min-h-screen bg-gray-950 text-gray-100">
	<div class="max-w-6xl mx-auto px-6 py-12">
		<a href="/" class="inline-flex items-center gap-2 text-cyan-400 hover:text-cyan-300 text-sm mb-8 transition-colors">
			&larr; Back to Pipeline
		</a>

		<header class="mb-12">
			<div class="flex items-center gap-3 mb-2">
				<span class="w-10 h-10 rounded-lg bg-cyan-500/10 text-cyan-400 font-bold text-lg flex items-center justify-center border border-cyan-500/20">1</span>
				<h1 class="text-3xl font-bold text-cyan-400">Raw Data Explorer</h1>
			</div>
			<p class="text-gray-400 ml-13">Inspect waveforms, amplitudes, and sample distributions from the raw dataset.</p>
		</header>

		<div class="bg-gray-900 border border-gray-800 rounded-xl p-8 mb-8">
			<h2 class="text-xl font-semibold text-gray-200 mb-4">Analysis Outputs</h2>
			<p class="text-gray-500 mb-6">
				Run the analysis script to generate visualizations:
			</p>
			<code class="block bg-gray-950 text-cyan-300 px-4 py-3 rounded-lg text-sm font-mono mb-6">
				python ml/processing/01_raw_analysis/analyze.py
			</code>
			<p class="text-gray-500 text-sm">
				Images will be saved to <code class="text-cyan-400/70">ml/processing/01_raw_analysis/*.png</code>
			</p>
		</div>

		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
			{#each analyses as analysis}
				<div class="bg-gray-900 border border-gray-800 rounded-xl p-6">
					<div class="flex items-center gap-3 mb-3">
						<div class="w-2 h-2 rounded-full bg-gray-600"></div>
						<h3 class="text-sm font-semibold text-gray-300">{analysis.title}</h3>
					</div>
					<p class="text-xs text-gray-600">{analysis.description}</p>
				</div>
			{/each}
		</div>
	</div>
</div>

<script lang="ts">
	const analyses = [
		{ title: 'Waveform Grid', description: 'Overlay of waveforms per string/fret combination' },
		{ title: 'Amplitude Histogram', description: 'Distribution of peak amplitudes across all samples' },
		{ title: 'Pitch Accuracy', description: 'Detected pitch vs expected frequency for each class' },
		{ title: 'Duration Histogram', description: 'Distribution of sample durations' },
		{ title: 'Sample Counts', description: 'Number of samples per string/fret class' },
		{ title: 'Clipping Report', description: 'Samples with clipping or DC offset issues' },
	];
</script>
