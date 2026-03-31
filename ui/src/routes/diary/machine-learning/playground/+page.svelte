<script lang="ts">
	import DiaryNav from '$lib/components/diary/DiaryNav.svelte';
	import ConceptInline from '$lib/components/diary/ConceptInline.svelte';
	import SpectrogramViewer from '$lib/components/diary/SpectrogramViewer.svelte';
	import {
		extractMelSpectrogram,
		melSpectrogramToModelInput,
		topKPredictions,
		resampleLinear,
		classIndexToLabel,
	} from '$lib/components/diary/melSpectrogram';

	// ---------------------------------------------------------------------------
	// State
	// ---------------------------------------------------------------------------

	type PlaygroundState = 'idle' | 'loading' | 'extracting' | 'ready' | 'inferring' | 'done' | 'error';

	let state: PlaygroundState = $state('idle');
	let errorMessage = $state('');

	// Audio
	let audioBuffer: AudioBuffer | null = $state(null);
	let audioFileName = $state('');
	let audioDuration = $state(0);

	// Spectrogram
	let melSpec: number[][] | null = $state(null);
	let modelInput: Float32Array | null = $state(null);

	// Results (placeholder for when inference is wired up)
	let predictions: { classIndex: number; probability: number; label: string }[] = $state([]);

	// Canvas refs
	let waveformCanvas: HTMLCanvasElement | undefined = $state();
	let fileInput: HTMLInputElement | undefined = $state();

	// ---------------------------------------------------------------------------
	// Constants
	// ---------------------------------------------------------------------------

	const MODEL_PARAMS = {
		nMels: 64,
		nFft: 1024,
		hopLength: 256,
		targetSampleRate: 48000,
		targetFrames: 94,
	};

	const crumbs = [
		{ label: 'Diary', href: '/diary' },
		{ label: 'Machine Learning', href: '/diary/machine-learning' },
		{ label: 'Playground' },
	];

	// ---------------------------------------------------------------------------
	// File upload handler
	// ---------------------------------------------------------------------------

	async function handleFileUpload(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		// Reset state
		predictions = [];
		melSpec = null;
		modelInput = null;
		audioBuffer = null;
		state = 'loading';
		errorMessage = '';
		audioFileName = file.name;

		try {
			const arrayBuffer = await file.arrayBuffer();
			const ctx = new AudioContext();
			audioBuffer = await ctx.decodeAudioData(arrayBuffer);
			audioDuration = audioBuffer.duration;

			// Draw waveform
			drawWaveform();

			// Extract mel spectrogram
			state = 'extracting';

			// Give the UI a frame to render the "extracting" state
			await new Promise((resolve) => requestAnimationFrame(resolve));

			let audioData = audioBuffer.getChannelData(0);

			// Resample if needed
			if (audioBuffer.sampleRate !== MODEL_PARAMS.targetSampleRate) {
				audioData = resampleLinear(
					audioData,
					audioBuffer.sampleRate,
					MODEL_PARAMS.targetSampleRate,
				);
			}

			melSpec = extractMelSpectrogram(audioData, MODEL_PARAMS.targetSampleRate, {
				nMels: MODEL_PARAMS.nMels,
				nFft: MODEL_PARAMS.nFft,
				hopLength: MODEL_PARAMS.hopLength,
			});

			modelInput = melSpectrogramToModelInput(melSpec, MODEL_PARAMS.targetFrames);
			state = 'ready';

			await ctx.close();
		} catch (e) {
			state = 'error';
			errorMessage = e instanceof Error ? e.message : 'Failed to process audio file';
		}
	}

	function triggerUpload() {
		fileInput?.click();
	}

	// ---------------------------------------------------------------------------
	// Waveform drawing
	// ---------------------------------------------------------------------------

	function drawWaveform() {
		if (!waveformCanvas || !audioBuffer) return;

		const raw = audioBuffer.getChannelData(0);
		const ctx = waveformCanvas.getContext('2d');
		if (!ctx) return;

		const dpr = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1;
		const w = waveformCanvas.clientWidth;
		const h = waveformCanvas.clientHeight;
		waveformCanvas.width = w * dpr;
		waveformCanvas.height = h * dpr;
		ctx.scale(dpr, dpr);

		// Background
		ctx.fillStyle = '#0a0a1a';
		ctx.fillRect(0, 0, w, h);

		// Downsample to peaks
		const points = 500;
		const blockSize = Math.floor(raw.length / points);
		const midY = h / 2;

		for (let i = 0; i < points; i++) {
			let max = 0;
			const start = i * blockSize;
			for (let j = 0; j < blockSize; j++) {
				const abs = Math.abs(raw[start + j] || 0);
				if (abs > max) max = abs;
			}
			const barH = max * midY * 0.9;
			const x = (i / points) * w;
			const barW = Math.max(w / points - 0.5, 1);

			ctx.fillStyle = '#2299cc66';
			ctx.fillRect(x, midY - barH, barW, barH * 2);
		}
	}

	// ---------------------------------------------------------------------------
	// Placeholder: inference (to be wired up with onnxruntime-web)
	// ---------------------------------------------------------------------------

	async function runInference() {
		if (!modelInput) return;

		state = 'inferring';

		// Simulated delay to show the UI state
		// In production, this will call onnxruntime-web inference session
		await new Promise((resolve) => setTimeout(resolve, 800));

		// Placeholder: show a "not yet connected" result
		state = 'done';
		predictions = [];
		errorMessage = 'Model inference not yet connected. Install onnxruntime-web to enable live classification.';
	}

	// ---------------------------------------------------------------------------
	// Reactive: redraw waveform when canvas resizes
	// ---------------------------------------------------------------------------

	$effect(() => {
		if (waveformCanvas && audioBuffer) {
			drawWaveform();
		}
	});
