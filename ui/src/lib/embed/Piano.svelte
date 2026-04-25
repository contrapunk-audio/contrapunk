<script lang="ts">
	/**
	 * Piano — canonical embed component. State-library-agnostic.
	 *
	 * Same pattern as embed/Fretboard.svelte: data via props, click
	 * events via callbacks, zero store imports. Wrapped per consumer
	 * (contrapunk app feeds engine.svelte; website embed feeds
	 * nanostores).
	 *
	 * Mirror-copied to website/src/lib/contrapunk/embed/. Per #66.
	 */
	import {
		COLOR_INPUT,
		COLOR_HARMONY,
		COLOR_BORROWED,
		PIANO_START,
		PIANO_END,
		isBlackKey,
		midiToName
	} from './music-utils';

	let {
		inputNotes,
		harmonyNotes,
		borrowedNotes = [],
		onNoteOn,
		onNoteOff,
		interactive = true,
		showLabels = true
	}: {
		inputNotes: number[];
		harmonyNotes: number[];
		borrowedNotes?: number[];
		onNoteOn?: (midi: number) => void;
		onNoteOff?: (midi: number) => void;
		interactive?: boolean;
		showLabels?: boolean;
	} = $props();

	const allMidi = Array.from({ length: PIANO_END - PIANO_START + 1 }, (_, i) => i + PIANO_START);
	const whiteKeys = allMidi.filter((n) => !isBlackKey(n));
	const blackKeys = allMidi.filter((n) => isBlackKey(n));
	const NUM_WHITE_KEYS = whiteKeys.length; // 52

	const whiteKeyIndex = new Map<number, number>();
	whiteKeys.forEach((midi, i) => whiteKeyIndex.set(midi, i));

	function getBlackKeyPosition(midi: number, whiteKeyWidthPx: number): number {
		const prevWhite = midi - 1;
		const wIndex = whiteKeyIndex.get(prevWhite);
		if (wIndex === undefined) return 0;
		return wIndex * whiteKeyWidthPx + whiteKeyWidthPx - (whiteKeyWidthPx * 0.6) / 2;
	}

	function keyColor(midi: number): string {
		if (inputNotes.includes(midi)) return COLOR_INPUT;
		if (harmonyNotes.includes(midi)) return COLOR_HARMONY;
		if (borrowedNotes.includes(midi)) return COLOR_BORROWED;
		return '';
	}
	function keyGlow(midi: number): string {
		if (inputNotes.includes(midi)) return `0 0 6px ${COLOR_INPUT}, 0 0 14px ${COLOR_INPUT}66`;
		if (harmonyNotes.includes(midi)) return `0 0 6px ${COLOR_HARMONY}, 0 0 14px ${COLOR_HARMONY}66`;
		if (borrowedNotes.includes(midi)) return `0 0 6px ${COLOR_BORROWED}, 0 0 14px ${COLOR_BORROWED}66`;
		return 'none';
	}
	function isActive(midi: number): boolean {
		return (
			inputNotes.includes(midi) ||
			harmonyNotes.includes(midi) ||
			borrowedNotes.includes(midi)
		);
	}
	function labelColorFor(midi: number, isBlack: boolean): string {
		if (inputNotes.includes(midi)) return '#0a0612';
		if (harmonyNotes.includes(midi) || borrowedNotes.includes(midi)) return '#f5e9c9';
		return isBlack ? '#b79cea' : '#1a1520';
	}

	const pressedByPointer = new Map<number, number>();
	let pressing = $state(new Set<number>());

	function handlePointerDown(midi: number, e: PointerEvent) {
		if (!interactive) return;
		const el = e.currentTarget as HTMLElement;
		try {
			el.setPointerCapture(e.pointerId);
		} catch {
			// safe to ignore on touch
		}
		pressedByPointer.set(e.pointerId, midi);
		pressing = new Set(pressing).add(midi);
		onNoteOn?.(midi);
	}
	function releasePointer(midi: number, pointerId: number) {
		if (pressedByPointer.get(pointerId) === midi) {
			onNoteOff?.(midi);
			pressedByPointer.delete(pointerId);
		}
		const next = new Set(pressing);
		next.delete(midi);
		pressing = next;
	}
	function handlePointerUp(midi: number, e: PointerEvent) {
		if (!interactive) return;
		releasePointer(midi, e.pointerId);
	}
	function handlePointerCancel(midi: number, e: PointerEvent) {
		if (!interactive) return;
		releasePointer(midi, e.pointerId);
	}
