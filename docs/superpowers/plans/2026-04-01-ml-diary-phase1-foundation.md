# ML Diary Phase 1: Foundation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restructure the diary into a multi-page HLD-themed experience with data loaded from JSON, replacing the monolithic hardcoded page.

**Architecture:** SvelteKit static adapter, Svelte 5 runes. New routes under `/diary/`. Diary layout bridges HLD pixel art theme (headers/labels) with readable sans-serif body text. All training data loaded via `fetch()` from `static/training/`. No new npm dependencies.

**Tech Stack:** SvelteKit 2, Svelte 5, Tailwind CSS v4, adapter-static. HLD design tokens from `ui/src/lib/theme/tokens.css`.

**Spec:** `docs/superpowers/specs/2026-04-01-ml-diary-redesign-design.md`

---

## File Structure

### Create
- `ui/src/lib/components/diary/DiaryNav.svelte` — sticky breadcrumb navigation
- `ui/src/lib/components/diary/ConceptInline.svelte` — inline expandable concept explanation
- `ui/src/lib/components/diary/RoundCard.svelte` — training round summary card for timeline
- `ui/src/lib/components/diary/StatBar.svelte` — horizontal stats display
- `ui/src/routes/diary/+page.svelte` — landing page (REPLACE current 404)
- `ui/src/routes/diary/machine-learning/round-1/+page.svelte` — Round 1 interactive entry
- `ui/src/routes/diary/machine-learning/round-2/+page.svelte` — Round 2 entry
- `ui/static/training/round_02/results.json` — copy from ml/training/round_02/

### Modify
- `ui/src/lib/theme/tokens.css` — add `--font-reading` variable
- `ui/src/routes/diary/+layout.svelte` — rewrite with HLD theming + DiaryNav
- `ui/src/routes/diary/machine-learning/+page.svelte` — rewrite as chapter overview

---

## Task 1: Add --font-reading to design tokens

**Files:**
- Modify: `ui/src/lib/theme/tokens.css`

- [ ] **Step 1: Add the CSS variable**

Add after the existing `--font-pixel` line in `tokens.css`:

```css
--font-reading: system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif;
```

- [ ] **Step 2: Verify build**

Run: `cd ui && npm run build`
Expected: Build succeeds with no errors.

- [ ] **Step 3: Commit**

```bash
git add ui/src/lib/theme/tokens.css
git commit -m "feat(theme): add --font-reading variable for diary body text"
```

---

## Task 2: Copy Round 2 results to static/

**Files:**
- Create: `ui/static/training/round_02/results.json`
- Create: `ui/static/training/round_02/onset_distribution.png`
- Create: `ui/static/training/round_02/before_after_spectrograms.png`
- Create: `ui/static/training/round_02/comparison_bars.png`

- [ ] **Step 1: Copy Round 2 artifacts**

```bash
mkdir -p ui/static/training/round_02
cp ml/training/round_02/results.json ui/static/training/round_02/
cp ml/training/round_02/*.png ui/static/training/round_02/
```

- [ ] **Step 2: Verify JSON loads**

```bash
cat ui/static/training/round_02/results.json | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d), 'models'); print(d[2]['name'], d[2]['accuracy'])"
```

Expected: `3 models` and `Pure CNN 0.9623...`

- [ ] **Step 3: Commit**

```bash
git add ui/static/training/round_02/
git commit -m "data: copy Round 2 onset alignment results to static/"
```

---

## Task 3: Build DiaryNav component

**Files:**
- Create: `ui/src/lib/components/diary/DiaryNav.svelte`

- [ ] **Step 1: Create the component**