</script>

<svelte:head>
	<title>Live Playground - Contrapunk Diary</title>
</svelte:head>

<DiaryNav {crumbs} />

<div class="playground">
	<!-- Header -->
	<header class="playground-header">
		<div class="label">INTERACTIVE</div>
		<h1>Try It Yourself</h1>
		<p>
			Upload a guitar recording and see how the Pure CNN model classifies which string and
			fret produced the sound. Everything runs in your browser — no server required.
		</p>
	</header>

	<!-- Input Section -->
	<section class="input-section">
		<div class="label">INPUT</div>
		<div class="input-options">
			<button class="input-card" onclick={triggerUpload}>
				<div class="input-icon">
					<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
						<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
						<polyline points="17 8 12 3 7 8" />
						<line x1="12" y1="3" x2="12" y2="15" />
					</svg>
				</div>
				<div class="input-title">Upload Audio</div>
				<div class="input-desc">WAV, MP3, OGG, or FLAC file</div>
				{#if audioFileName}
					<div class="input-file-name">{audioFileName}</div>
				{/if}
			</button>

			<div class="input-card input-card-disabled">
				<div class="input-icon">
					<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
						<path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
						<path d="M19 10v2a7 7 0 0 1-14 0v-2" />
						<line x1="12" y1="19" x2="12" y2="23" />
						<line x1="8" y1="23" x2="16" y2="23" />
					</svg>
				</div>
				<div class="input-title">Record</div>
				<div class="input-desc">Coming soon</div>
				<div class="input-badge">FUTURE</div>
			</div>
		</div>

		<input
			bind:this={fileInput}
			type="file"
			accept="audio/*"
			onchange={handleFileUpload}
			class="hidden-input"
		/>
	</section>

	<!-- Processing Pipeline -->
	{#if state !== 'idle'}
		<section class="pipeline-section">
			<div class="label">PROCESSING PIPELINE</div>

			<div class="pipeline-steps">
				<div class="pipeline-step" class:active={state === 'loading'} class:done={state !== 'loading' && state !== 'idle'}>
					<div class="step-indicator">
						{#if state === 'loading'}
							<div class="spinner"></div>
						{:else if state !== 'idle'}
							<div class="check">&#10003;</div>
						{:else}
							<div class="step-num">1</div>
						{/if}
					</div>
					<div class="step-info">
						<div class="step-name">Decode Audio</div>
						<div class="step-detail">Web Audio API decodes to PCM samples</div>
					</div>
				</div>

				<div class="pipeline-step" class:active={state === 'extracting'} class:done={['ready', 'inferring', 'done'].includes(state)}>
					<div class="step-indicator">
						{#if state === 'extracting'}
							<div class="spinner"></div>
						{:else if ['ready', 'inferring', 'done'].includes(state)}
							<div class="check">&#10003;</div>
						{:else}
							<div class="step-num">2</div>
						{/if}
					</div>
					<div class="step-info">
						<div class="step-name">Extract Mel Spectrogram</div>
						<div class="step-detail">{MODEL_PARAMS.nMels} mel bins, {MODEL_PARAMS.nFft} FFT, {MODEL_PARAMS.hopLength} hop</div>
					</div>
				</div>

				<div class="pipeline-step" class:active={state === 'inferring'} class:done={state === 'done'}>
					<div class="step-indicator">
						{#if state === 'inferring'}
							<div class="spinner"></div>
						{:else if state === 'done'}
							<div class="check">&#10003;</div>
						{:else}
							<div class="step-num">3</div>
						{/if}
					</div>
					<div class="step-info">
						<div class="step-name">CNN Inference</div>
						<div class="step-detail">Pure CNN via ONNX Runtime WASM</div>
					</div>
				</div>
			</div>

			{#if state === 'error'}
				<div class="error-banner">
					<strong>Error:</strong> {errorMessage}
				</div>
			{/if}
		</section>
	{/if}

	<!-- Waveform Display -->
	{#if audioBuffer}
		<section class="viz-section">
			<div class="label">WAVEFORM</div>
			<div class="viz-meta">
				<span>{audioDuration.toFixed(2)}s</span>
				<span>{audioBuffer.sampleRate} Hz</span>
				<span>{audioBuffer.numberOfChannels}ch</span>
			</div>
			<div class="waveform-wrap">
				<canvas bind:this={waveformCanvas} class="waveform-canvas" style="height: 80px"></canvas>
			</div>
		</section>
	{/if}

	<!-- Spectrogram Display -->
	{#if melSpec}
		<section class="viz-section">
			<div class="label">MEL SPECTROGRAM</div>
			<div class="viz-meta">
				<span>{melSpec.length} mel bins</span>
				<span>{melSpec[0]?.length ?? 0} frames</span>
				{#if modelInput}
					<span>model input: 1x1x{MODEL_PARAMS.nMels}x{MODEL_PARAMS.targetFrames}</span>
				{/if}
			</div>
			<SpectrogramViewer data={melSpec} height={200} label="" />
		</section>
	{/if}

	<!-- Inference Button + Results -->
	{#if state === 'ready' || state === 'done'}
		<section class="results-section">
			<div class="label">CLASSIFICATION</div>

			{#if state === 'ready'}
				<button class="classify-btn" onclick={runInference}>
					Classify Guitar Position
				</button>
				<p class="classify-hint">
					Runs the Pure CNN model (424K params) via ONNX Runtime WebAssembly
				</p>
			{/if}

			{#if state === 'done'}
				{#if predictions.length > 0}
					<div class="results-grid">
						{#each predictions as pred, i}
							<div class="result-row" class:top-result={i === 0}>
								<div class="result-rank">#{i + 1}</div>
								<div class="result-label">{pred.label}</div>
								<div class="result-bar-wrap">
									<div
										class="result-bar"
										style="width: {(pred.probability * 100).toFixed(1)}%"
									></div>
								</div>
								<div class="result-pct">{(pred.probability * 100).toFixed(1)}%</div>
							</div>
						{/each}
					</div>
				{:else}
					<div class="inference-pending">
						<div class="pending-icon">
							<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
								<circle cx="12" cy="12" r="10" />
								<line x1="12" y1="8" x2="12" y2="12" />
								<line x1="12" y1="16" x2="12.01" y2="16" />
							</svg>
						</div>
						<p>{errorMessage || 'No predictions available.'}</p>
						<p class="pending-sub">
							To enable live inference, <code>onnxruntime-web</code> needs to be installed
							and the ONNX session wired up. See the research notes below.
						</p>
					</div>
				{/if}
			{/if}
		</section>
	{/if}

	<!-- How It Works -->
	<section class="explainer-section">
		<div class="label">HOW IT WORKS</div>

		<div class="explainer-grid">
			<div class="explainer-card">
				<div class="explainer-num">1</div>
				<h3>Audio Input</h3>
				<p>
					Your audio file is decoded using the
					<ConceptInline term="Web Audio API">
						The Web Audio API is a browser API that provides audio processing capabilities.
						It can decode compressed audio formats (MP3, OGG, etc.) into raw PCM samples
						that we can analyze directly in JavaScript.
					</ConceptInline>
					into raw PCM samples at the original sample rate.
				</p>
			</div>

			<div class="explainer-card">
				<div class="explainer-num">2</div>
				<h3>Mel Spectrogram</h3>
				<p>
					The audio is converted into a
					<ConceptInline term="mel spectrogram">
						A mel spectrogram is a visual representation of audio that shows how energy
						is distributed across frequency bands over time. The "mel" scale warps frequencies
						to match how humans perceive pitch — compressing high frequencies and expanding
						low ones. This is the same representation used during training.
					</ConceptInline>
					using an FFT window of {MODEL_PARAMS.nFft} samples, {MODEL_PARAMS.hopLength}-sample
					hops, and {MODEL_PARAMS.nMels} mel frequency bins. The result is a 64x94 image
					that the CNN reads like a picture.
				</p>
			</div>

			<div class="explainer-card">
				<div class="explainer-num">3</div>
				<h3>CNN Classification</h3>
				<p>
					The
					<ConceptInline term="Pure CNN">
						Our Pure CNN is a 4-layer convolutional neural network with global average pooling.
						It has 424,266 parameters and achieves 97.1% accuracy classifying guitar positions.
						The ONNX export is only ~1.7 MB, making it practical for browser inference.
					</ConceptInline>
					processes the spectrogram through 4 convolutional layers, each extracting
					higher-level features. Global average pooling condenses these into a 256-dimensional
					vector, which a final linear layer maps to 138 class probabilities.
				</p>
			</div>

			<div class="explainer-card">
				<div class="explainer-num">4</div>
				<h3>Result</h3>
				<p>
					The output is a probability distribution over all 138 positions
					(6 strings x 23 frets). The top prediction tells you which string and fret
					most likely produced the sound. The model runs entirely in your browser via
					<ConceptInline term="WebAssembly">
						WebAssembly (WASM) is a binary format that runs at near-native speed in browsers.
						ONNX Runtime compiles its inference engine to WASM, letting us run the CNN model
						without any server. The ~1.7 MB model + runtime loads once and infers in milliseconds.
					</ConceptInline>
					— no data leaves your device.
				</p>
			</div>
		</div>
	</section>

	<!-- Technical Details -->
	<section class="tech-section">
		<div class="label">TECHNICAL DETAILS</div>
		<div class="tech-grid">
			<div class="tech-item">
				<div class="tech-key">Model</div>
				<div class="tech-val">Pure CNN (4 conv + GAP + linear)</div>
			</div>
			<div class="tech-item">
				<div class="tech-key">Parameters</div>
				<div class="tech-val">424,266</div>
			</div>
			<div class="tech-item">
				<div class="tech-key">ONNX Opset</div>
				<div class="tech-val">18</div>
			</div>
			<div class="tech-item">
				<div class="tech-key">Model Size</div>
				<div class="tech-val">~1.7 MB (ONNX + data)</div>
			</div>
			<div class="tech-item">
				<div class="tech-key">Input Shape</div>
				<div class="tech-val">(1, 1, 64, 94)</div>
			</div>
			<div class="tech-item">
				<div class="tech-key">Output</div>
				<div class="tech-val">138 class logits</div>
			</div>
			<div class="tech-item">
				<div class="tech-key">Runtime</div>
				<div class="tech-val">ONNX Runtime Web (WASM)</div>
			</div>
			<div class="tech-item">
				<div class="tech-key">Accuracy</div>
				<div class="tech-val">97.1% (Round 1)</div>
			</div>
		</div>
	</section>
</div>

<style>
	/* ======================================================================
	   Layout
	   ====================================================================== */

	.playground {
		max-width: 800px;
		margin: 0 auto;
		padding-bottom: 64px;
	}

	.label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-magenta);
		letter-spacing: 2px;
		margin-bottom: 12px;
	}

	/* ======================================================================
	   Header
	   ====================================================================== */

	.playground-header {
		padding: 32px 24px 24px;
		border-bottom: 1px solid var(--color-border);
	}

	.playground-header h1 {
		font-family: var(--font-reading);
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text-primary);
		margin: 0;
	}

	.playground-header p {
		font-size: 14px;
		color: var(--color-text-secondary);
		margin-top: 8px;
		line-height: 1.7;
		max-width: 600px;
	}

	/* ======================================================================
	   Input Section
	   ====================================================================== */

	.input-section {
		padding: 24px;
		border-bottom: 1px solid var(--color-border);
	}

	.input-options {
		display: flex;
		gap: 12px;
	}

	.input-card {
		flex: 1;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 24px 16px;
		text-align: center;
		cursor: pointer;
		transition: border-color 0.15s, background 0.15s;
		font-family: inherit;
		color: inherit;
	}

	.input-card:hover {
		border-color: var(--color-accent-cyan);
		background: var(--color-widget-bg);
	}

	.input-card-disabled {
		opacity: 0.5;
		cursor: not-allowed;
		position: relative;
	}

	.input-card-disabled:hover {
		border-color: var(--color-border);
		background: var(--color-bg-panel);
	}

	.input-icon {
		color: var(--color-accent-cyan);
		margin-bottom: 8px;
	}

	.input-card-disabled .input-icon {
		color: var(--color-text-dim);
	}

	.input-title {
		font-family: var(--font-pixel);
		font-size: 12px;
		color: var(--color-text-primary);
	}

	.input-desc {
		font-size: 11px;
		color: var(--color-text-dim);
		margin-top: 4px;
	}

	.input-file-name {
		font-size: 10px;
		color: var(--color-accent-teal);
		margin-top: 8px;
		font-family: var(--font-pixel);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.input-badge {
		position: absolute;
		top: 8px;
		right: 8px;
		font-family: var(--font-pixel);
		font-size: 8px;
		color: var(--color-text-dim);
		letter-spacing: 1px;
		padding: 2px 6px;
		border: 1px solid var(--color-border);
	}

	.hidden-input {
		display: none;
	}

	/* ======================================================================
	   Pipeline Steps
	   ====================================================================== */

	.pipeline-section {
		padding: 24px;
		border-bottom: 1px solid var(--color-border);
	}

	.pipeline-steps {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.pipeline-step {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 12px;
		background: var(--color-bg-panel);
		transition: background 0.15s;
	}

	.pipeline-step:first-child {
		border-radius: 4px 4px 0 0;
	}

	.pipeline-step:last-child {
		border-radius: 0 0 4px 4px;
	}

	.pipeline-step.active {
		background: var(--color-widget-bg);
	}

	.pipeline-step.done {
		opacity: 0.7;
	}

	.step-indicator {
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.step-num {
		font-family: var(--font-pixel);
		font-size: 12px;
		color: var(--color-text-dim);
	}

	.check {
		color: var(--color-accent-teal);
		font-size: 14px;
	}

	.spinner {
		width: 16px;
		height: 16px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-accent-cyan);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.step-info {
		flex: 1;
	}

	.step-name {
		font-size: 13px;
		color: var(--color-text-primary);
	}

	.step-detail {
		font-size: 11px;
		color: var(--color-text-dim);
		margin-top: 2px;
	}

	.error-banner {
		margin-top: 12px;
		padding: 10px 14px;
		background: rgba(255, 51, 136, 0.1);
		border: 1px solid rgba(255, 51, 136, 0.3);
		font-size: 13px;
		color: var(--color-accent-magenta);
	}

	/* ======================================================================
	   Visualization Sections
	   ====================================================================== */

	.viz-section {
		padding: 24px;
		border-bottom: 1px solid var(--color-border);
	}

	.viz-meta {
		display: flex;
		gap: 16px;
		font-size: 11px;
		color: var(--color-text-dim);
		font-family: var(--font-pixel);
		margin-bottom: 8px;
	}

	.waveform-wrap {
		width: 100%;
		background: var(--color-bg-deep);
		border: 1px solid var(--color-border);
		overflow: hidden;
	}

	.waveform-canvas {
		display: block;
		width: 100%;
	}

	/* ======================================================================
	   Results Section
	   ====================================================================== */

	.results-section {
		padding: 24px;
		border-bottom: 1px solid var(--color-border);
	}

	.classify-btn {
		display: block;
		width: 100%;
		padding: 14px;
		background: var(--color-accent-cyan);
		color: var(--color-bg-deep);
		border: none;
		font-family: var(--font-pixel);
		font-size: 13px;
		letter-spacing: 1px;
		cursor: pointer;
		transition: background 0.15s;
	}

	.classify-btn:hover {
		background: var(--color-accent-teal);
	}

	.classify-hint {
		text-align: center;
		font-size: 11px;
		color: var(--color-text-dim);
		margin-top: 8px;
	}

	.results-grid {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.result-row {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 12px;
		background: var(--color-bg-panel);
	}

	.result-row.top-result {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-accent-teal);
	}

	.result-rank {
		font-family: var(--font-pixel);
		font-size: 12px;
		color: var(--color-text-dim);
		width: 24px;
	}

	.top-result .result-rank {
		color: var(--color-accent-teal);
	}

	.result-label {
		font-size: 13px;
		color: var(--color-text-primary);
		width: 120px;
		font-family: var(--font-pixel);
	}

	.result-bar-wrap {
		flex: 1;
		height: 6px;
		background: var(--color-bg-deep);
		border-radius: 3px;
		overflow: hidden;
	}

	.result-bar {
		height: 100%;
		background: var(--color-accent-cyan);
		border-radius: 3px;
		transition: width 0.3s ease;
	}

	.top-result .result-bar {
		background: var(--color-accent-teal);
	}

	.result-pct {
		font-family: var(--font-pixel);
		font-size: 11px;
		color: var(--color-text-secondary);
		width: 48px;
		text-align: right;
	}

	.inference-pending {
		text-align: center;
		padding: 24px;
		background: var(--color-bg-panel);
		border: 1px dashed var(--color-border);
	}

	.pending-icon {
		color: var(--color-accent-amber);
		margin-bottom: 8px;
	}

	.inference-pending p {
		font-size: 13px;
		color: var(--color-text-secondary);
		margin: 4px 0;
		line-height: 1.6;
	}

	.pending-sub {
		font-size: 11px !important;
		color: var(--color-text-dim) !important;
		margin-top: 8px !important;
	}

	.inference-pending code {
		font-size: 12px;
		color: var(--color-accent-cyan);
		background: var(--color-bg-deep);
		padding: 1px 4px;
	}

	/* ======================================================================
	   Explainer Section
	   ====================================================================== */

	.explainer-section {
		padding: 24px;
		border-bottom: 1px solid var(--color-border);
	}

	.explainer-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 2px;
	}

	.explainer-card {
		background: var(--color-bg-panel);
		padding: 16px;
	}

	.explainer-card:first-child {
		border-radius: 4px 0 0 0;
	}

	.explainer-card:nth-child(2) {
		border-radius: 0 4px 0 0;
	}

	.explainer-card:nth-child(3) {
		border-radius: 0 0 0 4px;
	}

	.explainer-card:last-child {
		border-radius: 0 0 4px 0;
	}

	.explainer-num {
		font-family: var(--font-pixel);
		font-size: 18px;
		color: var(--color-accent-cyan);
		margin-bottom: 4px;
	}

	.explainer-card h3 {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin: 0 0 6px;
	}

	.explainer-card p {
		font-size: 12px;
		color: var(--color-text-secondary);
		line-height: 1.7;
		margin: 0;
	}

	/* ======================================================================
	   Technical Details
	   ====================================================================== */

	.tech-section {
		padding: 24px;
	}

	.tech-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 2px;
	}

	.tech-item {
		display: flex;
		justify-content: space-between;
		padding: 8px 12px;
		background: var(--color-bg-panel);
	}

	.tech-key {
		font-size: 12px;
		color: var(--color-text-dim);
	}

	.tech-val {
		font-size: 12px;
		color: var(--color-text-primary);
		font-family: var(--font-pixel);
	}

	/* ======================================================================
	   Responsive
	   ====================================================================== */

	@media (max-width: 600px) {
		.input-options {
			flex-direction: column;
		}

		.explainer-grid {
			grid-template-columns: 1fr;
		}

		.explainer-card:first-child {
			border-radius: 4px 4px 0 0;
		}

		.explainer-card:nth-child(2) {
			border-radius: 0;
		}

		.explainer-card:nth-child(3) {
			border-radius: 0;
		}

		.explainer-card:last-child {
			border-radius: 0 0 4px 4px;
		}

		.tech-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
