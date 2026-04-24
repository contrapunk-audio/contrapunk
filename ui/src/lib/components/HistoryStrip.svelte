<script lang="ts">
	/**
	 * HistoryStrip — rolling window of recent notes rendered as actual
	 * sheet-music notation via VexFlow. Single grand staff (treble +
	 * bass), one shared memory for all voices. Color encodes the source
	 * of each note: teal = user-played input, magenta = engine-generated
	 * harmony, violet = borrowed / modal interchange.
	 *
	 * SVG renderer sized to 1040 × 150 viewBox to visually match the
	 * Fretboard exactly (same aspect ratio, same apparent width/height
	 * when stacked above it).
	 */
	import { onMount, onDestroy } from 'svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { ui } from '$lib/stores/ui.svelte';
	import { Renderer, Stave, StaveNote, Formatter, Accidental, Voice } from 'vexflow';

	const WINDOW = 12;
	const W = 1040;
	// Taller than the fretboard so the grand staff has room to breathe;
	// the outer CSS keeps the horizontal width matched to Fretboard.
	const H = 240;

	type Kind = 'input' | 'harmony' | 'borrowed';
	interface Entry { kind: Kind; midi: number; ts: number; }

	const COLORS: Record<Kind, string> = {
		input: '#4fe8c3',
		harmony: '#ff2e88',
		borrowed: '#8a5cff',
	};

	/** A Moment is a single point in engine state — a chord of notes all
	 *  sounding together. Every engine change pushes one moment. */
	interface Moment { ts: number; keys: Array<{ midi: number; kind: Kind }>; }

	let history = $state<Moment[]>([]);
	let lastSig = '';

	/** Build a stable string signature of the current engine state so we
	 *  can detect "same chord, no change" and skip duplicate pushes. */
	function stateSig(inp: number[], har: number[], bor: number[]): string {
		return `i:${[...inp].sort().join(',')}|h:${[...har].sort().join(',')}|b:${[...bor].sort().join(',')}`;
	}

	function buildMoment(inp: number[], har: number[], bor: number[]): Moment {
		const keys: Array<{ midi: number; kind: Kind }> = [];
		for (const m of inp) keys.push({ midi: m, kind: 'input' });
		for (const m of har) keys.push({ midi: m, kind: 'harmony' });
		for (const m of bor) keys.push({ midi: m, kind: 'borrowed' });
		// Sort top-to-bottom by pitch so VexFlow renders chord heads cleanly.
		keys.sort((a, b) => b.midi - a.midi);
		return { ts: Date.now(), keys };
	}

	$effect(() => {
		const inp = engine.inputNotes;
		const har = engine.harmonyNotes;
		const bor = engine.borrowedNotes;
		const sig = stateSig(inp, har, bor);
		if (sig === lastSig) return;
		lastSig = sig;
		// Skip the initial all-empty snapshot so the strip doesn't start
		// with a silent rest — only push when at least one voice is active.
		if (inp.length === 0 && har.length === 0 && bor.length === 0) return;
		const moment = buildMoment(inp, har, bor);
		history = [...history.slice(-(WINDOW - 1)), moment];
	});

	/** MIDI → (keySpec, accidental). VexFlow uses letter+octave notation
	 *  (e.g. "c/4"); accidentals are attached as modifiers, not part of key. */
	function midiToKey(m: number): { key: string; acc: string | null } {
		const pc = ((m % 12) + 12) % 12;
		// Map each pitch class to its "natural letter + accidental needed"
		const map: [string, string | null][] = [
			['c', null], ['c', '#'],
			['d', null], ['d', '#'],
			['e', null],
			['f', null], ['f', '#'],
			['g', null], ['g', '#'],
			['a', null], ['a', '#'],
			['b', null],
		];
		const [letter, acc] = map[pc];
		const octave = Math.floor(m / 12) - 1;
		return { key: `${letter}/${octave}`, acc };
	}

	/** Which clef a note should sit on based on its pitch.
	 *  MIDI 60 (C4) is Middle C — conventional split point. */
	function clefForMidi(m: number): 'treble' | 'bass' {
		return m >= 60 ? 'treble' : 'bass';
	}

	// ===== VexFlow render =====

	let svgHost: HTMLDivElement | null = null;
	let renderer: Renderer | null = null;

	function draw() {
		const hostOrNull = svgHost;
		if (!hostOrNull) return;
		const host: HTMLDivElement = hostOrNull;

		// Clear previous frame
		host.innerHTML = '';
		renderer = new Renderer(host, Renderer.Backends.SVG);
		renderer.resize(W, H);
		const ctx = renderer.getContext();

		// Grand staff — treble on top, bass on bottom, both full width.
		// Vertical layout: 20px top margin, treble stave centered in the
		// upper half, 24px gutter, bass stave in the lower half, 20px
		// bottom margin.
		const staffX = 10;
		const staffW = W - 20;
		const treble = new Stave(staffX, 20, staffW);
		treble.addClef('treble');
		treble.setContext(ctx).draw();

		const bass = new Stave(staffX, 130, staffW);
		bass.addClef('bass');
		bass.setContext(ctx).draw();

		// Style the clefs + staff lines to match the HLD palette.
		// VexFlow draws in black (default SVG fill/stroke). Elements that
		// lack an explicit stroke attribute still render black — force the
		// override unconditionally so staff LINES also take the new color.
		host.querySelectorAll('path, line, rect, text').forEach((el) => {
			const svgEl = el as SVGElement;
			if (svgEl.getAttribute('fill') !== 'none') svgEl.setAttribute('fill', '#dcd3e8');
			if (svgEl.getAttribute('stroke') !== 'none') svgEl.setAttribute('stroke', '#dcd3e8');
		});

		if (history.length === 0) return;

		// Each moment becomes one StaveNote per clef (treble above C4, bass
		// below). All notes in the moment share an x-position — they render
		// as a CHORD, one stack per time-step, not as separate quarter notes.
		const trebleNotes: StaveNote[] = [];
		const bassNotes: StaveNote[] = [];

		history.forEach((moment) => {
			const trebleKeys = moment.keys.filter((k) => clefForMidi(k.midi) === 'treble');
			const bassKeys = moment.keys.filter((k) => clefForMidi(k.midi) === 'bass');

			const makeChord = (group: typeof trebleKeys, clef: 'treble' | 'bass') => {
				if (group.length === 0) return null;
				// VexFlow wants keys top→bottom; our sort already did that.
				const keyStrings = group.map((k) => midiToKey(k.midi).key);
				const n = new StaveNote({ keys: keyStrings, duration: 'q', clef });
				group.forEach((k, i) => {
					const { acc } = midiToKey(k.midi);
					if (acc) n.addModifier(new Accidental(acc), i);
					const c = COLORS[k.kind];
					(n as unknown as { setKeyStyle: (i: number, s: object) => void })
						.setKeyStyle(i, { fillStyle: c, strokeStyle: c });
				});
				return n;
			};

			const tn = makeChord(trebleKeys, 'treble');
			const bn = makeChord(bassKeys, 'bass');
			if (tn) trebleNotes.push(tn);
			if (bn) bassNotes.push(bn);
		});

		// VexFlow needs Voices with a fixed beats count. Use loose timing
		// (quarter notes = 4 per "measure"). Pad with rests where needed.
		function renderVoice(stave: Stave, notes: StaveNote[]) {
			if (notes.length === 0) return;
			const voice = new Voice({ numBeats: notes.length, beatValue: 4 }).setStrict(false);
			voice.addTickables(notes);
			new Formatter().joinVoices([voice]).format([voice], staffW - 80);
			voice.draw(ctx, stave);
			// After-the-fact color pass — setStyle on a note only colors the
			// primary shape; stems + flags inherit VexFlow's default. Re-paint
			// the note-head children so every glyph takes the right source color.
			notes.forEach((n) => {
				const color = (n.getStyle()?.fillStyle as string | undefined) ?? '#dcd3e8';
				const group = (n as unknown as { attrs?: { id?: string } }).attrs?.id;
				if (!group) return;
				const el = host.querySelector(`#${CSS.escape(group)}`);
				if (!el) return;
				el.querySelectorAll('path, rect, ellipse, circle').forEach((p) => {
					(p as SVGElement).setAttribute('fill', color);
					(p as SVGElement).setAttribute('stroke', color);
				});
			});
		}

		renderVoice(treble, trebleNotes);
		renderVoice(bass, bassNotes);

		// Note-name labels: per-chord pitch class list under each moment.
		if (ui.showNoteLabels) {
			const svgNS = 'http://www.w3.org/2000/svg';
			const svgRoot = host.querySelector('svg');
			if (svgRoot) {
				history.forEach((moment, mi) => {
					const chordLabel = moment.keys.map((k) => noteName(k.midi)).join(' ');
					// Position roughly under each chord column based on index.
					const colW = (W - 80) / Math.max(history.length, 1);
					const x = 60 + colW * (mi + 0.5);
					const t = document.createElementNS(svgNS, 'text');
					t.setAttribute('x', String(x));
					t.setAttribute('y', String(H - 6));
					t.setAttribute('text-anchor', 'middle');
					t.setAttribute('font-family', 'JetBrains Mono, ui-monospace, monospace');
					t.setAttribute('font-size', '8');
					t.setAttribute('fill', '#8888aa');
					t.textContent = chordLabel;
					svgRoot.appendChild(t);
				});
			}
		}
	}

	function noteName(m: number): string {
		const n = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
		return n[((m % 12) + 12) % 12] + (Math.floor(m / 12) - 1);
	}

	// Re-render when history or the label toggle changes
	$effect(() => {
		// read reactive deps so the effect re-fires
		history;
		ui.showNoteLabels;
		draw();
	});

	onMount(() => {
		draw();
		window.addEventListener('resize', draw);
	});
	onDestroy(() => {
		window.removeEventListener('resize', draw);
	});
</script>

<div class="history-strip" aria-label="Recent notes — grand staff memory">
	<!-- Fixed-aspect SVG host sized to match the Fretboard (1040 × 150).
	     VexFlow renders its SVG inside at that exact pixel size; the outer
	     CSS scales it responsively without distorting proportions. -->
	<div bind:this={svgHost} class="staff-host"></div>
</div>

<style>
	.history-strip {
		width: 100%;
		padding: 0 0 2px 0;
		background: var(--color-bg-deep);
		border-bottom: 1px solid var(--color-border);
	}
	.staff-host {
		width: 100%;
		height: auto;
		/* aspect-ratio locked to the SVG's native 1040 × 240 viewBox so the
		   strip scales width-first, taking whatever vertical room it needs
		   to keep clefs + staff lines readable. Width still matches the
		   Fretboard; height is ~60% taller to give notation room. */
		aspect-ratio: 1040 / 240 !important;
		display: block;
		overflow: hidden;
	}
	.staff-host :global(svg) {
		width: 100% !important;
		height: auto !important;
		display: block;
	}
</style>