```svelte
<script lang="ts">
	type Crumb = { label: string; href?: string };
	let { crumbs = [] }: { crumbs: Crumb[] } = $props();
</script>

<nav class="diary-nav">
	<a href="/" class="brand">CONTRAPUNK</a>
	{#each crumbs as crumb, i}
		<span class="sep">&gt;</span>
		{#if crumb.href && i < crumbs.length - 1}
			<a href={crumb.href} class="crumb">{crumb.label}</a>
		{:else}
			<span class="crumb active">{crumb.label}</span>
		{/if}
	{/each}
</nav>

<style>
	.diary-nav {
		position: sticky;
		top: 0;
		z-index: 50;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 16px;
		background: var(--color-bg-panel);
		border-bottom: 1px solid var(--color-border);
		font-size: 11px;
	}
	.brand {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-magenta);
		text-decoration: none;
	}
	.brand:hover {
		color: var(--color-accent-pink);
	}
	.sep {
		color: var(--color-text-dim);
	}
	.crumb {
		color: var(--color-text-secondary);
		text-decoration: none;
		font-family: var(--font-reading);
	}
	.crumb:hover {
		color: var(--color-text-primary);
	}
	.crumb.active {
		color: var(--color-accent-cyan);
	}
</style>
```

- [ ] **Step 2: Verify build**

Run: `cd ui && npm run build`
Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add ui/src/lib/components/diary/DiaryNav.svelte
git commit -m "feat(diary): add DiaryNav breadcrumb component"
```

---

## Task 4: Build ConceptInline component

**Files:**
- Create: `ui/src/lib/components/diary/ConceptInline.svelte`

- [ ] **Step 1: Create the component**

```svelte
<script lang="ts">
	let { term, children }: { term: string; children: any } = $props();
	let expanded = $state(false);
</script>

<span class="concept-trigger" onclick={() => expanded = !expanded}>
	{term}
