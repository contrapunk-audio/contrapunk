<script lang="ts">
	import { onMount } from 'svelte';
	import { platformName } from '$lib/adapter';
	import { engine } from '$lib/stores/engine.svelte';
	import { midiToName } from '$lib/embed/music-utils';

	type Role = 'player' | 'harmony' | 'canon' | 'counterpoint';
	type PitchSample = { at: number; midi: number; rms: number; clarity: number };
	type Gate = { id: number; role: Role; note: number; startedAt: number; endedAt: number | null };
	type GuitarSignalPayload = {
		rms: number;
		frequency: number | null;
		clarity: number;
		note_state: number;
		note_name: string;
		midi_note: number;
	};

	const WINDOW_MS = 8_000;
	const KEY_RAIL = 50;
	const DEFAULT_MIN = 40;
	const DEFAULT_MAX = 76;
	const MAX_PITCH_SAMPLES = 512;
	const MAX_GATES = 512;
	const colors: Record<Role, string> = {
		player: '#f1c75b',
		harmony: '#ec6f9e',
		canon: '#d8b85a',
		counterpoint: '#a7d878'
	};
	const offsets: Record<Role, number> = { player: -4, harmony: -1.5, canon: 1.5, counterpoint: 4 };

	let canvas = $state<HTMLCanvasElement>();
	let pitchName = $state('—');
	let frequency = $state<number | null>(null);
	let cents = $state(0);
	let dynamics = $state(0);
	let clarity = $state(0);
	let guitarLive = $state(false);
	let hasHistory = $state(false);
	let lastSignalAt = 0;
	let pitchSamples: PitchSample[] = [];
	let gates: Gate[] = [];
	let nextGateId = 1;
	const activeGates = new Map<string, Gate>();

	let playerLive = $derived(engine.inputNotes.length > 0);
	let harmonyLive = $derived(engine.harmonyNotes.length > 0);
	let canonLive = $derived(engine.canonNotes.length > 0);
	let counterpointLive = $derived(engine.counterpointNotes.length > 0);
	let midiSummary = $derived(
		engine.inputNotes.length ? engine.inputNotes.map(midiToName).join('  ') : 'Waiting for a stable pitch'
	);

	function syncRole(role: Role, notes: number[], now: number) {
		const current = new Set(notes);
		for (const [key, gate] of activeGates) {
			if (!key.startsWith(`${role}:`)) continue;
			if (current.has(gate.note)) continue;
			gate.endedAt = now;
			activeGates.delete(key);
		}
		for (const note of current) {
			const key = `${role}:${note}`;
			if (activeGates.has(key)) continue;
			const gate: Gate = { id: nextGateId++, role, note, startedAt: now, endedAt: null };
			gates.push(gate);
			activeGates.set(key, gate);
		}
	}

	$effect(() => {
		const now = Date.now();
		syncRole('player', engine.inputNotes, now);
		syncRole('harmony', engine.harmonyNotes, now);
		syncRole('canon', engine.canonNotes, now);
		syncRole('counterpoint', engine.counterpointNotes, now);
		if (gates.length > MAX_GATES) gates.splice(0, gates.length - MAX_GATES);
	});

	function readPitch(freq: number) {
		const fractional = 69 + 12 * Math.log2(freq / 440);
		const rounded = Math.round(fractional);
		pitchName = midiToName(rounded);
		frequency = freq;
		cents = Math.round((fractional - rounded) * 100);
		return fractional;
	}

	function draw() {
		if (!canvas) return;
		const ctx = canvas.getContext('2d');
		if (!ctx) return;
		const width = canvas.clientWidth;
		const height = canvas.clientHeight;
		const now = Date.now();
		const cutoff = now - WINDOW_MS;
		pitchSamples = pitchSamples.filter((sample) => sample.at >= cutoff);
		gates = gates.filter((gate) => gate.endedAt === null || gate.endedAt >= cutoff);
		hasHistory = pitchSamples.length > 0 || gates.length > 0;
		guitarLive = now - lastSignalAt < 250;
		if (!guitarLive) {
			frequency = null;
			pitchName = '—';
			cents = 0;
			dynamics = 0;
			clarity = 0;
		}

		ctx.clearRect(0, 0, width, height);
		ctx.fillStyle = '#0b0b0d';
		ctx.fillRect(0, 0, width, height);

		const recentNotes = [
			...gates.map((gate) => gate.note),
			...pitchSamples.map((sample) => sample.midi)
		];
		let minNote = DEFAULT_MIN;
		let maxNote = DEFAULT_MAX;
		if (recentNotes.length) {
			minNote = Math.max(0, Math.floor((Math.min(...recentNotes) - 4) / 12) * 12);
			maxNote = Math.min(127, Math.ceil((Math.max(...recentNotes) + 4) / 12) * 12);
			if (maxNote - minNote < 36) {
				const center = (minNote + maxNote) / 2;
				minNote = Math.max(0, Math.floor((center - 18) / 12) * 12);
				maxNote = Math.min(127, minNote + 36);
			}
		}
		const plotWidth = Math.max(1, width - KEY_RAIL);
		const plotHeight = Math.max(1, height - 16);
		const xFor = (at: number) => KEY_RAIL + ((at - cutoff) / WINDOW_MS) * plotWidth;
		const yFor = (note: number) => 8 + (1 - (note - minNote) / (maxNote - minNote)) * plotHeight;

		for (let note = minNote; note <= maxNote; note++) {
			const y = yFor(note);
			const black = [1, 3, 6, 8, 10].includes(note % 12);
			ctx.fillStyle = black ? '#1b1b1f' : '#d8d8da';
			ctx.fillRect(0, y - 2.5, black ? 33 : KEY_RAIL, 5);
			ctx.strokeStyle = note % 12 === 0 ? '#38383e' : '#202025';
			ctx.lineWidth = note % 12 === 0 ? 1 : 0.5;
			ctx.beginPath();
			ctx.moveTo(KEY_RAIL, Math.round(y) + 0.5);
			ctx.lineTo(width, Math.round(y) + 0.5);
			ctx.stroke();
			if (note % 12 === 0) {
				ctx.fillStyle = '#74747c';
				ctx.font = '9px ui-monospace, SFMono-Regular, Menlo, monospace';
				ctx.fillText(midiToName(note), 4, Math.max(9, y - 4));
			}
		}
		for (let step = 0; step <= 8; step++) {
			const x = KEY_RAIL + (step / 8) * plotWidth;
			ctx.strokeStyle = step === 8 ? '#55555c' : '#242429';
			ctx.lineWidth = 1;
			ctx.beginPath();
			ctx.moveTo(Math.round(x) + 0.5, 0);
			ctx.lineTo(Math.round(x) + 0.5, height);
			ctx.stroke();
		}

		for (const gate of gates) {
			const start = Math.max(cutoff, gate.startedAt);
			const end = Math.min(now, gate.endedAt ?? now);
			const x = xFor(start);
			const gateWidth = Math.max(3, xFor(end) - x);
			ctx.globalAlpha = gate.endedAt === null ? 0.95 : 0.62;
			ctx.fillStyle = colors[gate.role];
			ctx.fillRect(x, yFor(gate.note) + offsets[gate.role] - 2.5, gateWidth, 5);
		}
		ctx.globalAlpha = 1;

		if (pitchSamples.length > 1) {
			ctx.strokeStyle = '#42e8c4';
			ctx.lineWidth = 1.5;
			ctx.lineJoin = 'round';
			ctx.lineCap = 'round';
			ctx.beginPath();
			let previous: PitchSample | null = null;
			for (const sample of pitchSamples) {
				const x = xFor(sample.at);
				const y = yFor(sample.midi);
				if (!previous || sample.at - previous.at > 150) ctx.moveTo(x, y);
				else ctx.lineTo(x, y);
				previous = sample;
			}
			ctx.stroke();
			for (let index = 0; index < pitchSamples.length; index += 3) {
				const sample = pitchSamples[index];
				const age = (now - sample.at) / WINDOW_MS;
				ctx.globalAlpha = Math.max(0.12, (1 - age) * Math.max(0.25, sample.clarity));
				ctx.fillStyle = '#42e8c4';
				ctx.beginPath();
				ctx.arc(xFor(sample.at), yFor(sample.midi), 1.2 + Math.min(4, sample.rms * 32), 0, Math.PI * 2);
				ctx.fill();
			}
			const head = pitchSamples[pitchSamples.length - 1];
			ctx.globalAlpha = guitarLive ? 1 : 0.35;
			ctx.fillStyle = '#42e8c4';
			ctx.beginPath();
			ctx.arc(xFor(head.at), yFor(head.midi), 3 + Math.min(4, head.rms * 36), 0, Math.PI * 2);
			ctx.fill();
			ctx.globalAlpha = 1;
		}
	}

	onMount(() => {
		let disposed = false;
		let unlisten: (() => void) | undefined;
		let animation = 0;
		let lastDraw = 0;
		const resize = () => {
			if (!canvas) return;
			const ratio = Math.max(1, window.devicePixelRatio || 1);
			const rect = canvas.getBoundingClientRect();
			canvas.width = Math.round(rect.width * ratio);
			canvas.height = Math.round(rect.height * ratio);
			canvas.getContext('2d')?.setTransform(ratio, 0, 0, ratio, 0, 0);
			draw();
		};
		const observer = new ResizeObserver(resize);
		if (canvas) observer.observe(canvas);
		resize();

		const tick = (time: number) => {
			if (!document.hidden && time - lastDraw >= 33) {
				draw();
				lastDraw = time;
			}
			animation = requestAnimationFrame(tick);
		};
		animation = requestAnimationFrame(tick);

		if (platformName === 'tauri') {
			void import('@tauri-apps/api/event').then(async ({ listen }) => {
				if (disposed) return;
				unlisten = await listen<GuitarSignalPayload>('guitar-signal', ({ payload }) => {
					if (payload.frequency === null || !Number.isFinite(payload.frequency) || payload.frequency <= 20) return;
					const now = Date.now();
					const fractionalMidi = readPitch(payload.frequency);
					dynamics = Math.max(0, Math.min(1, payload.rms * 8));
					clarity = Math.max(0, Math.min(1, payload.clarity));
					lastSignalAt = now;
					pitchSamples.push({ at: now, midi: fractionalMidi, rms: Math.max(0, payload.rms), clarity });
					if (pitchSamples.length > MAX_PITCH_SAMPLES) {
						pitchSamples.splice(0, pitchSamples.length - MAX_PITCH_SAMPLES);
					}
				});
			});
		}

		return () => {
			disposed = true;
			unlisten?.();
			observer.disconnect();
			cancelAnimationFrame(animation);
		};
	});
