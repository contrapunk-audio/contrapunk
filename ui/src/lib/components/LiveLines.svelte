<script lang="ts">
	import { onMount } from 'svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { ui } from '$lib/stores/ui.svelte';

	type Source = 'player' | 'harmony' | 'canon' | 'counterpoint';
	type Filter = 'all' | Source;

	interface Segment {
		id: number;
		source: Source;
		note: number;
		startedAt: number;
		endedAt: number | null;
	}

	const WINDOW_MS = 8_000;
	const MAX_SEGMENTS = 512;
	const DEFAULT_MIN_NOTE = 48;
	const DEFAULT_MAX_NOTE = 84;
	const LEFT = 54;
	const WIDTH = 946;
	const HEIGHT = 236;
	const VERTICAL_HEIGHT = 360;
	const VERTICAL_TOP = 24;
	const sources: Array<{ id: Source; label: string; color: string }> = [
		{ id: 'player', label: 'YOU', color: '#4fe8c3' },
		{ id: 'harmony', label: 'HARMONY', color: '#ff2e88' },
		{ id: 'canon', label: 'CANON', color: '#ffdd44' },
		{ id: 'counterpoint', label: 'COUNTERPOINT', color: '#a3e635' }
	];
	const sourceOffset: Record<Source, number> = {
		player: -3,
		harmony: -1,
		canon: 1,
		counterpoint: 3
	};

	let { filter = $bindable<Filter>('all') }: { filter?: Filter } = $props();
	let orientation = $state<'horizontal' | 'vertical'>('horizontal');
	let now = $state(Date.now());
	let segments = $state<Segment[]>([]);
	let nextId = 1;
	let segmentState: Segment[] = [];
	const activeIds = new Map<string, number>();

	onMount(() => {
		try {
			const saved = localStorage.getItem('contrapunk-live-lines-orientation');
			if (saved === 'horizontal' || saved === 'vertical') orientation = saved;
		} catch {
			/* localStorage unavailable */
		}
	});

	function setOrientation(next: 'horizontal' | 'vertical') {
		orientation = next;
		try {
			localStorage.setItem('contrapunk-live-lines-orientation', next);
		} catch {
			/* localStorage unavailable */
		}
	}

	function isBlackKey(note: number): boolean {
		return [1, 3, 6, 8, 10].includes(note % 12);
	}

	function liveKeyColor(note: number): string {
		if (engine.inputNotes.includes(note)) return '#4fe8c3';
		if (engine.canonNotes.includes(note)) return '#ffdd44';
		if (engine.counterpointNotes.includes(note)) return '#a3e635';
		if (engine.harmonyNotes.includes(note)) return '#ff2e88';
		return isBlackKey(note) ? '#17141f' : '#efe8d5';
	}

	function noteName(note: number): string {
		const names = ['C', 'C♯', 'D', 'D♯', 'E', 'F', 'F♯', 'G', 'G♯', 'A', 'A♯', 'B'];
		return `${names[note % 12]}${Math.floor(note / 12) - 1}`;
	}

	function syncSource(source: Source, notes: number[], timestamp: number) {
		const current = new Set(notes);
		for (const [key, id] of [...activeIds]) {
			if (!key.startsWith(`${source}:`)) continue;
			const note = Number(key.slice(source.length + 1));
			if (current.has(note)) continue;
			const segment = segmentState.find((item) => item.id === id);
			if (segment) segment.endedAt = timestamp;
			activeIds.delete(key);
		}
		for (const note of current) {
			const key = `${source}:${note}`;
			if (activeIds.has(key)) continue;
			const segment: Segment = {
				id: nextId++,
				source,
				note,
				startedAt: timestamp,
				endedAt: null
			};
			segmentState.push(segment);
			activeIds.set(key, segment.id);
		}
	}

	$effect(() => {
		const canon = engine.canonNotes;
		const counterpoint = engine.counterpointNotes;
		const harmony = engine.harmonyNotes;
		const timestamp = Date.now();
		now = timestamp;
		syncSource('player', engine.inputNotes, timestamp);
		syncSource('harmony', harmony, timestamp);
		syncSource('canon', canon, timestamp);
		syncSource('counterpoint', counterpoint, timestamp);
		segmentState = segmentState.filter(
			(segment) => segment.endedAt === null || segment.endedAt >= timestamp - WINDOW_MS
		);
		if (segmentState.length > MAX_SEGMENTS) {
			const active = segmentState.filter((segment) => segment.endedAt === null);
			const closed = segmentState.filter((segment) => segment.endedAt !== null);
			segmentState = [...closed.slice(-(MAX_SEGMENTS - active.length)), ...active]
				.sort((a, b) => a.startedAt - b.startedAt);
		}
		segments = [...segmentState];
	});

	$effect(() => {
		if (ui.reducedMotion || typeof requestAnimationFrame !== 'function') return;
		let frame = 0;
		let lastUpdate = 0;
		const tick = (timestamp: number) => {
			if (timestamp - lastUpdate >= 33) {
				now = Date.now();
				lastUpdate = timestamp;
			}
			frame = requestAnimationFrame(tick);
		};
		frame = requestAnimationFrame(tick);
		return () => cancelAnimationFrame(frame);
	});

	let recentSegments = $derived(
		segments.filter((segment) => (segment.endedAt ?? now) >= now - WINDOW_MS)
	);
	let visibleSegments = $derived(
		recentSegments.filter((segment) => filter === 'all' || segment.source === filter)
	);
	let pitchRange = $derived.by(() => {
		if (recentSegments.length === 0) {
			return { min: DEFAULT_MIN_NOTE, max: DEFAULT_MAX_NOTE };
		}
		const notes = recentSegments.map((segment) => segment.note);
		let min = Math.floor((Math.min(...notes) - 3) / 12) * 12;
		let max = Math.ceil((Math.max(...notes) + 3) / 12) * 12;
		if (max - min < 36) {
			const center = (max + min) / 2;
			min = Math.floor((center - 18) / 12) * 12;
			max = min + 36;
		}
		return { min: Math.max(0, min), max: Math.min(127, max) };
	});
	let octaveLines = $derived(
		Array.from(
			{ length: Math.floor((pitchRange.max - pitchRange.min) / 12) + 1 },
			(_, index) => pitchRange.min + index * 12
		)
	);
	let pitchKeys = $derived(
		Array.from({ length: pitchRange.max - pitchRange.min + 1 }, (_, index) => pitchRange.min + index)
	);
	let hasVisibleNotes = $derived(visibleSegments.length > 0);
	let activeSummary = $derived(
		[
			engine.inputNotes.length ? `You ${engine.inputNotes.map(noteName).join(' ')}` : '',
			engine.harmonyNotes.length ? `Harmony ${engine.harmonyNotes.map(noteName).join(' ')}` : '',
			engine.canonNotes.length ? `Canon ${engine.canonNotes.map(noteName).join(' ')}` : '',
			engine.counterpointNotes.length ? `Counterpoint ${engine.counterpointNotes.map(noteName).join(' ')}` : ''
		].filter(Boolean).join('; ') || 'No notes sounding'
	);

	let chartHeight = $derived(orientation === 'vertical' ? VERTICAL_HEIGHT : HEIGHT);

	function timeFor(timestamp: number): number {
		const progress = (timestamp - (now - WINDOW_MS)) / WINDOW_MS;
		return orientation === 'vertical'
			? VERTICAL_TOP + progress * (VERTICAL_HEIGHT - VERTICAL_TOP)
			: LEFT + progress * WIDTH;
	}

	function pitchCoordinate(note: number): number {
		const progress = (note - pitchRange.min) / (pitchRange.max - pitchRange.min);
		return orientation === 'vertical'
			? LEFT + progress * (WIDTH - 12)
			: (1 - progress) * (HEIGHT - 16) + 8;
	}

	function pitchFor(note: number, source: Source): number {
		return pitchCoordinate(note) + sourceOffset[source] * 0.35;
	}

	function durationFor(segment: Segment): number {
		const start = Math.max(segment.startedAt, now - WINDOW_MS);
		const span = orientation === 'vertical' ? VERTICAL_HEIGHT - VERTICAL_TOP : WIDTH;
		return Math.max(3, ((Math.min(segment.endedAt ?? now, now) - start) / WINDOW_MS) * span);
	}

	function pathFor(source: Source): string {
		const sourceSegments = visibleSegments.filter((segment) => segment.source === source);
		if (sourceSegments.length === 0) return '';
		const first = sourceSegments[0];
		let previousEnd = timeFor(Math.min(first.endedAt ?? now, now));
		let previousPitch = pitchFor(first.note, source);
		const firstStart = Math.max(orientation === 'vertical' ? VERTICAL_TOP : LEFT, timeFor(first.startedAt));
		let path = orientation === 'vertical'
			? `M ${previousPitch} ${firstStart} L ${previousPitch} ${previousEnd}`
			: `M ${firstStart} ${previousPitch} L ${previousEnd} ${previousPitch}`;
		for (const segment of sourceSegments.slice(1)) {
			const start = Math.max(orientation === 'vertical' ? VERTICAL_TOP : LEFT, timeFor(segment.startedAt));
			const end = timeFor(Math.min(segment.endedAt ?? now, now));
			const pitch = pitchFor(segment.note, source);
			if (start < previousEnd) {
				path += orientation === 'vertical'
					? ` M ${pitch} ${start} L ${pitch} ${end}`
					: ` M ${start} ${pitch} L ${end} ${pitch}`;
				previousEnd = end;
				previousPitch = pitch;
				continue;
			}
			const gap = start - previousEnd;
			if (gap > 0) {
				path += orientation === 'vertical'
					? ` C ${previousPitch} ${previousEnd + gap * 0.35}, ${pitch} ${start - gap * 0.35}, ${pitch} ${start}`
					: ` C ${previousEnd + gap * 0.35} ${previousPitch}, ${start - gap * 0.35} ${pitch}, ${start} ${pitch}`;
			} else {
				path += orientation === 'vertical' ? ` L ${pitch} ${start}` : ` L ${start} ${pitch}`;
			}
			path += orientation === 'vertical' ? ` L ${pitch} ${end}` : ` L ${end} ${pitch}`;
			previousEnd = end;
			previousPitch = pitch;
		}
		return path;
	}

	function emptyMessage(): string {
		if (filter === 'all') return 'Play a note. Your melody will appear in teal and the ensemble will answer in color.';
		const label = sources.find((source) => source.id === filter)?.label.toLowerCase() ?? 'selected';
		return `No ${label} notes in the last 8 seconds.`;
	}