</span>
{#if expanded}
	<div class="concept-body">
		{@render children()}
	</div>
{/if}

<style>
	.concept-trigger {
		color: var(--color-accent-cyan);
		border-bottom: 1px dashed var(--color-accent-cyan-dim);
		cursor: pointer;
	}
	.concept-trigger:hover {
		color: var(--color-accent-cyan);
		border-bottom-color: var(--color-accent-cyan);
	}
	.concept-body {
		margin: 8px 0 12px;
		padding: 12px 16px;
		background: var(--color-bg-panel);
		border-left: 2px solid var(--color-accent-cyan-dim);
		font-size: 14px;
		line-height: 1.7;
		color: var(--color-text-secondary);
	}
</style>
```

- [ ] **Step 2: Verify build**

Run: `cd ui && npm run build`
Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add ui/src/lib/components/diary/ConceptInline.svelte
git commit -m "feat(diary): add ConceptInline expandable explanation component"
```

---

## Task 5: Build StatBar and RoundCard components

**Files:**
- Create: `ui/src/lib/components/diary/StatBar.svelte`
- Create: `ui/src/lib/components/diary/RoundCard.svelte`

- [ ] **Step 1: Create StatBar**

```svelte
<script lang="ts">
	type Stat = { value: string; label: string; color?: string };
	let { stats = [] }: { stats: Stat[] } = $props();
</script>

<div class="stat-bar">
	{#each stats as stat}
		<div class="stat">
			<div class="stat-value" style:color={stat.color || 'var(--color-accent-cyan)'}>
				{stat.value}
			</div>
			<div class="stat-label">{stat.label}</div>
		</div>
	{/each}
</div>

<style>
	.stat-bar {
		display: flex;
		border-top: 1px solid var(--color-border);
		border-bottom: 1px solid var(--color-border);
	}
	.stat {
		flex: 1;
		padding: 16px;
		text-align: center;
		border-right: 1px solid var(--color-border);
	}
	.stat:last-child {
		border-right: none;
	}
	.stat-value {
		font-family: var(--font-pixel);
		font-size: 22px;
		font-weight: 700;
	}
	.stat-label {
		color: var(--color-text-dim);
		font-size: 11px;
		margin-top: 4px;
	}
</style>
```

- [ ] **Step 2: Create RoundCard**

```svelte
<script lang="ts">
	let {
		number,
		title,
		accuracy,
		delta = null,
		description,
		date,
		complete = false,
		href = '',
	}: {
		number: number;
		title: string;
		accuracy: number;
		delta?: number | null;
		description: string;
		date: string;
		complete?: boolean;
		href?: string;
	} = $props();

	function pct(n: number): string {
		return (n * 100).toFixed(1) + '%';
	}
</script>

<a class="round-card" class:complete class:active={!complete && href} {href}>
	<div class="round-number">
		<div class="circle" class:complete>
			<span>{number}</span>
		</div>
		<div class="line"></div>
	</div>
	<div class="round-body">
		<div class="round-header">
			<span class="round-title">{title}</span>
			<span class="round-accuracy">
				{pct(accuracy)}
				{#if delta !== null}
					<span class="delta" class:positive={delta > 0} class:negative={delta < 0}>
						({delta > 0 ? '+' : ''}{pct(delta)})
					</span>
				{/if}
			</span>
		</div>
		<div class="round-desc">{description}</div>
		<div class="round-meta">
			{#if complete}
				<span class="done">Complete</span>
			{:else}
				<span class="in-progress">In progress</span>
			{/if}
			<span class="date">{date}</span>
		</div>
	</div>
</a>

<style>
	.round-card {
		display: flex;
		gap: 16px;
		text-decoration: none;
		margin-bottom: 16px;
	}
	.round-number {
		flex-shrink: 0;
		width: 48px;
		text-align: center;
	}
	.circle {
		width: 40px;
		height: 40px;
		border-radius: 50%;
		background: transparent;
		border: 2px dashed var(--color-text-dim);
		display: flex;
		align-items: center;
		justify-content: center;
		margin: 0 auto;
		font-family: var(--font-pixel);
		font-size: 16px;
		color: var(--color-text-dim);
	}
	.circle.complete {
		border-style: solid;
		border-color: var(--color-accent-cyan);
		background: rgba(51, 221, 255, 0.1);
		color: var(--color-accent-cyan);
	}
	.line {
		width: 2px;
		height: 24px;
		margin: 4px auto;
		background: var(--color-border);
	}
	.round-body {
		flex: 1;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		border-radius: 0;
		padding: 16px;
	}
	.complete .round-body {
		border-color: rgba(51, 221, 255, 0.3);
	}
	.round-header {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
	}
	.round-title {
		font-family: var(--font-reading);
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text-primary);
	}
	.round-accuracy {
		font-family: var(--font-pixel);
		font-size: 16px;
		color: var(--color-accent-cyan);
	}
	.delta {
		font-size: 11px;
		color: var(--color-text-dim);
	}
	.delta.positive { color: var(--color-accent-teal); }
	.delta.negative { color: var(--color-accent-magenta); }
	.round-desc {
		color: var(--color-text-secondary);
		font-size: 13px;
		margin-top: 4px;
	}
	.round-meta {
		font-size: 11px;
		margin-top: 8px;
	}
	.done { color: var(--color-accent-teal); }
	.in-progress { color: var(--color-accent-cyan); }
	.date { color: var(--color-text-dim); margin-left: 8px; }
</style>
```

- [ ] **Step 3: Verify build**

Run: `cd ui && npm run build`
Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add ui/src/lib/components/diary/StatBar.svelte ui/src/lib/components/diary/RoundCard.svelte
git commit -m "feat(diary): add StatBar and RoundCard components"
```

---

## Task 6: Rewrite diary layout with HLD theming

**Files:**
- Modify: `ui/src/routes/diary/+layout.svelte`

- [ ] **Step 1: Replace the layout**

Replace the entire contents of `ui/src/routes/diary/+layout.svelte` with:

```svelte
<script lang="ts">
	let { children } = $props();
</script>

<div class="diary-shell">
	{@render children()}
</div>

<style>
	.diary-shell {
		position: fixed;
		inset: 0;
		z-index: 100;
		overflow-y: auto;
		background: var(--color-bg-deep);
		font-family: var(--font-reading);
		color: var(--color-text-primary);
		-webkit-font-smoothing: antialiased;
		-moz-osx-font-smoothing: grayscale;
	}
</style>
```

Key changes: `background: #030712` becomes `background: var(--color-bg-deep)`. Font uses `var(--font-reading)`. Text color uses `var(--color-text-primary)`.

- [ ] **Step 2: Verify build**

Run: `cd ui && npm run build`
Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add ui/src/routes/diary/+layout.svelte
git commit -m "feat(diary): rewrite layout with HLD design tokens"
```

---

## Task 7: Create /diary landing page

**Files:**
- Create: `ui/src/routes/diary/+page.svelte`

- [ ] **Step 1: Create the landing page**

```svelte
<script lang="ts">
	import DiaryNav from '$lib/components/diary/DiaryNav.svelte';
	import StatBar from '$lib/components/diary/StatBar.svelte';

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
```

- [ ] **Step 2: Verify build and route**

Run: `cd ui && npm run build`
Expected: Build succeeds. `build/diary/index.html` exists.

- [ ] **Step 3: Commit**

```bash
git add ui/src/routes/diary/+page.svelte
git commit -m "feat(diary): create landing page with chapter grid"
```

---

## Task 8: Rewrite ML chapter overview page

**Files:**
- Modify: `ui/src/routes/diary/machine-learning/+page.svelte`

- [ ] **Step 1: Replace with data-driven chapter overview**

Replace the entire 578-line file with a chapter overview that loads data from JSON. This page shows the iterative approach, progress chart placeholder, and round timeline with links to individual round pages.

```svelte
<script lang="ts">
	import DiaryNav from '$lib/components/diary/DiaryNav.svelte';
	import RoundCard from '$lib/components/diary/RoundCard.svelte';

	type ModelResult = {
		name: string;
		accuracy: number;
		train_time?: number;
		per_string: number[];
		notes: string;
	};

	let r1Results: ModelResult[] = $state([]);
	let r2Results: ModelResult[] = $state([]);
	let loading = $state(true);

	$effect(() => {
		Promise.all([
			fetch('/training/round_01/results.json').then(r => r.json()),
			fetch('/training/round_02/results.json').then(r => r.json()),
		]).then(([r1, r2]) => {
			r1Results = r1;
			r2Results = r2;
			loading = false;
		}).catch(() => { loading = false; });
	});

	function bestAccuracy(results: ModelResult[]): number {
		if (!results.length) return 0;
		return Math.max(...results.map(r => r.accuracy));
	}

	function pct(n: number): string {
		return (n * 100).toFixed(1) + '%';
	}

	const crumbs = [
		{ label: 'Diary', href: '/diary' },
		{ label: 'Machine Learning' },
	];
</script>

<svelte:head>
	<title>Machine Learning - Contrapunk Diary</title>
</svelte:head>

<DiaryNav {crumbs} />

<div class="chapter">
	<header class="chapter-header">
		<div class="label">CHAPTER</div>
		<h1>Teaching a Model to Identify Guitar Positions</h1>
		<p>
			138 positions on a guitar neck. 1,380 audio samples. Three model architectures.
			We train iteratively — changing one thing at a time, measuring the impact, documenting everything.
		</p>
	</header>

	<section class="approach">
		<div class="label">THE APPROACH</div>
		<div class="steps">
			<div class="step"><div class="step-num">1</div><div class="step-text">Train on raw data</div><div class="step-sub">Establish baseline</div></div>
			<div class="step"><div class="step-num">2</div><div class="step-text">Change one thing</div><div class="step-sub">Measure impact</div></div>
			<div class="step"><div class="step-num">3</div><div class="step-text">Document result</div><div class="step-sub">Learn what matters</div></div>
			<div class="step"><div class="step-num">4</div><div class="step-text">Repeat</div><div class="step-sub">Until production-ready</div></div>
		</div>
	</section>

	<section class="rounds">
		<div class="label">TRAINING ROUNDS</div>

		{#if loading}
			<p class="loading-text">Loading results...</p>
		{:else}
			<RoundCard
				number={1}
				title="Raw Baseline"
				accuracy={bestAccuracy(r1Results)}
				description="No preprocessing. Raw mel-spectrograms into 3 classifiers. Pure CNN wins."
				date="Mar 31, 2026"
				complete={true}
				href="/diary/machine-learning/round-1"
			/>

			<RoundCard
				number={2}
				title="Onset Alignment"
				accuracy={bestAccuracy(r2Results)}
				delta={bestAccuracy(r2Results) - bestAccuracy(r1Results)}
				description="Aligned all samples to pluck onset. No accuracy change — capture tool was already triggering well."
				date="Apr 1, 2026"
				complete={true}
				href="/diary/machine-learning/round-2"
			/>

			<RoundCard
				number={3}
				title="Quality Cleanup"
				accuracy={0}
				description="Remove 32 clipped + 7 silent samples."
				date=""
				complete={false}
				href=""
			/>

			<RoundCard
				number={4}
				title="Goertzel Harmonics"
				accuracy={0}
				description="Physics-based harmonic ratio features for string identification."
				date=""
				complete={false}
				href=""
			/>
		{/if}
	</section>

	<section class="tools">
		<a class="tool-card" href="/diary/machine-learning/explore" style="border-color: rgba(255, 51, 136, 0.3);">
			<div class="tool-label" style="color: var(--color-accent-magenta);">EXPLORE DATA</div>
			<div class="tool-desc">Browse 138 classes, hear samples</div>
		</a>
		<a class="tool-card" href="/diary/machine-learning/playground" style="border-color: rgba(0, 204, 170, 0.3);">
			<div class="tool-label" style="color: var(--color-accent-teal);">LIVE PLAYGROUND</div>
			<div class="tool-desc">Try the model in your browser</div>
		</a>
	</section>
</div>

<style>
	.chapter {
		max-width: 800px;
		margin: 0 auto;
		padding-bottom: 64px;
	}
	.chapter-header {
		padding: 32px 24px 24px;
		border-bottom: 1px solid var(--color-border);
	}
	.label {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-magenta);
		letter-spacing: 2px;
		margin-bottom: 12px;
	}
	h1 {
		font-family: var(--font-reading);
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text-primary);
		margin: 0;
	}
	.chapter-header p {
		font-size: 14px;
		color: var(--color-text-secondary);
		margin-top: 8px;
		line-height: 1.7;
		max-width: 600px;
	}
	.approach {
		padding: 24px;
		border-bottom: 1px solid var(--color-border);
	}
	.steps {
		display: flex;
		gap: 2px;
	}
	.step {
		flex: 1;
		background: var(--color-bg-panel);
		padding: 12px;
		text-align: center;
	}
	.step:first-child { border-radius: 4px 0 0 4px; }
	.step:last-child { border-radius: 0 4px 4px 0; }
	.step-num {
		font-family: var(--font-pixel);
		font-size: 18px;
		color: var(--color-accent-cyan);
	}
	.step-text {
		font-size: 12px;
		color: var(--color-text-primary);
		margin-top: 4px;
	}
	.step-sub {
		font-size: 10px;
		color: var(--color-text-dim);
		margin-top: 2px;
	}
	.rounds {
		padding: 24px;
	}
	.loading-text {
		color: var(--color-text-dim);
		font-size: 14px;
	}
	.tools {
		padding: 0 24px 24px;
		display: flex;
		gap: 12px;
	}
	.tool-card {
		flex: 1;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 16px;
		text-align: center;
		text-decoration: none;
	}
	.tool-card:hover {
		background: var(--color-widget-bg);
	}
	.tool-label {
		font-family: var(--font-pixel);
		font-size: 11px;
	}
	.tool-desc {
		color: var(--color-text-secondary);
		font-size: 11px;
		margin-top: 4px;
	}
