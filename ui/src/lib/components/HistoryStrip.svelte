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
	const H = 150;

	type Kind = 'input' | 'harmony' | 'borrowed';
	interface Entry { kind: Kind; midi: number; ts: number; }

	const COLORS: Record<Kind, string> = {
		input: '#4fe8c3',
		harmony: '#ff2e88',
		borrowed: '#8a5cff',
	};

	let history = $state<Entry[]>([]);
	let prevInput: Set<number> = new Set();
	let prevHarmony: Set<number> = new Set();
	let prevBorrowed: Set<number> = new Set();

	function push(kind: Kind, midi: number) {
		history = [...history.slice(-(WINDOW - 1)), { kind, midi, ts: Date.now() }];
	}

	$effect(() => {
		const curr = new Set(engine.inputNotes);
		for (const m of curr) if (!prevInput.has(m)) push('input', m);
		prevInput = curr;
	});
	$effect(() => {
		const curr = new Set(engine.harmonyNotes);
		for (const m of curr) if (!prevHarmony.has(m)) push('harmony', m);
		prevHarmony = curr;
	});
	$effect(() => {
		const curr = new Set(engine.borrowedNotes);
		for (const m of curr) if (!prevBorrowed.has(m)) push('borrowed', m);
		prevBorrowed = curr;
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
		const host = svgHost;
		if (!host) return;

		// Clear previous frame
		host.innerHTML = '';
		renderer = new Renderer(host, Renderer.Backends.SVG);
		renderer.resize(W, H);
		const ctx = renderer.getContext();

		// Grand staff — treble on top, bass on bottom, both full width.
		const staffX = 10;
		const staffW = W - 20;
		const treble = new Stave(staffX, 10, staffW);
		treble.addClef('treble');
		treble.setContext(ctx).draw();

		const bass = new Stave(staffX, 75, staffW);
		bass.addClef('bass');
		bass.setContext(ctx).draw();

		// Style the clefs + staff lines to match the HLD palette.
		// VexFlow defaults to black which is invisible on our dark bg;
		// override to a readable pale violet at full opacity.
		host.querySelectorAll('path, line, rect, text').forEach((el) => {
			const svgEl = el as SVGElement;
			const fill = svgEl.getAttribute('fill');
			if (fill !== 'none') svgEl.setAttribute('fill', '#dcd3e8');
			const stroke = svgEl.getAttribute('stroke');
			if (stroke && stroke !== 'none') svgEl.setAttribute('stroke', '#dcd3e8');
		});

		if (history.length === 0) return;

		// Build VexFlow notes, split by clef so they render on the right stave.
		const trebleNotes: StaveNote[] = [];
		const bassNotes: StaveNote[] = [];
		const allEntries: { entry: Entry; vfnote: StaveNote; clef: 'treble' | 'bass' }[] = [];

		history.forEach((e) => {
			const clef = clefForMidi(e.midi);
			const { key, acc } = midiToKey(e.midi);
			const note = new StaveNote({ keys: [key], duration: 'q', clef });
			if (acc) note.addModifier(new Accidental(acc), 0);
			const c = COLORS[e.kind];
			note.setStyle({ fillStyle: c, strokeStyle: c, shadowBlur: 4, shadowColor: c });
			if (clef === 'treble') trebleNotes.push(note);
			else bassNotes.push(note);
			allEntries.push({ entry: e, vfnote: note, clef });
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

		// Hide note labels if the toggle is off — VexFlow does not render
		// letter names by default so there's nothing to hide; conversely if
		// the user WANTS them we add small text overlays under each notehead.
		if (ui.showNoteLabels) {
			allEntries.forEach(({ entry, vfnote, clef }) => {
				const box = (vfnote as unknown as { getBoundingBox?: () => { x: number; y: number; w: number; h: number } }).getBoundingBox?.();
				if (!box) return;
				const svgNS = 'http://www.w3.org/2000/svg';
				const text = document.createElementNS(svgNS, 'text');
				text.setAttribute('x', String(box.x + box.w / 2));
				text.setAttribute('y', String(clef === 'treble' ? 68 : 140));
				text.setAttribute('text-anchor', 'middle');
				text.setAttribute('font-family', 'JetBrains Mono, ui-monospace, monospace');
				text.setAttribute('font-size', '8');
				text.setAttribute('fill', COLORS[entry.kind]);
				text.textContent = noteName(entry.midi);
				host.querySelector('svg')?.appendChild(text);
			});
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
	<div
		bind:this={svgHost}
		class="staff-host"
		style:aspect-ratio="1040 / 150"
	></div>
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
		display: block;
		overflow: hidden;
	}
	.staff-host :global(svg) {
		width: 100% !important;
		height: auto !important;
		display: block;
	}
</style>
