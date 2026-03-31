<script lang="ts">
	import { audioStore } from '$lib/components/diary/audioContext.svelte';

	let { url, label = '' }: { url: string; label?: string } = $props();

	let localDuration = $state(0);
	let loaded = $state(false);

	// Pre-fetch and decode to get duration
	$effect(() => {
		loaded = false;
		audioStore.decode(url).then((buffer) => {
			localDuration = buffer.duration;
			loaded = true;
		}).catch(() => {
			loaded = false;
		});
	});

	function toggle() {
		if (audioStore.isPlaying(url)) {
			audioStore.stop();
		} else {
			audioStore.play(url);
		}
	}

	function formatTime(seconds: number): string {
		const s = Math.floor(seconds);
		const mins = Math.floor(s / 60);
		const secs = s % 60;
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	}

	let isActive = $derived(audioStore.isPlaying(url));
	let progress = $derived(audioStore.progress(url));
	let currentTime = $derived(isActive ? audioStore.currentTime : 0);
	let displayDuration = $derived(isActive ? audioStore.duration : localDuration);
</script>

<div class="audio-player" class:active={isActive}>
	<button
		class="play-btn"
		onclick={toggle}
		aria-label={isActive ? 'Pause' : 'Play'}
		disabled={!loaded}
	>
		{#if isActive}
			<!-- Pause icon: two vertical bars -->
			<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
				<rect x="3" y="2" width="3.5" height="12" rx="0.5" fill="currentColor" />
				<rect x="9.5" y="2" width="3.5" height="12" rx="0.5" fill="currentColor" />
			</svg>
		{:else}
			<!-- Play icon: triangle -->
			<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
				<path d="M4 2L14 8L4 14V2Z" fill="currentColor" />
			</svg>
		{/if}
	</button>

	<div class="track-area">
		{#if label}
			<div class="label">{label}</div>
		{/if}
		<div class="progress-bar">
			<div class="progress-fill" style="width: {progress * 100}%"></div>
		</div>
		<div class="time-row">
			<span class="time">{formatTime(currentTime)}</span>
			<span class="time">{loaded ? formatTime(displayDuration) : '--:--'}</span>
		</div>
	</div>
</div>

<style>
	.audio-player {
		display: flex;
		align-items: center;
		gap: 12px;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 12px;
	}
	.audio-player.active {
		border-color: var(--color-accent-cyan-dim);
	}

	.play-btn {
		flex-shrink: 0;
		width: 36px;
		height: 36px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: transparent;
		border: 2px solid var(--color-accent-magenta);
		color: var(--color-accent-magenta);
		cursor: pointer;
		transition: background 0.15s ease, border-color 0.15s ease;
	}
	.play-btn:hover:not(:disabled) {
		background: rgba(255, 51, 136, 0.1);
	}
	.play-btn:disabled {
		opacity: 0.4;
		cursor: default;
	}
	.active .play-btn {
		border-color: var(--color-accent-cyan);
		color: var(--color-accent-cyan);
	}
	.active .play-btn:hover {
		background: rgba(51, 221, 255, 0.1);
	}

	.track-area {
		flex: 1;
		min-width: 0;
	}

	.label {
		font-family: var(--font-reading);
		font-size: 12px;
		color: var(--color-text-primary);
		margin-bottom: 6px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.progress-bar {
		width: 100%;
		height: 6px;
		background: var(--color-bg-deep);
		border: 1px solid var(--color-border);
		overflow: hidden;
	}
	.progress-fill {
		height: 100%;
		background: var(--color-accent-cyan);
		transition: width 0.05s linear;
	}

	.time-row {
		display: flex;
		justify-content: space-between;
		margin-top: 4px;
	}
	.time {
		font-family: var(--font-pixel);
		font-size: 8px;
		color: var(--color-text-dim);
	}
</style>