</style>
```

- [ ] **Step 2: Verify build**

Run: `cd ui && npm run build`
Expected: Build succeeds. No Tailwind gray classes remain.

- [ ] **Step 3: Commit**

```bash
git add ui/src/routes/diary/machine-learning/+page.svelte
git commit -m "feat(diary): rewrite ML chapter overview with data-driven rounds"
```

---

## Task 9: Create Round 1 page

**Files:**
- Create: `ui/src/routes/diary/machine-learning/round-1/+page.svelte`

- [ ] **Step 1: Create the Round 1 page**

This page loads results from JSON and presents the narrative with station sections. For Phase 1, stations use text + static images. Interactive Canvas/SVG visualizations come in later phases.

The file is long (~300 lines). Key sections:
- Load data from `/training/round_01/results.json` via fetch
- Hero with round number, title, accuracy callouts
- Station 1: "Hear the Data" — placeholder text about A/B comparison (audio comes Phase 2)
- Station 2: "The Models" — architecture descriptions with monospace pre blocks (SVG diagrams come Phase 5)
- Station 3: "The Results" — per-string accuracy bars rendered from JSON data, model selector
- Station 4: "What We Learned" — bullet points from the training results

Per-string accuracy bars use inline SVG rects computed from `results.per_string` array values. Model selector uses `$state` to toggle between models.

Create this file at `ui/src/routes/diary/machine-learning/round-1/+page.svelte`. Full implementation should follow the mockup from the brainstorming session (pixel font headers, system font body, HLD colors, data from fetch).

- [ ] **Step 2: Verify build and route**

Run: `cd ui && npm run build`
Expected: Build succeeds. `build/diary/machine-learning/round-1/index.html` exists.

- [ ] **Step 3: Commit**

```bash
git add ui/src/routes/diary/machine-learning/round-1/+page.svelte
git commit -m "feat(diary): create Round 1 narrative page with data from JSON"
```

---

## Task 10: Create Round 2 page

**Files:**
- Create: `ui/src/routes/diary/machine-learning/round-2/+page.svelte`

- [ ] **Step 1: Create the Round 2 page**

Same structure as Round 1 but loads both `/training/round_01/results.json` and `/training/round_02/results.json`. Shows the round comparison banner (Round 1 accuracy -> Round 2 accuracy + delta + what changed). Embeds the onset distribution and before/after spectrogram images from `static/training/round_02/`.

Key difference from Round 1: the "What We Learned" station explains why onset alignment had zero impact (median onset was 21.8ms, capture tool was already triggering near the pluck, CNN pooling layers provide translation invariance).

- [ ] **Step 2: Verify build**

Run: `cd ui && npm run build`
Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add ui/src/routes/diary/machine-learning/round-2/+page.svelte
git commit -m "feat(diary): create Round 2 page with comparison to Round 1"
```

