<section class="oscillator" aria-labelledby="oscillator-heading">
	<header>
		<h2 id="oscillator-heading">Oscillator</h2>
		<span class="number font-code">1</span>
	</header>

	<div class="oscillator-body">
		<div class="scope" aria-label="Sine oscillator preview">
			<svg viewBox="0 0 720 220" role="img">
				<title>Sine waveform</title>
				<path class="grid" d="M0 55 H720 M0 110 H720 M0 165 H720 M90 0 V220 M180 0 V220 M270 0 V220 M360 0 V220 M450 0 V220 M540 0 V220 M630 0 V220" />
				<path class="axis" d="M0 110 H720" />
				<path class="wave" d="M0 110 C45 24 135 24 180 110 S315 196 360 110 S495 24 540 110 S675 196 720 110" />
			</svg>
		</div>

		<div class="parameter-list">
			<div><span>Waveform</span><strong>Sine</strong></div>
			<div><span>Voices</span><strong>16</strong></div>
			<div><span>Pitch</span><strong>Exact Hz</strong></div>
			<div><span>Ramp</span><strong>5 ms</strong></div>
		</div>
	</div>
</section>

<style>
	.oscillator {
		min-width: 0;
		border: 1px solid #3b3b3b;
		background: #242424;
	}
	header {
		height: 38px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 12px;
		border-bottom: 1px solid #3b3b3b;
		background: #292929;
	}
	h2 { margin: 0; font: 600 12px var(--font-ui); text-transform: uppercase; letter-spacing: .06em; color: #d0d0d0; }
	.number { color: #888; font-size: 10px; }
	.oscillator-body { padding: 12px; }
	.scope { border: 1px solid #404040; background: #171717; }
	svg { display: block; width: 100%; min-height: 220px; }
	.grid { fill: none; stroke: #242424; stroke-width: 1; }
	.axis { fill: none; stroke: #383838; stroke-width: 1; }
	.wave { fill: none; stroke: #8aaac0; stroke-width: 2; vector-effect: non-scaling-stroke; }
	.parameter-list {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		border: 1px solid #3b3b3b;
		border-top: 0;
	}
	.parameter-list div { min-width: 0; padding: 10px; border-right: 1px solid #3b3b3b; display: grid; gap: 3px; }
	.parameter-list div:last-child { border-right: 0; }
	.parameter-list span { color: #777; font: 10px var(--font-ui); }
	.parameter-list strong { color: #c8c8c8; font: 500 12px var(--font-ui); }
	@media (max-width: 600px) {
		.parameter-list { grid-template-columns: repeat(2, 1fr); }
		.parameter-list div:nth-child(2) { border-right: 0; }
		.parameter-list div:nth-child(-n + 2) { border-bottom: 1px solid #3b3b3b; }
	}
</style>