</script>

<section class="expression-roll" aria-labelledby="expression-title">
	<header>
		<h2 id="expression-title">Performance</h2>
		<div class="legend" aria-label="Sound roles">
			<span class:live={guitarLive}><i class="guitar"></i>Guitar pitch</span>
			<span class:live={playerLive}><i class="player"></i>Emitted MIDI</span>
			<span class:live={harmonyLive}><i class="harmony"></i>Harmony</span>
			<span class:live={canonLive}><i class="canon"></i>Canon</span>
			<span class:live={counterpointLive}><i class="counterpoint"></i>Counterpoint</span>
		</div>
	</header>
	<div class="roll-frame">
		<canvas bind:this={canvas} aria-label="Eight-second piano roll comparing continuous guitar pitch with emitted MIDI and generated ensemble notes"></canvas>
		{#if !hasHistory}
			<div class="empty">Play a note to reveal its shape.</div>
		{/if}
	</div>
	<footer>
		<div><span>GUITAR</span><strong class:live={guitarLive}>{pitchName}{frequency === null ? '' : `  ${cents >= 0 ? '+' : ''}${cents}¢`}</strong></div>
		<div><span>MIDI OUT</span><strong class:live={playerLive}>{midiSummary}</strong></div>
		<div><span>DYNAMICS</span><strong class:live={guitarLive}>{guitarLive ? `${Math.round(dynamics * 100)}% · ${Math.round(clarity * 100)}% clear` : 'Waiting for sound'}</strong></div>
	</footer>
</section>

<style>
	.expression-roll { display: grid; min-height: 0; grid-template-rows: auto minmax(0, 1fr) auto; border: 1px solid var(--proto-line); background: var(--proto-panel); }
	header { min-height: 48px; display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 7px 12px; border-bottom: 1px solid var(--proto-line); }
	h2 { margin: 0; color: var(--proto-text); font-size: 14px; font-weight: 650; }
	.legend { display: flex; align-items: center; justify-content: flex-end; flex-wrap: wrap; gap: 12px; color: var(--proto-muted); font-size: 10px; }
	.legend span { display: inline-flex; align-items: center; gap: 5px; }
	.legend i { width: 12px; height: 2px; background: var(--proto-dim); }
	.legend span.live { color: var(--proto-text); }
	.legend span.live i.guitar { background: #42e8c4; }
	.legend span.live i.player { background: #f1c75b; }
	.legend span.live i.harmony { background: #ec6f9e; }
	.legend span.live i.canon { background: #d8b85a; }
	.legend span.live i.counterpoint { background: #a7d878; }
	.roll-frame { position: relative; min-height: 0; background: #0b0b0d; }
	canvas { display: block; width: 100%; height: 100%; }
	.empty { position: absolute; inset: 0; display: grid; place-items: center; padding-left: 50px; color: var(--proto-dim); font-size: 12px; pointer-events: none; }
	footer { display: grid; grid-template-columns: 1fr 1fr 1fr; border-top: 1px solid var(--proto-line); }
	footer > div { min-width: 0; padding: 8px 11px; border-right: 1px solid var(--proto-line); }
	footer > div:last-child { border-right: 0; }
	footer span { display: block; margin-bottom: 4px; color: var(--proto-muted); font-size: 9px; font-weight: 700; letter-spacing: .12em; }
	footer strong { display: block; overflow: hidden; color: var(--proto-dim); font: 500 11px/1.3 ui-monospace, SFMono-Regular, Menlo, monospace; text-overflow: ellipsis; white-space: nowrap; }
	footer strong.live { color: var(--proto-text); }
	@media (max-width: 760px) {
		header { align-items: flex-start; flex-direction: column; }
		.legend { justify-content: flex-start; }
		footer { grid-template-columns: 1fr; }
		footer > div { border-right: 0; border-bottom: 1px solid var(--proto-line); }
	}
</style>
