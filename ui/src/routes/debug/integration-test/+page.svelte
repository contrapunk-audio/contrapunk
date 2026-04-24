<script lang="ts">
	const FIXTURES_BASE = '/debug/fixtures';

	let fixtures = [
		{ name: 'Recording 01', wav: 'guitar_recording_01.wav', synth: 'guitar_recording_01_bp_synth.wav', midi: 'guitar_recording_01_expected.mid' },
		{ name: 'Open Strings', wav: 'guitar_open_strings.wav', synth: 'guitar_open_strings_bp_synth.wav', midi: 'guitar_open_strings_basic_pitch.mid' },
		{ name: 'Scale', wav: 'guitar_scale.wav', synth: 'guitar_scale_bp_synth.wav', midi: 'guitar_scale_basic_pitch.mid' },
	];

	let activeFixture = $state(0);
	let playing = $state<'none' | 'original' | 'synth' | 'both'>('none');
	let currentTime = $state(0);
	let duration = $state(0);

	// Audio
	let originalAudio: HTMLAudioElement | null = null;
	let synthAudio: HTMLAudioElement | null = null;
	let animFrame: number | null = null;

	// Waveform data
	let originalWaveform: Float32Array = new Float32Array(0);
	let synthWaveform: Float32Array = new Float32Array(0);
	let waveformSampleRate = 48000;

	// MIDI data
	let midiNotes: { start: number; end: number; pitch: number; velocity: number; name: string }[] = $state([]);

	// Canvas refs
	let originalWaveCanvas: HTMLCanvasElement;
	let synthWaveCanvas: HTMLCanvasElement;
	let pianoRollCanvas: HTMLCanvasElement;

	// ── Load fixture data ────────────────────────────────────
	async function loadFixture(idx: number) {
		stopAll();
		activeFixture = idx;
		const f = fixtures[idx];

		// Load original waveform
		originalWaveform = await loadWavData(`${FIXTURES_BASE}/${f.wav}`);
		// Load synth waveform
		synthWaveform = await loadWavData(`${FIXTURES_BASE}/${f.synth}`);
		// Load MIDI
		midiNotes = await loadMidi(`${FIXTURES_BASE}/${f.midi}`);

		// Draw everything
		requestAnimationFrame(() => {
			drawWaveform(originalWaveCanvas, originalWaveform, '#00e5cc');
			drawWaveform(synthWaveCanvas, synthWaveform, '#6366f1');
			drawPianoRoll();
		});
	}

	async function loadWavData(url: string): Promise<Float32Array> {
		const response = await fetch(url);
		const arrayBuffer = await response.arrayBuffer();
		const ctx = new OfflineAudioContext(1, 1, 48000);
		const audioBuffer = await ctx.decodeAudioData(arrayBuffer);
		waveformSampleRate = audioBuffer.sampleRate;
		duration = audioBuffer.duration;
		return audioBuffer.getChannelData(0);
	}

	async function loadMidi(url: string): Promise<typeof midiNotes> {
		const response = await fetch(url);
		const buffer = await response.arrayBuffer();
		return parseMidi(new Uint8Array(buffer));
	}

	// ── Simple MIDI parser (enough for basic-pitch output) ───
	function parseMidi(data: Uint8Array): typeof midiNotes {
		const notes: typeof midiNotes = [];
		// Find tracks
		let pos = 0;

		// Read header
		if (String.fromCharCode(...data.slice(0, 4)) !== 'MThd') return [];
		const headerLen = readUint32(data, 4);
		const format = readUint16(data, 8);
		const numTracks = readUint16(data, 10);
		const division = readUint16(data, 12);
		pos = 8 + headerLen;

		let microsecondsPerBeat = 500000; // 120 BPM default

		for (let track = 0; track < numTracks; track++) {
			if (pos + 8 > data.length) break;
			if (String.fromCharCode(...data.slice(pos, pos + 4)) !== 'MTrk') break;
			const trackLen = readUint32(data, pos + 4);
			let trackPos = pos + 8;
			const trackEnd = trackPos + trackLen;
			let tickTime = 0;
			const activeNotes: Map<number, { startTime: number; velocity: number }> = new Map();
			let prevStatus = 0;

			while (trackPos < trackEnd) {
				const [delta, newPos] = readVarLen(data, trackPos);
				trackPos = newPos;
				tickTime += delta;
				const currentTimeSeconds = (tickTime / division) * (microsecondsPerBeat / 1_000_000);

				let status = data[trackPos];
				if (status < 0x80) {
					// Running status
					status = prevStatus;
				} else {
					trackPos++;
					prevStatus = status;
				}

				const type = status & 0xf0;

				if (type === 0x90) {
					const pitch = data[trackPos++];
					const vel = data[trackPos++];
					if (vel > 0) {
						activeNotes.set(pitch, { startTime: currentTimeSeconds, velocity: vel });
					} else {
						// Note off
						const on = activeNotes.get(pitch);
						if (on) {
							notes.push({
								start: on.startTime,
								end: currentTimeSeconds,
								pitch,
								velocity: on.velocity,
								name: midiNoteName(pitch),
							});
							activeNotes.delete(pitch);
						}
					}
				} else if (type === 0x80) {
					const pitch = data[trackPos++];
					trackPos++; // velocity
					const on = activeNotes.get(pitch);
					if (on) {
						notes.push({
							start: on.startTime,
							end: currentTimeSeconds,
							pitch,
							velocity: on.velocity,
							name: midiNoteName(pitch),
						});
						activeNotes.delete(pitch);
					}
				} else if (type === 0xb0 || type === 0xe0) {
					trackPos += 2;
				} else if (type === 0xc0 || type === 0xd0) {
					trackPos += 1;
				} else if (status === 0xff) {
					const metaType = data[trackPos++];
					const [metaLen, mp] = readVarLen(data, trackPos);
					trackPos = mp;
					if (metaType === 0x51 && metaLen === 3) {
						microsecondsPerBeat = (data[trackPos] << 16) | (data[trackPos + 1] << 8) | data[trackPos + 2];
					}
					trackPos += metaLen;
				} else if (status === 0xf0 || status === 0xf7) {
					const [sysLen, sp] = readVarLen(data, trackPos);
					trackPos = sp + sysLen;
				} else {
					break; // unknown, bail
				}
			}
			pos = trackEnd;
		}

		return notes.sort((a, b) => a.start - b.start);
	}

	function readUint32(data: Uint8Array, pos: number): number {
		return (data[pos] << 24) | (data[pos + 1] << 16) | (data[pos + 2] << 8) | data[pos + 3];
	}
	function readUint16(data: Uint8Array, pos: number): number {
		return (data[pos] << 8) | data[pos + 1];
	}
	function readVarLen(data: Uint8Array, pos: number): [number, number] {
		let value = 0;
		let b;
		do {
			b = data[pos++];
			value = (value << 7) | (b & 0x7f);
		} while (b & 0x80);
		return [value, pos];
	}

	const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
	function midiNoteName(n: number): string {
		return `${NOTE_NAMES[n % 12]}${Math.floor(n / 12) - 1}`;
	}

	// ── Draw waveform ────────────────────────────────────────
	function drawWaveform(canvas: HTMLCanvasElement, samples: Float32Array, color: string) {
		if (!canvas || samples.length === 0) return;
		const ctx = canvas.getContext('2d');
		if (!ctx) return;
		const W = canvas.width;
		const H = canvas.height;

		ctx.fillStyle = '#0a0a0f';
		ctx.fillRect(0, 0, W, H);

		// Center line
		ctx.strokeStyle = '#1a1a2a';
		ctx.lineWidth = 1;
		ctx.beginPath();
		ctx.moveTo(0, H / 2);
		ctx.lineTo(W, H / 2);
		ctx.stroke();

		// Draw waveform as min/max per pixel column
		const samplesPerPixel = Math.floor(samples.length / W);
		ctx.fillStyle = color + '60';
		ctx.strokeStyle = color;
		ctx.lineWidth = 0.5;

		for (let x = 0; x < W; x++) {
			const start = x * samplesPerPixel;
			const end = Math.min(start + samplesPerPixel, samples.length);
			let min = 1, max = -1;
			for (let i = start; i < end; i++) {
				if (samples[i] < min) min = samples[i];
				if (samples[i] > max) max = samples[i];
			}
			const yMin = H / 2 - max * H / 2;
			const yMax = H / 2 - min * H / 2;
			ctx.fillRect(x, yMin, 1, yMax - yMin);
		}

		// Time markers
		const totalTime = samples.length / waveformSampleRate;
		ctx.fillStyle = '#333';
		ctx.font = '9px monospace';
		for (let t = 1; t < totalTime; t++) {
			const x = (t / totalTime) * W;
			ctx.fillRect(x, 0, 1, H);
			ctx.fillText(`${t}s`, x + 2, 10);
		}
	}

	// ── Draw piano roll ──────────────────────────────────────
	function drawPianoRoll() {
		if (!pianoRollCanvas || midiNotes.length === 0) return;
		const ctx = pianoRollCanvas.getContext('2d');
		if (!ctx) return;
		const W = pianoRollCanvas.width;
		const H = pianoRollCanvas.height;

		ctx.fillStyle = '#0a0a0f';
		ctx.fillRect(0, 0, W, H);

		// Find time and pitch range
		const maxTime = Math.max(...midiNotes.map(n => n.end), duration);
		const pitches = midiNotes.map(n => n.pitch);
		const minPitch = Math.min(...pitches) - 2;
		const maxPitch = Math.max(...pitches) + 2;
		const pitchRange = maxPitch - minPitch;

		// Grid lines for each note
		const noteHeight = H / pitchRange;
		ctx.strokeStyle = '#111';
		ctx.lineWidth = 0.5;
		for (let p = minPitch; p <= maxPitch; p++) {
			const y = H - ((p - minPitch) / pitchRange) * H;
			ctx.beginPath();
			ctx.moveTo(0, y);
			ctx.lineTo(W, y);
			ctx.stroke();

			// Label white keys
			const noteClass = p % 12;
			const isBlack = [1, 3, 6, 8, 10].includes(noteClass);
			if (!isBlack && noteHeight > 4) {
				ctx.fillStyle = '#222';
				ctx.font = '8px monospace';
				ctx.fillText(midiNoteName(p), 2, y - 1);
			}
		}

		// Time grid
		ctx.fillStyle = '#1a1a2a';
		for (let t = 1; t < maxTime; t++) {
			const x = (t / maxTime) * W;
			ctx.fillRect(x, 0, 1, H);
		}

		// Draw notes
		for (const note of midiNotes) {
			const x = (note.start / maxTime) * W;
			const w = Math.max(2, ((note.end - note.start) / maxTime) * W);
			const y = H - ((note.pitch - minPitch + 1) / pitchRange) * H;
			const h = Math.max(2, noteHeight - 1);

			// Color by pitch class — warm for low, cool for high
			const hue = ((note.pitch - 40) / 48) * 240;
			const alpha = 0.5 + (note.velocity / 127) * 0.5;
			ctx.fillStyle = `hsla(${hue}, 70%, 60%, ${alpha})`;
			ctx.fillRect(x, y, w, h);

			// Note name label if note is wide enough
			if (w > 20) {
				ctx.fillStyle = '#fff';
				ctx.font = '9px monospace';
				ctx.fillText(note.name, x + 2, y + h - 2);
			}
		}

		// Playback cursor
		if (playing !== 'none' && maxTime > 0) {
			const cx = (currentTime / maxTime) * W;
			ctx.strokeStyle = '#fff';
			ctx.lineWidth = 2;
			ctx.beginPath();
			ctx.moveTo(cx, 0);
			ctx.lineTo(cx, H);
			ctx.stroke();
		}
	}

	// ── Playback ─────────────────────────────────────────────
	function playOriginal() {
		stopAll();
		originalAudio = new Audio(`${FIXTURES_BASE}/${fixtures[activeFixture].wav}`);
		originalAudio.play();
		playing = 'original';
		trackTime(originalAudio);
	}

	function playSynth() {
		stopAll();
		synthAudio = new Audio(`${FIXTURES_BASE}/${fixtures[activeFixture].synth}`);
		synthAudio.play();
		playing = 'synth';
		trackTime(synthAudio);
	}

	function playBoth() {
		stopAll();
		originalAudio = new Audio(`${FIXTURES_BASE}/${fixtures[activeFixture].wav}`);
		synthAudio = new Audio(`${FIXTURES_BASE}/${fixtures[activeFixture].synth}`);
		originalAudio.volume = 0.7;
		synthAudio.volume = 0.5;
		originalAudio.play();
		synthAudio.play();
		playing = 'both';
		trackTime(originalAudio);
	}

	function stopAll() {
		if (originalAudio) { originalAudio.pause(); originalAudio = null; }
		if (synthAudio) { synthAudio.pause(); synthAudio = null; }
		if (animFrame) { cancelAnimationFrame(animFrame); animFrame = null; }
		playing = 'none';
		currentTime = 0;
	}

	function trackTime(audio: HTMLAudioElement) {
		const tick = () => {
			currentTime = audio.currentTime;
			duration = audio.duration || 0;
			drawPianoRoll(); // Redraw to move cursor
			if (!audio.paused) {
				animFrame = requestAnimationFrame(tick);
			} else {
				playing = 'none';
			}
		};
		animFrame = requestAnimationFrame(tick);
	}

	function formatTime(t: number): string {
		const m = Math.floor(t / 60);
		const s = Math.floor(t % 60);
		return `${m}:${s.toString().padStart(2, '0')}`;
	}

	// ── Init ─────────────────────────────────────────────────
	$effect(() => {
		loadFixture(0);
		return () => stopAll();
	});
