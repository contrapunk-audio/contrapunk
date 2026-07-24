<script lang="ts">
	import OscillatorPanel from './OscillatorPanel.svelte';
	import {
		createOscillatorState,
		type OscillatorState
	} from '$lib/elixir/oscillator';

	let oscillator = $state<OscillatorState>(createOscillatorState());

	function updateOscillator(patch: Partial<OscillatorState>) {
		oscillator = { ...oscillator, ...patch };
	}

	function resetOscillator() {
		oscillator = createOscillatorState();
	}
</script>

<main class="workspace">
	<header class="masthead">
		<div>
			<p class="kicker font-code">ELIXIR / INSTRUMENT LAB</p>
			<h1>Shape one source.<br /><em>Hear the architecture.</em></h1>
		</div>
		<div class="mock-status" role="status">
			<span aria-hidden="true"></span>
			<div>
				<strong>MOCK</strong>
				<small>Local controls only · not connected to audio</small>
			</div>
		</div>
	</header>

	<div class="workspace-grid">
		<OscillatorPanel state={oscillator} onchange={updateOscillator} onreset={resetOscillator} />
		<aside aria-label="Mock scope">
			<p class="font-code">SCAFFOLD 01</p>
			<h2>One oscillator.<br />No ghosts.</h2>
			<div class="rule"></div>
			<p>
				This isolated workspace exercises Elixir's current spectral, phase, and unison
				vocabulary without pretending to control the engine.
			</p>
			<dl class="font-code">
				<div><dt>modules</dt><dd>1</dd></div>
				<div><dt>signal</dt><dd>local</dd></div>
				<div><dt>backend</dt><dd>none</dd></div>
			</dl>
		</aside>
	</div>
</main>

<style>
	.workspace {
		height: 100%;
		overflow: auto;
		box-sizing: border-box;
		padding: clamp(24px, 5vw, 72px);
		background:
			radial-gradient(circle at 78% 12%, rgba(0, 204, 170, 0.09), transparent 24rem),
			linear-gradient(120deg, rgba(255, 51, 136, 0.04), transparent 45%);
	}
	.masthead { display: flex; justify-content: space-between; align-items: flex-end; gap: 40px; max-width: 1180px; margin: 0 auto clamp(26px, 4vw, 48px); }
	.kicker { color: var(--color-accent-teal); font-size: var(--font-size-xs); letter-spacing: 0.2em; margin: 0 0 12px; }
	h1 { margin: 0; max-width: 720px; font: 600 clamp(32px, 5.8vw, 74px)/0.92 var(--font-display); letter-spacing: -0.055em; color: var(--color-text-primary); }
	h1 em { color: transparent; font-style: normal; -webkit-text-stroke: 1px var(--color-text-secondary); }
	.mock-status { display: flex; align-items: center; gap: 11px; min-width: 230px; padding: 11px 13px; border: 1px solid rgba(255, 170, 51, 0.45); background: rgba(255, 170, 51, 0.07); }
	.mock-status > span { width: 8px; height: 8px; background: var(--color-accent-amber); box-shadow: 0 0 10px rgba(255, 170, 51, 0.55); }
	.mock-status div { display: grid; gap: 2px; }
	.mock-status strong { color: var(--color-accent-amber); font: 700 var(--font-size-xs) var(--font-code); letter-spacing: 0.14em; }
	.mock-status small { color: var(--color-text-secondary); font: var(--font-size-xs) var(--font-ui); }
	.workspace-grid { display: grid; grid-template-columns: minmax(0, 1fr) 220px; gap: 18px; max-width: 1180px; margin: 0 auto; align-items: stretch; }
	aside { border: 1px solid var(--color-border); padding: 22px; background: rgba(10, 10, 26, 0.82); display: flex; flex-direction: column; }
	aside > p:first-child { color: var(--color-accent-magenta); font-size: var(--font-size-xs); letter-spacing: 0.16em; margin: 0; }
	aside h2 { margin: 18px 0; font: 600 var(--font-size-lg)/1.05 var(--font-display); letter-spacing: -0.035em; }
	aside .rule { height: 1px; background: linear-gradient(90deg, var(--color-accent-magenta), transparent); margin-bottom: 18px; }
	aside > p { color: var(--color-text-secondary); font-size: var(--font-size-sm); line-height: 1.55; }
	dl { margin: auto 0 0; font-size: var(--font-size-xs); }
	dl div { display: flex; justify-content: space-between; padding: 7px 0; border-top: 1px solid var(--color-border); }
	dt { color: var(--color-text-dim); } dd { margin: 0; color: var(--color-accent-cyan); text-transform: uppercase; }
	@media (max-width: 900px) {
		.masthead { align-items: flex-start; flex-direction: column; }
		.workspace-grid { grid-template-columns: 1fr; }
		aside { display: none; }
	}
</style>