</script>

<div class="live-lines">
	<div class="toolbar">
		<div>
			<strong class="font-ui">LIVE LINES</strong>
			<span class="description font-ui">{orientation === 'horizontal' ? 'Vertical piano rail · time moves right' : 'Pitch across · time moves down'} · matching shapes reveal relationships</span>
		</div>
		<div class="orientation" role="group" aria-label="Live Lines orientation">
			<button class:active={orientation === 'horizontal'} aria-pressed={orientation === 'horizontal'} type="button" title="Places pitch vertically with an aligned piano rail while time moves left to right." onclick={() => setOrientation('horizontal')}>TIME →</button>
			<button class:active={orientation === 'vertical'} aria-pressed={orientation === 'vertical'} type="button" title="Places pitch horizontally with an aligned piano rail while time moves downward." onclick={() => setOrientation('vertical')}>TIME ↓</button>
		</div>
		<div class="filters" role="group" aria-label="Visible ensemble parts">
			<button class:active={filter === 'all'} aria-pressed={filter === 'all'} type="button" title="Shows the player and every generated ensemble part together." onclick={() => (filter = 'all')}>ALL</button>
			{#each sources as source}
				<button
					class:active={filter === source.id}
					style:--source-color={source.color}
					aria-pressed={filter === source.id}
					title={`Shows only ${source.label.toLowerCase()} notes in the recent performance.`}
					type="button"
					onclick={() => (filter = source.id)}
				>
					{source.label}
				</button>
			{/each}
		</div>
	</div>

	<div class="roll" class:vertical={orientation === 'vertical'} style:height={`${chartHeight}px`}>
		<svg viewBox={`0 0 1000 ${chartHeight}`} preserveAspectRatio="none" role="img" aria-label={`Recent player and ensemble notes with time moving ${orientation === 'vertical' ? 'top to bottom' : 'left to right'}`}>
			<title>Live Lines: recent notes moving {orientation === 'vertical' ? 'top to bottom' : 'left to right'}</title>
			<rect width="1000" height={chartHeight} class="roll-bg" />
			{#if orientation === 'horizontal'}
				{@const keyHeight = (HEIGHT - 16) / (pitchRange.max - pitchRange.min)}
				{#each pitchKeys as note}
					<rect
						x="0"
						y={pitchCoordinate(note) - keyHeight / 2}
						width={isBlackKey(note) ? 38 : LEFT}
						height={Math.max(4, keyHeight)}
						fill={liveKeyColor(note)}
						class="piano-rail-key"
					/>
				{/each}
			{:else}
				{@const keyWidth = (WIDTH - 12) / (pitchRange.max - pitchRange.min)}
				{#each pitchKeys as note}
					<rect
						x={pitchCoordinate(note) - keyWidth / 2}
						y="0"
						width={Math.max(4, keyWidth)}
						height={isBlackKey(note) ? 15 : VERTICAL_TOP}
						fill={liveKeyColor(note)}
						class="piano-rail-key"
					/>
				{/each}
			{/if}
			{#each octaveLines as note}
				{@const pitch = pitchCoordinate(note)}
				{#if orientation === 'vertical'}
					<line x1={pitch} x2={pitch} y1={VERTICAL_TOP} y2={chartHeight} class="octave-line" />
					<text x={pitch} y="14" text-anchor="middle" class="pitch-label">{noteName(note)}</text>
				{:else}
					<line x1={LEFT} x2="1000" y1={pitch} y2={pitch} class="octave-line" />
					<text x="7" y={pitch + 3} class="pitch-label">{noteName(note)}</text>
				{/if}
			{/each}
			{#each [0, 1, 2, 3, 4, 5, 6, 7, 8] as step}
				{#if orientation === 'vertical'}
					{@const y = VERTICAL_TOP + (step / 8) * (VERTICAL_HEIGHT - VERTICAL_TOP)}
					<line x1={LEFT} x2="1000" y1={y} y2={y} class:now-line={step === 8} class="time-line" />
				{:else}
					{@const x = LEFT + (step / 8) * WIDTH}
					<line x1={x} x2={x} y1="0" y2={chartHeight} class:now-line={step === 8} class="time-line" />
				{/if}
			{/each}
			{#each sources.filter((source) => source.id !== 'harmony') as source}
				{@const path = pathFor(source.id)}
				{#if path}
					<path d={path} fill="none" stroke={source.color} class="contour" />
				{/if}
			{/each}
			{#each visibleSegments as segment (segment.id)}
				{@const source = sources.find((item) => item.id === segment.source)!}
				<rect
					x={orientation === 'vertical' ? pitchFor(segment.note, segment.source) - 4 : Math.max(LEFT, timeFor(segment.startedAt))}
					y={orientation === 'vertical' ? Math.max(VERTICAL_TOP, timeFor(segment.startedAt)) : pitchFor(segment.note, segment.source) - 4}
					width={orientation === 'vertical' ? 8 : durationFor(segment)}
					height={orientation === 'vertical' ? durationFor(segment) : 8}
					fill={source.color}
					stroke="#f5e9c9"
					stroke-opacity="0.72"
					stroke-width="1"
					class="note"
				>
					<title>{source.label}: {noteName(segment.note)}</title>
				</rect>
			{/each}
		</svg>
		{#if !hasVisibleNotes}
			<div class="empty font-ui">{emptyMessage()}</div>
		{/if}
		<p class="sr-only">{activeSummary}</p>
		{#if orientation === 'vertical'}
			<div class="axis-help vertical-axis font-ui"><span>EARLIER</span><b>TIME ↓</b><span>NOW</span></div>
		{:else}
			<div class="axis-help font-ui">← EARLIER <span>TIME</span> NOW →</div>
		{/if}
	</div>

	<div class="plain-language font-ui">
		{#if filter === 'all'}
			<strong>ALL PARTS:</strong> compare their note shapes to see who supports, copies, or moves independently.
		{:else if filter === 'player'}
			<strong>YOU:</strong> teal is the melody you perform. It is the source the ensemble hears.
		{:else if filter === 'harmony'}
			<strong>HARMONIC SUPPORT:</strong> magenta notes arrive with your notes to create chordal support.
		{:else if filter === 'canon'}
			<strong>CANON:</strong> gold is the delayed answering voice, making imitation visible as it follows your melody.
		{:else}
			<strong>COUNTERPOINT:</strong> lime moves as an independent line against the melody you play.
		{/if}
	</div>
</div>

<style>
	.live-lines { background: var(--color-bg-deep); }
	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 5px 7px;
		border-bottom: 1px solid var(--color-border);
		background: rgba(26, 24, 51, 0.82);
	}
	.toolbar > div:first-child { display: flex; align-items: baseline; gap: 8px; min-width: 0; }
	.toolbar strong { color: var(--color-accent-gold); font-size: 9px; letter-spacing: 1px; white-space: nowrap; }
	.description { color: var(--color-text-dim); font-size: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.orientation, .filters { display: flex; gap: 2px; flex-shrink: 0; }
	.orientation { margin-left: auto; }
	.orientation button, .filters button {
		min-height: 24px;
		padding: 4px 7px;
		border: 1px solid var(--color-border);
		border-radius: 0;
		background: var(--color-widget-bg);
		color: var(--color-text-dim);
		font: 8px/1 var(--font-ui);
		cursor: pointer;
	}
	.orientation button.active, .filters button.active {
		border-color: var(--source-color, var(--color-accent-cyan));
		color: var(--source-color, var(--color-accent-cyan));
		box-shadow: inset 0 -2px 0 var(--source-color, var(--color-accent-cyan));
	}
	.roll { position: relative; height: 236px; overflow: hidden; }
	.roll.vertical { min-height: 360px; }
	.roll svg { display: block; width: 100%; height: 100%; }
	.roll-bg { fill: #0a0912; }
	.piano-rail-key { stroke: #39333f; stroke-width: 0.65; }
	.octave-line { stroke: #312b43; stroke-width: 1; }
	.time-line { stroke: #252137; stroke-width: 1; }
	.time-line.now-line { stroke: rgba(245, 233, 201, 0.75); }
	.pitch-label { fill: #8f879f; font: 8px var(--font-code); }
	.contour {
		stroke-width: 1.6;
		stroke-linecap: round;
		stroke-linejoin: round;
		opacity: 0.55;
		vector-effect: non-scaling-stroke;
	}
	.note {
		opacity: 0.92;
		paint-order: stroke fill;
		filter: drop-shadow(0 0 2px currentColor);
	}
	.empty {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		padding: 20px;
		color: var(--color-text-secondary);
		font-size: 9px;
		text-align: center;
	}
	.axis-help {
		position: absolute;
		left: 62px;
		right: 7px;
		bottom: 5px;
		display: flex;
		justify-content: space-between;
		color: var(--color-text-dim);
		font-size: 7px;
		pointer-events: none;
	}
	.axis-help span { color: var(--color-text-secondary); }
	.vertical-axis {
		top: 29px;
		bottom: 7px;
		left: 7px;
		right: auto;
		width: 42px;
		flex-direction: column;
		align-items: center;
	}
	.vertical-axis b { color: var(--color-text-secondary); font-weight: 400; writing-mode: vertical-rl; }
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}
	.plain-language {
		min-height: 27px;
		padding: 6px 8px;
		border-top: 1px solid rgba(51, 221, 255, 0.32);
		background: rgba(10, 24, 31, 0.92);
		color: var(--color-text-secondary);
		font-size: 8px;
		line-height: 1.45;
	}
	.plain-language strong { color: var(--color-accent-cyan); }

	@media (max-width: 720px) {
		.toolbar { flex-wrap: wrap; }
		.description { display: none; }
		.orientation { margin-left: 0; }
		.orientation button, .filters button { padding-inline: 4px; }
		.roll:not(.vertical) { height: 190px !important; }
	}
</style>
