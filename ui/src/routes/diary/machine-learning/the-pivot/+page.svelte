<script lang="ts">
	import DiaryNav from '$lib/components/diary/DiaryNav.svelte';
	import ConceptInline from '$lib/components/diary/ConceptInline.svelte';

	const crumbs = [
		{ label: 'Diary', href: '/diary' },
		{ label: 'Machine Learning', href: '/diary/machine-learning' },
		{ label: 'The Pivot' },
	];
</script>

<svelte:head>
	<title>The Pivot - Contrapunk Diary</title>
</svelte:head>

<DiaryNav {crumbs} />

<div class="pivot-page">
	<!-- Hero -->
	<header class="hero">
		<div class="section-label">CONCLUSION</div>
		<h1>The Pivot</h1>
		<p class="hero-sub">
			What five rounds of training taught us — and what a research paper changed.
		</p>
	</header>

	<!-- Station 1: The Journey So Far -->
	<section class="station">
		<div class="station-label">THE JOURNEY SO FAR</div>
		<h2>Five Rounds, One Ceiling</h2>
		<p>
			Each training round changed one variable and measured the impact. The results
			told a clear story.
		</p>

		<div class="journey-table-wrap">
			<table class="journey-table">
				<thead>
					<tr>
						<th>Round</th>
						<th>Change</th>
						<th>Pure CNN</th>
						<th>Lesson</th>
					</tr>
				</thead>
				<tbody>
					<tr>
						<td class="round-num">1</td>
						<td>Raw baseline</td>
						<td class="acc">96.2%</td>
						<td>Mel-spectrograms work</td>
					</tr>
					<tr>
						<td class="round-num">2</td>
						<td>Onset alignment</td>
						<td class="acc">96.2%</td>
						<td>Alignment doesn't matter</td>
					</tr>
					<tr class="highlight-row">
						<td class="round-num">3</td>
						<td>Quality cleanup</td>
						<td class="acc best">97.3%</td>
						<td>Clean data > more data</td>
					</tr>
					<tr>
						<td class="round-num">4</td>
						<td>Goertzel harmonics</td>
						<td class="acc">96.4%</td>
						<td>Extra features hurt</td>
					</tr>
					<tr>
						<td class="round-num">5</td>
						<td>3x augmentation</td>
						<td class="acc">96.4%</td>
						<td>More data doesn't help</td>
					</tr>
				</tbody>
			</table>
		</div>

		<p class="callout">
			The pattern was clear: nothing we tried could break through 97%.
		</p>
	</section>

	<!-- Station 2: The Ceiling -->
	<section class="station">
		<div class="station-label">THE CEILING</div>
		<h2>Why 97% Is a Wall</h2>
		<p>
			The accuracy plateau is not a training problem. It is a physics problem.
			Three factors create a hard ceiling for any spectrogram-based classifier
			working with this dataset:
		</p>
		<ul class="findings">
			<li>
				<span class="finding-icon num">1</span>
				<div>
					<strong>10 samples per class is fundamentally limiting for a 138-class problem.</strong>
					With 138 string+fret positions and only 10 examples of each, the model sees
					minimal variation per class. It cannot learn the full range of how a given
					position sounds across different playing dynamics, pick positions, and
					string ages. Any statistical classifier needs more examples to reliably
					distinguish classes that are acoustically similar.
				</div>
			</li>
			<li>
				<span class="finding-icon num">2</span>
				<div>
					<strong>Mel-spectrograms capture frequency content but lose phase information.</strong>
					The
					<ConceptInline term="mel-spectrogram">
						{#snippet children()}
							A mel-spectrogram is a visual representation of audio that shows frequency
							content over time, using the mel scale (a perceptual scale that spaces
							frequencies the way humans hear them). It captures what frequencies are
							present and how loud they are, but discards phase information — the
							timing relationships between frequency components. Phase carries subtle
							cues about the physical source of a sound.
						{/snippet}
					</ConceptInline>
					tells us what frequencies are present and how loud they are, but not the
					precise timing relationships between harmonics. Those timing relationships
					— the phase structure — carry information about the physical source of the
					sound that could help distinguish strings.
				</div>
			</li>
			<li>
				<span class="finding-icon num">3</span>
				<div>
					<strong>37 same-note-different-string pairs share the same fundamental frequency.</strong>
					A guitar can produce the same pitch on multiple strings at different frets.
					For example, the note A3 (220 Hz) can be played on the 5th fret of the low E
					string, the open A string, or the 7th fret of the D string. The fundamental
					frequency is identical — the model must rely on subtle harmonic differences
					between strings, and 10 samples per class is not enough to learn those
					differences reliably.
				</div>
			</li>
		</ul>
	</section>

	<!-- Station 3: The Research -->
	<section class="station">
		<div class="station-label">THE RESEARCH</div>
		<h2>A 2019 Paper Changed Everything</h2>
		<p>
			While investigating why our CNN plateaued on same-note pairs, we read the
			literature on guitar string identification. A paper by Hjerrild and Christensen
			(2019) described an approach based on the physics of vibrating strings — the
			<ConceptInline term="inharmonicity B coefficient">
				{#snippet children()}
					The inharmonicity coefficient B is a dimensionless number that describes
					how much the harmonics of a vibrating string deviate from perfect integer
					multiples of the fundamental. An ideal string would have harmonics at
					exactly 2x, 3x, 4x the fundamental. Real strings, because they have
					stiffness, produce harmonics that are slightly sharp — and the deviation
					grows with harmonic number. B depends on the string's physical properties:
					diameter, tension, density, and vibrating length. Each string on a guitar
					has a unique B value.
				{/snippet}
			</ConceptInline>.
		</p>

		<div class="equation-box">
			<div class="equation-label">THE INHARMONICITY MODEL</div>
			<div class="equation">
				<span class="eq-part">f<sub>n</sub></span>
				<span class="eq-op">=</span>
				<span class="eq-part">n</span>
				<span class="eq-op">&middot;</span>
				<span class="eq-part">f<sub>1</sub></span>
				<span class="eq-op">&middot;</span>
				<span class="eq-part">&radic;(1 + B &middot; n<sup>2</sup>)</span>
			</div>
			<div class="equation-legend">
				<span><em>f<sub>n</sub></em> = frequency of the nth harmonic</span>
				<span><em>n</em> = harmonic number (1, 2, 3...)</span>
				<span><em>f<sub>1</sub></em> = fundamental frequency</span>
				<span><em>B</em> = inharmonicity coefficient (unique per string)</span>
			</div>
		</div>

		<p>
			The key insight:
			<ConceptInline term="B depends on string properties, not fret position">
				{#snippet children()}
					The B coefficient is determined by the string's diameter, tension, density,
					and vibrating length. When you press a different fret, you change the
					vibrating length — but B stays roughly constant because the ratio of
					stiffness to tension doesn't change significantly. This means B identifies
					the string regardless of which fret is pressed. Once you know the pitch
					(from any standard pitch detector) and the string (from B), the fret is
					trivially computed.
				{/snippet}
			</ConceptInline>.
			Each string has a unique B regardless of fret. By measuring the deviation of
			the first 6 harmonics from perfect integer multiples of the fundamental,
			you can compute B and identify which string is vibrating — with 98.5% accuracy
			using just 1 calibration sample per string.
		</p>
		<p class="callout">
			The physics already solved this problem. We were trying to learn with neural
			networks what string theory already describes with an equation.
		</p>
	</section>

	<!-- Station 4: DSP vs ML -->
	<section class="station">
		<div class="station-label">THE COMPARISON</div>
		<h2>DSP vs ML — Side by Side</h2>
		<p>
			The numbers make the case. A physics-based approach outperforms our best
			trained model on every metric that matters for a real-time instrument.
		</p>

		<div class="compare-table-wrap">
			<table class="compare-table">
				<thead>
					<tr>
						<th>Aspect</th>
						<th>ML (our CNN)</th>
						<th class="dsp-col">DSP (inharmonicity)</th>
					</tr>
				</thead>
				<tbody>
					<tr>
						<td class="aspect-label">String+fret accuracy</td>
						<td>97.3%</td>
						<td class="dsp-col winner">98.5%</td>
					</tr>
					<tr>
						<td class="aspect-label">Training data needed</td>
						<td>1,380 samples</td>
						<td class="dsp-col winner">6 samples (1/string)</td>
					</tr>
					<tr>
						<td class="aspect-label">Latency</td>
						<td>~50ms</td>
						<td class="dsp-col winner">~40ms</td>
					</tr>
					<tr>
						<td class="aspect-label">Pitch bends</td>
						<td class="limitation">Cannot track</td>
						<td class="dsp-col winner">Frame-by-frame</td>
					</tr>
					<tr>
						<td class="aspect-label">Legato</td>
						<td class="limitation">Cannot detect</td>
						<td class="dsp-col winner">Detects pitch without onset</td>
					</tr>
				</tbody>
			</table>
		</div>
	</section>

	<!-- Station 5: The Hybrid Architecture -->
	<section class="station">
		<div class="station-label">THE PLAN</div>
		<h2>The Hybrid Architecture</h2>
		<p>
			We are not throwing away the ML work. The plan is to use both approaches
			where each excels — DSP for speed and physics-grounded accuracy, ML for
			edge-case refinement.
		</p>

		<div class="architecture-box">
			<div class="arch-label">HYBRID PIPELINE</div>
			<div class="arch-flow">
				<div class="arch-node input">Audio In</div>
				<div class="arch-split">
					<div class="arch-branch fast">
						<div class="arch-branch-label">FAST PATH (~8ms)</div>
						<div class="arch-node">Onset + Pitch (DSP)</div>
						<div class="arch-result">Instant MIDI note</div>
					</div>
					<div class="arch-branch slow">
						<div class="arch-branch-label">SLOW PATH (~50ms)</div>
						<div class="arch-node">CNN Classifier (ML)</div>
						<div class="arch-result">String confirmation</div>
					</div>
				</div>
				<div class="arch-node output">Tablature-accurate MIDI with string routing</div>
			</div>
		</div>

		<p>
			The fast DSP path detects the onset, estimates the pitch, and sends a MIDI
			note within 8ms — fast enough for a player to feel no latency. The slow ML
			path runs the CNN classifier in parallel and, 50ms later, either confirms
			the string assignment or corrects it. The result is both fast and accurate.
		</p>
	</section>

	<!-- Station 6: What the ML Journey Taught Us -->
	<section class="station">
		<div class="station-label">LESSONS</div>
		<h2>What the ML Journey Taught Us</h2>
		<ul class="learnings">
			<li>
				<strong>Data quality matters more than quantity.</strong>
				Round 3 (removing 39 bad samples) outperformed Round 5 (tripling the
				dataset through augmentation). Cleaning the data gave us +0.9%.
				Augmenting it gave us +0.0%.
			</li>
			<li>
				<strong>Simple architectures beat complex ones on small data.</strong>
				The Pure CNN consistently outperformed the Hybrid fusion model. With
				only 10 samples per class, a simpler model generalizes better — it has
				fewer parameters to overfit.
			</li>
			<li>
				<strong>The iterative approach was valuable.</strong>
				Each round revealed something specific. Round 2 told us the capture tool
				was already good. Round 3 told us data quality matters. Round 4 told us
				the CNN already learns harmonics implicitly. Round 5 confirmed the
				ceiling is physics-bounded, not data-bounded.
			</li>
			<li>
				<strong>The journey to ML led us to understand the physics.</strong>
				Investigating why Goertzel harmonics did not help the CNN led us to the
				inharmonicity literature. The failure of Round 4 pointed directly to
				the solution.
			</li>
			<li>
				<strong>Sometimes the best outcome of a machine learning project is discovering you don't need machine learning.</strong>
				Five rounds of training, 1,380 carefully captured samples, three model
				architectures — and the most important result was understanding the
				problem well enough to find a better tool. The ML journey was not wasted.
				It was the path to the answer.
			</li>
		</ul>
	</section>

	<!-- Station 7: What's Next -->
	<section class="station last">
		<div class="station-label">WHAT'S NEXT</div>
		<h2>The DSP Pipeline</h2>
		<p>
			The next chapter implements the physics-based approach: pitch detection with
			YIN autocorrelation, inharmonicity measurement from the first 6 harmonics,
			string identification via B coefficient lookup, and continuous pitch tracking
			for bends and vibrato.
		</p>
		<div class="nav-links">
			<a href="/diary/machine-learning/round-3" class="nav-link prev">
				&lt;- Round 3: Quality Cleanup
			</a>
			<a href="/diary/machine-learning" class="nav-link next">
				Back to Overview ->
			</a>
		</div>
	</section>
</div>

<style>
	.pivot-page {
		max-width: 800px;
		margin: 0 auto;
		padding-bottom: 64px;
	}

	/* Hero */
	.hero {
		padding: 32px 24px 24px;
		border-bottom: 1px solid var(--color-border);
	}
	.section-label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-amber, #ffaa33);
		letter-spacing: 3px;
		margin-bottom: 8px;
	}
	.hero h1 {
		font-family: var(--font-reading);
		font-size: 28px;
		font-weight: 700;
		color: var(--color-text-primary);
		margin: 0;
	}
	.hero-sub {
		font-size: 15px;
		color: var(--color-text-secondary);
		margin-top: 8px;
		line-height: 1.7;
		max-width: 600px;
		font-style: italic;
	}

	/* Stations */
	.station {
		padding: 24px;
		border-bottom: 1px solid var(--color-border);
	}
	.station.last {
		border-bottom: none;
	}
	.station-label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-amber, #ffaa33);
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

	/* Callout */
	.callout {
		font-family: var(--font-reading);
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text-primary);
		border-left: 3px solid var(--color-accent-amber, #ffaa33);
		padding-left: 16px;
		margin: 20px 0;
		line-height: 1.6;
	}

	/* Journey table */
	.journey-table-wrap {
		overflow-x: auto;
		margin: 16px 0;
	}
	.journey-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}
	.journey-table th {
		font-family: var(--font-pixel);
		font-size: 9px;
		color: var(--color-text-dim);
		letter-spacing: 1px;
		text-transform: uppercase;
		text-align: left;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border);
		background: var(--color-bg-panel);
	}
	.journey-table td {
		padding: 10px 12px;
		border-bottom: 1px solid var(--color-border);
		color: var(--color-text-secondary);
	}
	.journey-table .round-num {
		font-family: var(--font-pixel);
		color: var(--color-accent-cyan);
		width: 48px;
	}
	.journey-table .acc {
		font-family: var(--font-pixel);
		color: var(--color-accent-cyan);
	}
	.journey-table .acc.best {
		color: var(--color-accent-teal);
		font-weight: 700;
	}
	.journey-table .highlight-row {
		background: rgba(0, 204, 170, 0.05);
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

	/* Equation box */
	.equation-box {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 20px;
		margin: 20px 0;
		text-align: center;
	}
	.equation-label {
		font-family: var(--font-pixel);
		font-size: 9px;
		color: var(--color-text-dim);
		letter-spacing: 2px;
		margin-bottom: 12px;
	}
	.equation {
		font-family: var(--font-reading);
		font-size: 22px;
		color: var(--color-text-primary);
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	.eq-part {
		color: var(--color-accent-cyan);
	}
	.eq-op {
		color: var(--color-text-dim);
	}
	.equation-legend {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 16px;
		margin-top: 16px;
		font-size: 11px;
		color: var(--color-text-dim);
	}
	.equation-legend em {
		color: var(--color-accent-cyan);
		font-style: normal;
	}

	/* Comparison table */
	.compare-table-wrap {
		overflow-x: auto;
		margin: 16px 0;
	}
	.compare-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}
	.compare-table th {
		font-family: var(--font-pixel);
		font-size: 9px;
		color: var(--color-text-dim);
		letter-spacing: 1px;
		text-transform: uppercase;
		text-align: left;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border);
		background: var(--color-bg-panel);
	}
	.compare-table th.dsp-col {
		color: var(--color-accent-teal);
	}
	.compare-table td {
		padding: 10px 12px;
		border-bottom: 1px solid var(--color-border);
		color: var(--color-text-secondary);
	}
	.compare-table .aspect-label {
		font-weight: 600;
		color: var(--color-text-primary);
		font-size: 12px;
	}
	.compare-table .dsp-col {
		background: rgba(0, 204, 170, 0.03);
	}
	.compare-table .dsp-col.winner {
		color: var(--color-accent-teal);
		font-weight: 600;
	}
	.compare-table .limitation {
		color: var(--color-text-dim);
		font-style: italic;
	}

	/* Architecture box */
	.architecture-box {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 20px;
		margin: 20px 0;
	}
	.arch-label {
		font-family: var(--font-pixel);
		font-size: 9px;
		color: var(--color-text-dim);
		letter-spacing: 2px;
		margin-bottom: 16px;
		text-align: center;
	}
	.arch-flow {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
	}
	.arch-node {
		font-family: var(--font-reading);
		font-size: 12px;
		color: var(--color-text-primary);
		padding: 8px 16px;
		border: 1px solid var(--color-border);
		text-align: center;
		background: var(--color-bg-deep);
	}
	.arch-node.input,
	.arch-node.output {
		font-weight: 600;
		border-color: var(--color-accent-cyan-dim, rgba(51, 221, 255, 0.3));
	}
	.arch-node.output {
		border-color: var(--color-accent-teal);
		color: var(--color-accent-teal);
	}
	.arch-split {
		display: flex;
		gap: 16px;
		width: 100%;
	}
	.arch-branch {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 12px;
		border: 1px dashed var(--color-border);
	}
	.arch-branch-label {
		font-family: var(--font-pixel);
		font-size: 8px;
		letter-spacing: 1px;
	}
	.arch-branch.fast .arch-branch-label {
		color: var(--color-accent-teal);
	}
	.arch-branch.slow .arch-branch-label {
		color: var(--color-accent-cyan);
	}
	.arch-result {
		font-size: 11px;
		color: var(--color-text-dim);
		font-style: italic;
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
		font-family: var(--font-pixel);
		font-size: var(--font-size-sm);
		color: var(--color-accent-cyan);
		text-decoration: none;
		padding: 12px 20px;
		border: 1px solid var(--color-accent-cyan-dim, rgba(51, 221, 255, 0.3));
	}
	.nav-link:hover {
		background: rgba(51, 221, 255, 0.05);
		border-color: var(--color-accent-cyan);
	}
</style>
