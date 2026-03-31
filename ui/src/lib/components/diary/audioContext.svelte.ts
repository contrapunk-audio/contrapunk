/**
 * Singleton AudioContext store for diary audio playback.
 * Manages a single AudioContext instance, handles browser autoplay policy,
 * and tracks currently playing audio.
 *
 * Usage:
 *   import { audioStore } from '$lib/components/diary/audioContext.svelte';
 *   audioStore.play('/samples/showcase/E2_open.wav');
 *   audioStore.stop();
 */

class AudioStore {
	playing = $state(false);
	currentUrl: string | null = $state(null);
	/** Current playback position in seconds */
	currentTime = $state(0);
	/** Duration of the currently loaded audio in seconds */
	duration = $state(0);

	private ctx: AudioContext | null = null;
	private sourceNode: AudioBufferSourceNode | null = null;
	private audioBuffer: AudioBuffer | null = null;
	private startedAt = 0;
	private rafId: number | null = null;
	private bufferCache = new Map<string, AudioBuffer>();

	private getContext(): AudioContext {
		if (!this.ctx) {
			this.ctx = new AudioContext();
		}
		return this.ctx;
	}

	/**
	 * Fetch and decode a WAV file, returning an AudioBuffer.
	 * Results are cached so repeated plays of the same URL are instant.
	 */
	async decode(url: string): Promise<AudioBuffer> {
		const cached = this.bufferCache.get(url);
		if (cached) return cached;

		const ctx = this.getContext();
		const response = await fetch(url);
		const arrayBuffer = await response.arrayBuffer();
		const audioBuffer = await ctx.decodeAudioData(arrayBuffer);
		this.bufferCache.set(url, audioBuffer);
		return audioBuffer;
	}

	/**
	 * Play audio from a URL. If the same URL is already playing, this is a no-op.
	 * If a different URL is playing, it stops first.
	 */
	async play(url: string): Promise<void> {
		// Already playing this URL — do nothing
		if (this.playing && this.currentUrl === url) return;

		// Stop anything currently playing
		this.stopInternal();

		const ctx = this.getContext();

		// Handle browser autoplay policy
		if (ctx.state === 'suspended') {
			try {
				await ctx.resume();
			} catch {
				// Still suspended — will be retried on next user gesture
				return;
			}
		}

		try {
			const buffer = await this.decode(url);
			this.audioBuffer = buffer;

			const source = ctx.createBufferSource();
			source.buffer = buffer;
			source.connect(ctx.destination);

			source.onended = () => {
				// Only reset if this source is still the active one
				if (this.sourceNode === source) {
					this.stopInternal();
				}
			};

			this.sourceNode = source;
			this.currentUrl = url;
			this.duration = buffer.duration;
			this.playing = true;
			this.startedAt = ctx.currentTime;
			source.start(0);

			this.startTimeTracking();
		} catch (e) {
			console.error('AudioStore: failed to play', url, e);
			this.stopInternal();
		}
	}

	/** Stop all playback and reset state. */
	stop(): void {
		this.stopInternal();
	}

	private stopInternal(): void {
		if (this.sourceNode) {
			try {
				this.sourceNode.stop();
			} catch {
				// Already stopped
			}
			this.sourceNode.disconnect();
			this.sourceNode = null;
		}
		this.stopTimeTracking();
		this.playing = false;
		this.currentUrl = null;
		this.currentTime = 0;
		this.duration = 0;
		this.audioBuffer = null;
	}

	/** Returns true if the given URL is the one currently playing. */
	isPlaying(url: string): boolean {
		return this.playing && this.currentUrl === url;
	}

	/** Get playback progress as a fraction 0-1. */
	progress(url: string): number {
		if (!this.playing || this.currentUrl !== url || this.duration === 0) return 0;
		return Math.min(this.currentTime / this.duration, 1);
	}

	private startTimeTracking(): void {
		this.stopTimeTracking();
		const tick = () => {
			if (!this.playing || !this.ctx) return;
			this.currentTime = this.ctx.currentTime - this.startedAt;
			this.rafId = requestAnimationFrame(tick);
		};
		this.rafId = requestAnimationFrame(tick);
	}

	private stopTimeTracking(): void {
		if (this.rafId !== null) {
			cancelAnimationFrame(this.rafId);
			this.rafId = null;
		}
	}
}

export const audioStore = new AudioStore();