</script>

<svelte:head>
	<title>Integration Test — Contrapunk</title>
</svelte:head>

<div class="page">
	<header>
		<h1>Integration Test — Input vs Expected</h1>
	</header>

	<nav class="fixtures">
		{#each fixtures as fixture, i}
			<button
				class="fixture-tab"
				class:active={activeFixture === i}
				onclick={() => loadFixture(i)}
			>
				{fixture.name}
			</button>
		{/each}
	</nav>

	<!-- Waveforms -->
	<section class="viz-section">
		<h2 class="original-color">Original Guitar</h2>
		<canvas bind:this={originalWaveCanvas} width={900} height={80}></canvas>
	</section>

	<section class="viz-section">
		<h2 class="synth-color">Basic-Pitch Synth</h2>
		<canvas bind:this={synthWaveCanvas} width={900} height={80}></canvas>
	</section>

	<!-- Piano roll -->
	<section class="viz-section">
		<h2 class="midi-color">MIDI Piano Roll <span class="dim">({midiNotes.length} notes)</span></h2>
		<canvas bind:this={pianoRollCanvas} width={900} height={200}></canvas>
	</section>

	<!-- Controls -->
	<section class="controls">
		<button class="play-btn original-btn" class:active={playing === 'original'} onclick={playOriginal}>
			▶ Original
		</button>
		<button class="play-btn synth-btn" class:active={playing === 'synth'} onclick={playSynth}>
			▶ Expected
		</button>
		<button class="play-btn both-btn" class:active={playing === 'both'} onclick={playBoth}>
			▶ Both
		</button>
		<button class="stop-btn" onclick={stopAll}>⏹</button>
		{#if playing !== 'none'}
			<span class="time">{formatTime(currentTime)} / {formatTime(duration)}</span>
		{/if}
	</section>
</div>

<style>
	.page {
		font-family: var(--font-code);
		background: #0a0a0f;
		color: #e0e0e0;
		min-height: 100vh;
		padding: 20px;
		max-width: 960px;
		margin: 0 auto;
	}

	header { margin-bottom: 12px; }
	h1 { font-size: 18px; font-weight: 600; color: #fff; margin: 0; }
	h2 { font-size: 11px; font-weight: 600; margin: 0 0 4px 0; text-transform: uppercase; letter-spacing: 0.5px; }
	.original-color { color: #00e5cc; }
	.synth-color { color: #6366f1; }
	.midi-color { color: #fbbf24; }
	.dim { color: #555; font-weight: 400; }

	.fixtures {
		display: flex;
		gap: 4px;
		margin-bottom: 12px;
	}

	.fixture-tab {
		padding: 5px 12px;
		background: #1a1a2e;
		border: 1px solid #333;
		color: #888;
		font-size: 12px;
		font-family: inherit;
		border-radius: 3px;
		cursor: pointer;
	}
	.fixture-tab.active { background: #6366f120; color: #6366f1; border-color: #6366f1; }

	.viz-section { margin-bottom: 8px; }

	canvas {
		width: 100%;
		height: auto;
		border: 1px solid #1a1a2a;
		border-radius: 4px;
		display: block;
	}

	.controls {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: 12px;
		position: sticky;
		bottom: 12px;
		background: #0a0a0fdd;
		padding: 10px;
		border-radius: 6px;
		border: 1px solid #222;
		backdrop-filter: blur(8px);
	}

	.play-btn {
		padding: 6px 14px;
		border: 1px solid;
		border-radius: 3px;
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		font-family: inherit;
	}
	.original-btn { background: #0f2a2a; color: #00e5cc; border-color: #00e5cc40; }
	.original-btn.active { background: #00e5cc; color: #000; }
	.synth-btn { background: #1a1a3e; color: #6366f1; border-color: #6366f140; }
	.synth-btn.active { background: #6366f1; color: #fff; }
	.both-btn { background: #2a2a1a; color: #fbbf24; border-color: #fbbf2440; }
	.both-btn.active { background: #fbbf24; color: #000; }

	.stop-btn {
		padding: 6px 10px;
		background: #1a1a2e;
		color: #888;
		border: 1px solid #333;
		border-radius: 3px;
		font-size: 12px;
		cursor: pointer;
		font-family: inherit;
	}

	.time { font-size: 11px; color: #555; margin-left: 8px; }
</style>