</script>

<div class="piano-wrapper">
	<div class="piano-container" style="--num-white-keys: {NUM_WHITE_KEYS};">
		{#each whiteKeys as midi}
			{@const color = keyColor(midi)}
			{@const glow = keyGlow(midi)}
			{@const active = isActive(midi)}
			<div
				class="white-key"
				class:pressed={pressing.has(midi)}
				class:interactive
				data-midi={midi}
				style:background={color || '#f5e9c9'}
				style:box-shadow={glow}
				onpointerdown={(e) => handlePointerDown(midi, e)}
				onpointerup={(e) => handlePointerUp(midi, e)}
				onpointercancel={(e) => handlePointerCancel(midi, e)}
				onpointerleave={(e) => handlePointerCancel(midi, e)}
				role="button"
				tabindex="-1"
				aria-label={midiToName(midi)}
			>
				{#if active && showLabels}
					<span class="key-label" style:color={labelColorFor(midi, isBlackKey(midi))}>{midiToName(midi)}</span>
				{/if}
			</div>
		{/each}
		{#each blackKeys as midi}
			{@const color = keyColor(midi)}
			{@const glow = keyGlow(midi)}
			{@const active = isActive(midi)}
			<div
				class="black-key"
				class:pressed={pressing.has(midi)}
				class:interactive
				data-midi={midi}
				style:left="calc({getBlackKeyPosition(midi, 1)} * (100% / {NUM_WHITE_KEYS}))"
				style:width="calc(0.6 * (100% / {NUM_WHITE_KEYS}))"
				style:background={color || '#1a1520'}
				style:box-shadow={glow}
				onpointerdown={(e) => handlePointerDown(midi, e)}
				onpointerup={(e) => handlePointerUp(midi, e)}
				onpointercancel={(e) => handlePointerCancel(midi, e)}
				onpointerleave={(e) => handlePointerCancel(midi, e)}
				role="button"
				tabindex="-1"
				aria-label={midiToName(midi)}
			>
				{#if active && showLabels}
					<span class="key-label" style:color={labelColorFor(midi, true)}>{midiToName(midi)}</span>
				{/if}
			</div>
		{/each}
	</div>
</div>

<style>
	.piano-wrapper { width: 100%; position: relative; }
	.piano-container {
		position: relative;
		width: 100%;
		height: 90px;
		display: flex;
		flex-direction: row;
		background: #0a0612;
	}
	.white-key {
		flex: 1;
		height: 100%;
		border-right: 1px solid #2a1648;
		border-radius: 0;
		position: relative;
		z-index: 1;
		display: flex;
		align-items: flex-end;
		justify-content: center;
		padding-bottom: 2px;
		user-select: none;
		-webkit-user-select: none;
		-webkit-tap-highlight-color: transparent;
		transition: transform 60ms ease-out;
	}
	.white-key.interactive { cursor: pointer; }
	.black-key {
		position: absolute;
		top: 0;
		height: 55%;
		z-index: 2;
		border-radius: 0;
		border-left: 1px solid #000;
		border-right: 1px solid #000;
		border-bottom: 1px solid #000;
		display: flex;
		align-items: flex-end;
		justify-content: center;
		padding-bottom: 2px;
		user-select: none;
		-webkit-user-select: none;
		-webkit-tap-highlight-color: transparent;
		transition: transform 60ms ease-out;
	}
	.black-key.interactive { cursor: pointer; }
	.key-label {
		font-family: var(--font-code, ui-monospace, monospace);
		font-size: 9px;
		pointer-events: none;
		line-height: 1;
		text-align: center;
	}
	.white-key.pressed,
	.black-key.pressed { animation: cp-key-press 160ms ease-out; }
	@keyframes cp-key-press {
		0%   { transform: scaleY(0.94); filter: brightness(1.4); }
		60%  { transform: scaleY(0.98); filter: brightness(1.2); }
		100% { transform: scaleY(1);    filter: brightness(1);   }
	}
	@media (prefers-reduced-motion: reduce) {
		.white-key, .black-key { transition: none; }
		.white-key.pressed, .black-key.pressed { animation: none; }
	}
</style>
