<script lang="ts">
	import { onMount } from 'svelte';
	import { platformName } from '$lib/adapter';
	import type { SlideVoiceState } from '$lib/adapter/types';
	import { engine } from '$lib/stores/engine.svelte';
	import { ui, type PianoKeyCount } from '$lib/stores/ui.svelte';
	import { slide } from '$lib/stores/slide.svelte';
	import { harmonySlotLabel } from '$lib/slide/config';
	import { isBlackKey, midiToName } from '$lib/embed/music-utils';
	import {
		getPianoKeyColor,
		PIANO_CANON,
		PIANO_COUNTERPOINT,
		PIANO_HARMONY,
		PIANO_INPUT
	} from '$lib/theme/colors';

	type Role = 'player' | 'harmony' | 'canon' | 'counterpoint';
	type PitchSample = { at: number; midi: number; rms: number; clarity: number };
	type SlideSample = { id: string; at: number; midi: number; target: number; role: Role; voice: number; progress: number };
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
	const KEY_RANGES: Record<PianoKeyCount, readonly [number, number]> = {
		25: [48, 72],
		32: [41, 72],
		37: [36, 72],
		49: [36, 84],
		61: [36, 96],
		76: [28, 103],
		88: [21, 108]
	};
	const MAX_PITCH_SAMPLES = 512;
	const MAX_GATES = 512;
	const colors: Record<Role, string> = {
		player: PIANO_INPUT,
		harmony: PIANO_HARMONY,
		canon: PIANO_CANON,
		counterpoint: PIANO_COUNTERPOINT
	};

	let canvas = $state<HTMLCanvasElement>();
	let orientation = $state<'horizontal' | 'vertical'>('horizontal');
	let pitchName = $state('—');
	let frequency = $state<number | null>(null);
	let cents = $state(0);
	let dynamics = $state(0);
	let clarity = $state(0);
	let guitarLive = $state(false);
	let hasHistory = $state(false);
	let lastSignalAt = 0;
	let pitchSamples: PitchSample[] = [];
	let slideSamples: SlideSample[] = [];
	let gates: Gate[] = [];
	let nextGateId = 1;
	const activeGates = new Map<string, Gate>();

	let playerLive = $derived(engine.inputNotes.length > 0);
	let harmonyLive = $derived(engine.harmonyNotes.length > 0);
	let canonLive = $derived(engine.canonNotes.length > 0);
	let counterpointLive = $derived(engine.counterpointNotes.length > 0);
	let slideLive = $derived(slide.voices.length > 0);
	let midiSummary = $derived(
		engine.inputNotes.length ? engine.inputNotes.map(midiToName).join('  ') : 'Waiting for a stable pitch'
	);

	function slideRole(role: string): Role {
		return role === 'input' ? 'player' : role === 'canon' ? 'canon' : role === 'counterpoint' ? 'counterpoint' : 'harmony';
	}

	function slideVoiceLabel(role: Role, voice: number): string {
		if (role === 'player') return 'Your Voice';
		if (role === 'harmony') return `Harmonic Support · ${harmonySlotLabel(voice, engine.voiceCount)}`;
		if (role === 'canon') return `Canon · Voice ${voice + 1}`;
		return 'Counterpoint';
	}

	function slideCanvasLabel(voice: SlideVoiceState): string {
		const role = slideRole(voice.slot.role);
		const current = midiToName(Math.round(frequencyToMidi(voice.currentFrequencyHz)));
		const target = midiToName(Math.round(frequencyToMidi(voice.targetFrequencyHz)));
		return `${slideVoiceLabel(role, voice.slot.voice)} · ${current} → ${target} · ${Math.round(voice.progress * 100)}%`;
	}

	function activeKeyColor(note: number): string {
		return getPianoKeyColor(
			note,
			engine.inputNotes,
			engine.harmonyNotes,
			[],
			[],
			engine.canonNotes,
			engine.counterpointNotes
		);
	}

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

	function frequencyToMidi(freq: number) {
		return 69 + 12 * Math.log2(freq / 440);
	}

	function readPitch(freq: number) {
		const fractional = frequencyToMidi(freq);
		const rounded = Math.round(fractional);
		pitchName = midiToName(rounded);
		frequency = freq;
		cents = Math.round((fractional - rounded) * 100);
		return fractional;
	}

	function recordGuitarSignal(payload: GuitarSignalPayload) {
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
	}

	function draw() {
		if (!canvas) return;
		const ctx = canvas.getContext('2d');
		if (!ctx) return;
		const width = canvas.clientWidth;
		const height = canvas.clientHeight;
		const now = Date.now();
		const cutoff = now - WINDOW_MS;
		for (const voice of slide.voices) {
			if (
				voice.currentFrequencyHz > 0 &&
				voice.targetFrequencyHz > 0 &&
				Number.isFinite(voice.currentFrequencyHz)
			) {
				slideSamples.push({
					id: voice.voiceId,
					at: now,
					midi: frequencyToMidi(voice.currentFrequencyHz),
					target: frequencyToMidi(voice.targetFrequencyHz),
					role: slideRole(voice.slot.role),
					voice: voice.slot.voice,
					progress: voice.progress
				});
			}
		}
		pitchSamples = pitchSamples.filter((sample) => sample.at >= cutoff);
		slideSamples = slideSamples.filter((sample) => sample.at >= cutoff).slice(-1024);
		gates = gates.filter((gate) => gate.endedAt === null || gate.endedAt >= cutoff);
		hasHistory = pitchSamples.length > 0 || slideSamples.length > 0 || gates.length > 0;
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

		const [minNote, maxNote] = KEY_RANGES[ui.pianoKeyCount];
		const vertical = orientation === 'vertical';
		const railSize = vertical ? 42 : KEY_RAIL;
		const plotWidth = Math.max(1, width - (vertical ? 0 : KEY_RAIL));
		const plotHeight = Math.max(1, height - (vertical ? railSize : 0));
		const timeFor = (at: number) => vertical
			? railSize + ((at - cutoff) / WINDOW_MS) * plotHeight
			: KEY_RAIL + ((at - cutoff) / WINDOW_MS) * plotWidth;

		// Use real piano geometry: white keys occupy the rail and black keys
		// overlay their boundaries. The old fixed 5px bars were the reason
		// the keyboard looked like a barcode on a wide roll.
		const whiteNotes = Array.from(
			{ length: maxNote - minNote + 1 },
			(_, index) => minNote + index
		).filter((note) => !isBlackKey(note));
		const whiteIndex = new Map(whiteNotes.map((note, index) => [note, index]));
		const whiteCount = whiteNotes.length;
		const keyPosition = (note: number) => {
			const index = whiteIndex.get(note);
			if (index !== undefined) return index + 0.5;
			return (whiteIndex.get(note - 1) ?? -1) + 1;
		};
		const pianoPosition = (note: number) => {
			const bounded = Math.max(minNote, Math.min(maxNote, note));
			const low = Math.floor(bounded);
			const fraction = bounded - low;
			if (fraction === 0 || low === maxNote) return keyPosition(low);
			return keyPosition(low) + (keyPosition(low + 1) - keyPosition(low)) * fraction;
		};
		const pitchFor = (note: number) => {
			const position = pianoPosition(note) / whiteCount;
			return vertical ? position * width : (1 - position) * height;
		};
		const pointFor = (at: number, note: number): [number, number] => vertical
			? [pitchFor(note), timeFor(at)]
			: [timeFor(at), pitchFor(note)];

		const whiteSize = (vertical ? width : height) / whiteCount;
		ctx.fillStyle = '#111115';
		if (vertical) ctx.fillRect(0, 0, width, railSize);
		else ctx.fillRect(0, 0, railSize, height);

		for (const [index, note] of whiteNotes.entries()) {
			const inScale = engine.inScaleNotes.includes(note);
			const activeColor = activeKeyColor(note);
			ctx.fillStyle = activeColor || (inScale ? '#e4f1ee' : '#c9c9cd');
			ctx.strokeStyle = '#57575f';
			ctx.lineWidth = 0.75;
			ctx.shadowColor = activeColor || 'transparent';
			ctx.shadowBlur = activeColor ? 10 : 0;
			if (vertical) {
				const x = index * whiteSize;
				ctx.fillRect(x, 0, whiteSize, railSize);
				ctx.strokeRect(x, 0, whiteSize, railSize);
			} else {
				const y = height - (index + 1) * whiteSize;
				ctx.fillRect(0, y, railSize, whiteSize);
				ctx.strokeRect(0, y, railSize, whiteSize);
			}
			ctx.shadowBlur = 0;
		}

		for (let note = minNote; note <= maxNote; note++) {
			if (!isBlackKey(note)) continue;
			const inScale = engine.inScaleNotes.includes(note);
			const activeColor = activeKeyColor(note);
			ctx.fillStyle = activeColor || (inScale ? '#789e96' : '#17171b');
			ctx.strokeStyle = '#08080a';
			ctx.lineWidth = 1;
			ctx.shadowColor = activeColor || 'transparent';
			ctx.shadowBlur = activeColor ? 10 : 0;
			const center = pianoPosition(note) * whiteSize;
			if (vertical) {
				const keyWidth = whiteSize * 0.6;
				ctx.fillRect(center - keyWidth / 2, 0, keyWidth, railSize * 0.62);
				ctx.strokeRect(center - keyWidth / 2, 0, keyWidth, railSize * 0.62);
			} else {
				const keyHeight = whiteSize * 0.6;
				const y = height - center - keyHeight / 2;
				ctx.fillRect(0, y, railSize * 0.66, keyHeight);
				ctx.strokeRect(0, y, railSize * 0.66, keyHeight);
			}
			ctx.shadowBlur = 0;
		}

		for (let note = minNote; note <= maxNote; note++) {
			const pitch = pitchFor(note);
			ctx.strokeStyle = note % 12 === 0 ? '#38383e' : '#202025';
			ctx.lineWidth = note % 12 === 0 ? 1 : 0.5;
			ctx.beginPath();
			if (vertical) {
				ctx.moveTo(Math.round(pitch) + 0.5, railSize);
				ctx.lineTo(Math.round(pitch) + 0.5, height);
			} else {
				ctx.moveTo(KEY_RAIL, Math.round(pitch) + 0.5);
				ctx.lineTo(width, Math.round(pitch) + 0.5);
			}
			ctx.stroke();
			if (note % 12 === 0) {
				ctx.fillStyle = '#55555d';
				ctx.font = '9px ui-monospace, SFMono-Regular, Menlo, monospace';
				ctx.textAlign = vertical ? 'center' : 'start';
				ctx.fillText(midiToName(note), vertical ? pitch : 37, vertical ? railSize - 4 : pitch + 3);
			}
		}
		ctx.textAlign = 'start';
		for (let step = 0; step <= 8; step++) {
			const time = vertical
				? railSize + (step / 8) * plotHeight
				: KEY_RAIL + (step / 8) * plotWidth;
			ctx.strokeStyle = step === 8 ? '#55555c' : '#242429';
			ctx.lineWidth = 1;
			ctx.beginPath();
			if (vertical) {
				ctx.moveTo(0, Math.round(time) + 0.5);
				ctx.lineTo(width, Math.round(time) + 0.5);
			} else {
				ctx.moveTo(Math.round(time) + 0.5, 0);
				ctx.lineTo(Math.round(time) + 0.5, height);
			}
			ctx.stroke();
		}

		for (const gate of gates) {
			const start = Math.max(cutoff, gate.startedAt);
			const end = Math.min(now, gate.endedAt ?? now);
			const [startX, startY] = pointFor(start, gate.note);
			const [endX, endY] = pointFor(end, gate.note);
			const keySize = whiteSize * (isBlackKey(gate.note) ? 0.6 : 1);
			ctx.globalAlpha = gate.endedAt === null ? 0.95 : 0.62;
			ctx.fillStyle = colors[gate.role];
			if (vertical) {
				ctx.fillRect(startX - keySize / 2, startY, keySize, Math.max(3, endY - startY));
			} else {
				ctx.fillRect(startX, startY - keySize / 2, Math.max(3, endX - startX), keySize);
			}
		}
		ctx.globalAlpha = 1;

		const slidePaths = new Map<string, SlideSample[]>();
		for (const sample of slideSamples) {
			const path = slidePaths.get(sample.id) ?? [];
			path.push(sample);
			slidePaths.set(sample.id, path);
		}
		for (const path of slidePaths.values()) {
			if (!path.length) continue;
			ctx.strokeStyle = colors[path[0].role];
			ctx.lineWidth = 2.5;
			ctx.lineJoin = 'round';
			ctx.lineCap = 'round';
			ctx.beginPath();
			let previous: SlideSample | null = null;
			for (const sample of path) {
				const [x, y] = pointFor(sample.at, sample.midi);
				if (!previous || sample.at - previous.at > 100) ctx.moveTo(x, y);
				else ctx.lineTo(x, y);
				previous = sample;
			}
			ctx.stroke();
		}
		for (const voice of slide.voices) {
			const role = slideRole(voice.slot.role);
			const nowPoint = pointFor(now, frequencyToMidi(voice.currentFrequencyHz));
			const targetPoint = pointFor(now, frequencyToMidi(voice.targetFrequencyHz));
			ctx.strokeStyle = colors[role];
			ctx.fillStyle = colors[role];
			ctx.lineWidth = 1.5;
			ctx.setLineDash([3, 3]);
			ctx.beginPath();
			ctx.moveTo(nowPoint[0], nowPoint[1]);
			ctx.lineTo(targetPoint[0], targetPoint[1]);
			ctx.stroke();
			ctx.setLineDash([]);
			ctx.beginPath();
			ctx.arc(nowPoint[0], nowPoint[1], 4, 0, Math.PI * 2);
			ctx.fill();
			ctx.beginPath();
			ctx.arc(targetPoint[0], targetPoint[1], 5, 0, Math.PI * 2);
			ctx.stroke();
			ctx.font = '9px ui-monospace, SFMono-Regular, Menlo, monospace';
			const alignRight = targetPoint[0] > width * 0.65;
			ctx.textAlign = alignRight ? 'right' : 'start';
			ctx.fillText(
				slideCanvasLabel(voice),
				alignRight ? width - 6 : targetPoint[0] + 6,
				Math.max(11, targetPoint[1] - 6)
			);
			ctx.textAlign = 'start';
		}

		if (pitchSamples.length > 1) {
			ctx.strokeStyle = '#42e8c4';
			ctx.lineWidth = 1.5;
			ctx.lineJoin = 'round';
			ctx.lineCap = 'round';
			ctx.beginPath();
			let previous: PitchSample | null = null;
			for (const sample of pitchSamples) {
				const [x, y] = pointFor(sample.at, sample.midi);
				if (!previous || sample.at - previous.at > 150) ctx.moveTo(x, y);
				else ctx.lineTo(x, y);
				previous = sample;
			}
			ctx.stroke();
			for (let index = 0; index < pitchSamples.length; index += 3) {
				const sample = pitchSamples[index];
				const [x, y] = pointFor(sample.at, sample.midi);
				const age = (now - sample.at) / WINDOW_MS;
				ctx.globalAlpha = Math.max(0.12, (1 - age) * Math.max(0.25, sample.clarity));
				ctx.fillStyle = '#42e8c4';
				ctx.beginPath();
				ctx.arc(x, y, 1.2 + Math.min(4, sample.rms * 32), 0, Math.PI * 2);
				ctx.fill();
			}
			const head = pitchSamples[pitchSamples.length - 1];
			const [headX, headY] = pointFor(head.at, head.midi);
			ctx.globalAlpha = guitarLive ? 1 : 0.35;
			ctx.fillStyle = '#42e8c4';
			ctx.beginPath();
			ctx.arc(headX, headY, 3 + Math.min(4, head.rms * 36), 0, Math.PI * 2);
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
					recordGuitarSignal(payload);
				});
			});
		} else if (platformName === 'plugin') {
			unlisten = window.plugin.listen((message) => {
				try {
					const payload = JSON.parse(message) as GuitarSignalPayload & { type?: string };
					if (payload.type === 'guitarSignal') recordGuitarSignal(payload);
				} catch {
					/* Ignore unrelated host messages. */
				}
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

<section
	class="expression-roll"
	aria-labelledby="expression-title"
	style={`--player-color:${colors.player};--harmony-color:${colors.harmony};--canon-color:${colors.canon};--counterpoint-color:${colors.counterpoint}`}
>
	<header>
		<div class="roll-title">
			<h2 id="expression-title">{engine.chordName || '—'}</h2>
			<button
				type="button"
				aria-label="Flip piano roll orientation"
				title={orientation === 'horizontal' ? 'Place pitch left to right and time top to bottom' : 'Place pitch bottom to top and time left to right'}
				onclick={() => (orientation = orientation === 'horizontal' ? 'vertical' : 'horizontal')}
			>Flip {orientation === 'horizontal' ? '↕' : '↔'}</button>
		</div>
		<div class="legend" aria-label="Sound roles">
			<span class:live={guitarLive}><i class="guitar"></i>Guitar pitch</span>
			<span class:live={playerLive}><i class="player"></i>Emitted MIDI</span>
			<span class:live={harmonyLive}><i class="harmony"></i>Harmony</span>
			<span class:live={canonLive}><i class="canon"></i>Canon</span>
			<span class:live={counterpointLive}><i class="counterpoint"></i>Counterpoint</span>
			<span class:live={slideLive}><i class="slide"></i>Slide {slideLive ? slide.voices.length : ''}</span>
		</div>
	</header>
	<div class="roll-frame">
		<canvas bind:this={canvas} aria-label={`Eight-second piano roll with time moving ${orientation === 'horizontal' ? 'left to right' : 'top to bottom'}`}></canvas>
		{#if !hasHistory}
			<div class="empty" class:vertical={orientation === 'vertical'}>Play a note to reveal its shape.</div>
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
	.roll-title { display: flex; min-width: 0; align-items: center; gap: 10px; }
	h2 { min-width: 1.5em; margin: 0; overflow: hidden; color: var(--proto-text); font-size: 14px; font-weight: 650; text-overflow: ellipsis; white-space: nowrap; }
	.roll-title button { min-height: 26px; padding: 0 8px; border: 1px solid var(--proto-line-strong); background: transparent; color: var(--proto-muted); font: 650 9px var(--font-grotesk); }
	.roll-title button:hover { border-color: var(--proto-text); color: var(--proto-text); }
	.legend { display: flex; align-items: center; justify-content: flex-end; flex-wrap: wrap; gap: 12px; color: var(--proto-muted); font-size: 10px; }
	.legend span { display: inline-flex; align-items: center; gap: 5px; }
	.legend i { width: 12px; height: 2px; background: var(--proto-dim); }
	.legend span.live { color: var(--proto-text); }
	.legend span.live i.guitar { background: #42e8c4; }
	.legend span.live i.player { background: var(--player-color); }
	.legend span.live i.harmony { background: var(--harmony-color); }
	.legend span.live i.canon { background: var(--canon-color); }
	.legend span.live i.counterpoint { background: var(--counterpoint-color); }
	.legend span.live i.slide { background: linear-gradient(90deg, var(--player-color), var(--harmony-color)); }
	.roll-frame { position: relative; min-height: 0; background: #0b0b0d; }
	canvas { display: block; width: 100%; height: 100%; }
	.empty { position: absolute; inset: 0; display: grid; place-items: center; padding-left: 50px; color: var(--proto-dim); font-size: 12px; pointer-events: none; }
	.empty.vertical { padding-top: 42px; padding-left: 0; }
	footer { display: grid; grid-template-columns: repeat(3, 1fr); border-top: 1px solid var(--proto-line); }
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
