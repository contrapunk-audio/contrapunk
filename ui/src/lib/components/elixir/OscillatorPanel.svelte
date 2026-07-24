<script lang="ts">
	let { roleAware = true } = $props<{ roleAware?: boolean }>();
</script>

<section class="oscillator" aria-labelledby="oscillator-heading">
	<header>
		<div>
			<p class="eyebrow font-code">VOICE SOURCE 01</p>
			<h2 id="oscillator-heading">Sine</h2>
		</div>
		<span class="fixed font-code">FIXED</span>
	</header>

	<div class="scope" aria-label="Fixed sine oscillator preview">
		<svg viewBox="0 0 720 184" role="img">
			<title>Two cycles of the fixed sine oscillator</title>
			<defs>
				<linearGradient id="elixir-wave" x1="0" x2="1">
					<stop offset="0" stop-color="var(--color-accent-cyan)" />
					<stop offset="0.55" stop-color="var(--color-accent-teal)" />
					<stop offset="1" stop-color="var(--color-accent-magenta)" />
				</linearGradient>
			</defs>
			<path class="axis" d="M0 92 H720" />
			<path
				class="wave"
				d="M0 92 C45 14 135 14 180 92 S315 170 360 92 S495 14 540 92 S675 170 720 92"
			/>
		</svg>
		<div class="scope-readout font-code">
			<span>SINE</span>
			<span>16 VOICES</span>
			<span>5 MS DE-CLICK</span>
		</div>
	</div>

	<div class="contract">
		<div>
			<span class="font-code">PITCH</span>
			<strong>Exact host frequency</strong>
		</div>
		<div>
			<span class="font-code">DYNAMICS</span>
			<strong>MIDI velocity</strong>
		</div>
		<div>
			<span class="font-code">LEVEL</span>
			<strong>{roleAware ? 'Role-aware gain' : 'Master gain'}</strong>
		</div>
	</div>
</section>

<style>
	.oscillator {
		min-width: 0;
		border: 1px solid var(--color-border);
		background: linear-gradient(155deg, rgba(26, 24, 51, 0.96), rgba(15, 14, 26, 0.98));
		box-shadow: 0 20px 80px rgba(0, 0, 0, 0.34), inset 0 1px rgba(255, 255, 255, 0.025);
	}
	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 24px;
		padding: 18px 20px 14px;
		border-bottom: 1px solid var(--color-border);
	}
	h2 { margin: 2px 0 0; font-size: var(--font-size-lg); font-weight: 600; letter-spacing: -0.03em; }
	.eyebrow { margin: 0; color: var(--color-accent-cyan); font-size: var(--font-size-xs); letter-spacing: 0.18em; }
	.fixed { border: 1px solid rgba(51, 221, 255, 0.45); padding: 6px 9px; color: var(--color-accent-cyan); font-size: var(--font-size-xs); letter-spacing: 0.16em; }
	.scope { position: relative; margin: 18px 20px 0; border: 1px solid #242240; background: #090916; overflow: hidden; }
	.scope::before {
		content: '';
		position: absolute;
		inset: 0;
		background-image: linear-gradient(rgba(51, 221, 255, 0.055) 1px, transparent 1px), linear-gradient(90deg, rgba(51, 221, 255, 0.055) 1px, transparent 1px);
		background-size: 36px 23px;
		pointer-events: none;
	}
	svg { display: block; width: 100%; height: 184px; }
	.axis { fill: none; stroke: var(--color-border); stroke-width: 1; stroke-dasharray: 4 8; }
	.wave { fill: none; stroke: url(#elixir-wave); stroke-width: 2.5; vector-effect: non-scaling-stroke; filter: drop-shadow(0 0 7px rgba(51, 221, 255, 0.5)); }
	.scope-readout { display: flex; justify-content: space-between; gap: 12px; padding: 7px 10px; border-top: 1px solid #242240; color: var(--color-text-dim); font-size: var(--font-size-xs); }
	.contract { display: grid; grid-template-columns: repeat(3, 1fr); padding: 20px; gap: 1px; }
	.contract div { display: grid; gap: 8px; padding: 16px; border: 1px solid var(--color-border); background: rgba(9, 9, 22, 0.55); }
	.contract span { color: var(--color-accent-magenta); font-size: var(--font-size-xs); letter-spacing: 0.12em; }
	.contract strong { color: var(--color-text-secondary); font: 500 var(--font-size-sm) var(--font-ui); }
	@media (max-width: 760px) {
		.contract { grid-template-columns: 1fr; }
		.scope-readout span:last-child { display: none; }
	}
</style>
