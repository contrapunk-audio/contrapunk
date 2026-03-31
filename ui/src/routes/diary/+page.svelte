<script lang="ts">
	import DiaryNav from '$lib/components/diary/DiaryNav.svelte';
	import StatBar from '$lib/components/diary/StatBar.svelte';
	import DemoAnimation from '$lib/components/diary/DemoAnimation.svelte';

	const chapters = [
		{ phase: '01-03', title: 'MIDI Foundation', desc: 'Input, harmony engine, multi-port output', complete: true },
		{ phase: '04-05', title: 'Network and WASM', desc: 'Server mode and browser deployment', complete: true },
		{ phase: '06', title: 'Harmony and UI', desc: 'Modal harmony, chord detection, HLD pixel art UI', complete: true },
		{ phase: '08', title: 'Machine Learning', desc: 'Guitar string+fret classifier', complete: false, active: true, href: '/diary/machine-learning' },
		{ phase: '09', title: 'Vocoder', desc: 'Real-time vocal harmonization', complete: false },
		{ phase: '10', title: 'Guitar Input', desc: 'Live audio to MIDI via ML classifier', complete: false },
	];

	const stats = [
		{ value: '96.2%', label: 'Accuracy', color: 'var(--color-accent-cyan)' },
		{ value: '2.1ms', label: 'Inference', color: 'var(--color-accent-teal)' },
		{ value: '138', label: 'Classes', color: 'var(--color-accent-magenta)' },
		{ value: '0', label: 'Dependencies', color: 'var(--color-text-primary)' },
	];
</script>

<svelte:head>
	<title>Diary - Contrapunk</title>
</svelte:head>

<DiaryNav crumbs={[{ label: 'Diary' }]} />

<div class="landing">
	<header class="hero">
		<div class="tagline-label">CONTRAPUNK</div>
		<h1>An Improvisation Companion</h1>
		<p class="subtitle">
			From raw audio to real-time MIDI. The open-source journey of building
			a tool that understands your instrument and harmonizes with you live.
		</p>
	</header>

	<div class="demo-section">
		<DemoAnimation />
	</div>

	<StatBar {stats} />

	<section class="chapters">
		<div class="section-label">THE JOURNEY</div>
		<div class="chapter-grid">
			{#each chapters as ch}
				{#if ch.href}
					<a class="chapter-card" class:active={ch.active} href={ch.href}>
						<div class="chapter-phase">{ch.phase}</div>
						<div class="chapter-title">{ch.title}</div>
						<div class="chapter-desc">{ch.desc}</div>
						{#if ch.active}
							<div class="chapter-status active">Active</div>
						{/if}
					</a>
				{:else}
					<div class="chapter-card" class:complete={ch.complete} class:future={!ch.complete}>
						<div class="chapter-phase">{ch.phase}</div>
						<div class="chapter-title">{ch.title}</div>
						<div class="chapter-desc">{ch.desc}</div>
						{#if ch.complete}
							<div class="chapter-status complete">Complete</div>
						{/if}
					</div>
				{/if}
			{/each}
		</div>
	</section>
</div>

<style>
	.landing {
		max-width: 900px;
		margin: 0 auto;
		padding-bottom: 64px;
	}
	.hero {
		padding: 48px 24px 32px;
		text-align: center;
	}
	.tagline-label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-sm);
		color: var(--color-accent-magenta);
		letter-spacing: 4px;
		margin-bottom: 16px;
	}
	h1 {
		font-family: var(--font-reading);
		font-size: 28px;
		font-weight: 700;
		color: var(--color-text-primary);
		margin: 0;
	}
	.subtitle {
		font-size: 15px;
		color: var(--color-text-secondary);
		margin-top: 12px;
		max-width: 520px;
		margin-left: auto;
		margin-right: auto;
		line-height: 1.7;
	}
	.demo-section {
		padding: 0 24px 16px;
	}
	.chapters {
		padding: 32px 24px;
	}
	.section-label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-magenta);
		letter-spacing: 2px;
		margin-bottom: 16px;
	}
	.chapter-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}
	.chapter-card {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 16px;
		text-decoration: none;
		display: block;
	}
	.chapter-card.complete {
		opacity: 0.6;
	}
	.chapter-card.future {
		opacity: 0.35;
		border-style: dashed;
	}
	.chapter-card.active {
		border-color: rgba(51, 221, 255, 0.3);
		box-shadow: 0 0 20px rgba(51, 221, 255, 0.07);
	}
	.chapter-phase {
		font-family: var(--font-pixel);
		font-size: 9px;
		color: var(--color-text-dim);
	}
	.active .chapter-phase {
		color: var(--color-accent-cyan);
	}
	.chapter-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin-top: 4px;
	}
	.chapter-desc {
		font-size: 12px;
		color: var(--color-text-secondary);
		margin-top: 4px;
	}
	.chapter-status {
		font-size: 11px;
		margin-top: 8px;
	}
	.chapter-status.complete { color: var(--color-accent-teal); }
	.chapter-status.active { color: var(--color-accent-cyan); }
</style>