---

## Task 11: Build, preview, and deploy

**Files:** None new — integration verification.

- [ ] **Step 1: Full build**

```bash
cd ui && npm run build
```

Expected: All routes build. Check `build/diary/index.html`, `build/diary/machine-learning/index.html`, `build/diary/machine-learning/round-1/index.html`, `build/diary/machine-learning/round-2/index.html` all exist.

- [ ] **Step 2: Local preview**

```bash
cd ui && npm run preview
```

Navigate to:
- `http://localhost:4173/diary` — landing page loads, chapter grid visible, ML card links work
- `http://localhost:4173/diary/machine-learning` — overview loads, round cards visible with data from JSON
- `http://localhost:4173/diary/machine-learning/round-1` — narrative page loads, per-string bars render
- `http://localhost:4173/diary/machine-learning/round-2` — comparison banner shows, onset images load

- [ ] **Step 3: Deploy to Fly.io**

```bash
cd /Users/vibhavbobade/go/src/github.com/waveywaves/contrapunk
flyctl deploy -c fly.toml
```

Verify at `https://contrapunk.fly.dev/diary`.

- [ ] **Step 4: Commit any fixes and final commit**

```bash
git add -A
git commit -m "feat(diary): Phase 1 Foundation complete — multi-page HLD-themed diary"
```
