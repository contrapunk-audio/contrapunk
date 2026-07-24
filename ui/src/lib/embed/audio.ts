import workletUrl from './elixir-audio-processor.ts?worker&url';
import { addVoiceOwner, takeVoiceOwner } from './elixir-ownership.js';

const MAX_PENDING = 1024;
const ROLE_COUNT = 4;

interface AudioEvent {
	kind: number;
	atFrame: number;
	voiceId?: number;
	role?: number;
	anchor?: number;
	frequency?: number;
	velocity?: number;
	value?: number;
	seq?: number;
}

let ctx: AudioContext | null = null;
let node: AudioWorkletNode | null = null;
let nextVoiceId = 1;
let nextSequence = 1;
let pending = 0;
let panicSequence = 0;
let faulted = false;
let enabled = true;
let compareStandard = false;
let masterGain = 0.25;
const roleGains = [1, 1, 1, 1];
const queued: AudioEvent[] = [];
const voices = new Map<string, number[]>();
const voiceTargets = new Map<number, { anchor: number; frequency: number }>();

function ensureAudio(): AudioContext | null {
	if (ctx) return ctx;
	if (typeof window === 'undefined') return null;
	const Constructor =
		window.AudioContext ??
		(window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
	if (!Constructor) return null;

	try {
		ctx = new Constructor();
		const audio = ctx;
		void audio.audioWorklet
			.addModule(workletUrl)
			.then(() => {
				if (ctx !== audio) return;
				node = new AudioWorkletNode(audio, 'elixir-audio', {
					numberOfInputs: 0,
					numberOfOutputs: 1,
					outputChannelCount: [2]
				});
				node.connect(audio.destination);
				node.port.onmessage = ({ data }: MessageEvent<{ ack: number; panic?: boolean }>) => {
					pending = Math.max(0, pending - 1);
					if (data.panic) {
						voices.clear();
						voiceTargets.clear();
						faulted = false;
					}
					if (data.ack === panicSequence) faulted = false;
				};
				while (queued.length > 0) post(queued.shift()!);
				enqueue({ kind: 4, atFrame: audio.currentTime * audio.sampleRate, value: masterGain });
				for (let role = 0; role < ROLE_COUNT; role++) {
					enqueue({
						kind: 5,
						atFrame: audio.currentTime * audio.sampleRate,
						role,
						value: roleGains[role]
					});
				}
			})
			.catch(() => {
				if (ctx !== audio) return;
				queued.length = 0;
				voices.clear();
				faulted = true;
			});
		return ctx;
	} catch {
		ctx = null;
		return null;
	}
}

function post(event: AudioEvent) {
	if (!node) return;
	pending++;
	node.port.postMessage(event);
}

function overflow(audio: AudioContext) {
	voices.clear();
	voiceTargets.clear();
	queued.length = 0;
	faulted = true;
	const event = {
		kind: 3,
		atFrame: audio.currentTime * audio.sampleRate,
		seq: nextSequence++
	};
	panicSequence = event.seq;
	if (node) post(event);
	else queued.push(event);
}

function enqueue(event: AudioEvent) {
	const audio = ensureAudio();
	if (!audio || (faulted && event.kind !== 3)) return;
	if (pending + queued.length >= MAX_PENDING - 1) {
		overflow(audio);
		return;
	}
	event.seq = nextSequence++;
	if (node) post(event);
	else queued.push(event);
}

function eventFrame(audio: AudioContext, scheduleAt?: number): number {
	const when = Number.isFinite(scheduleAt) ? Math.max(audio.currentTime, scheduleAt!) : audio.currentTime;
	return when * audio.sampleRate;
}

function midiFrequency(midi: number): number {
	return 440 * 2 ** ((midi - 69) / 12);
}

async function resumeIfSuspended() {
	if (ctx?.state === 'suspended') {
		try {
			await ctx.resume();
		} catch {
			// A later user gesture retries.
		}
	}
}

/** Start one independently owned voice. Repeated pitches are released FIFO. */
export function noteOn(
	midi: number,
	velocity = 100,
	scheduleAt?: number,
	role = 0,
	frequencyHz = midiFrequency(midi)
) {
	const audio = ensureAudio();
	if (
		!audio ||
		!enabled ||
		faulted ||
		!Number.isFinite(midi) ||
		!Number.isFinite(velocity) ||
		!Number.isFinite(role) ||
		!Number.isFinite(frequencyHz)
	)
		return;
	void resumeIfSuspended();
	const voiceId = nextVoiceId++ >>> 0;
	if (voiceId === 0) {
		overflow(audio);
		nextVoiceId = 1;
		return;
	}
	role = Math.max(0, Math.min(ROLE_COUNT - 1, Math.trunc(role)));
	addVoiceOwner(voices, role, midi, voiceId);
	const anchor = Math.max(0, Math.min(127, Math.trunc(midi)));
	voiceTargets.set(voiceId, { anchor, frequency: frequencyHz });
	enqueue({
		kind: 0,
		atFrame: eventFrame(audio, scheduleAt),
		voiceId,
		role,
		anchor,
		frequency: compareStandard ? midiFrequency(anchor) : frequencyHz,
		velocity: Math.max(0, Math.min(127, Math.trunc(velocity)))
	});
}

export function noteOff(midi: number, scheduleAt?: number, role = 0) {
	const audio = ensureAudio();
	if (!Number.isFinite(midi) || !Number.isFinite(role)) return;
	role = Math.max(0, Math.min(ROLE_COUNT - 1, Math.trunc(role)));
	if (!audio) return;
	const voiceId = takeVoiceOwner(voices, role, midi);
	if (voiceId === undefined) return;
	voiceTargets.delete(voiceId);
	enqueue({ kind: 1, atFrame: eventFrame(audio, scheduleAt), voiceId });
}

export function allNotesOff() {
	const audio = ensureAudio();
	if (!audio) return;
	voices.clear();
	voiceTargets.clear();
	enqueue({ kind: 3, atFrame: audio.currentTime * audio.sampleRate });
}

export function setSustainPedal(on: boolean, scheduleAt?: number) {
	const audio = ensureAudio();
	if (!audio) return;
	enqueue({ kind: 2, atFrame: eventFrame(audio, scheduleAt), value: on ? 1 : 0 });
}

export function setCompareStandard(enabled: boolean) {
	if (compareStandard === enabled) return;
	compareStandard = enabled;
	const audio = ensureAudio();
	if (!audio) return;
	for (const [voiceId, target] of voiceTargets) {
		enqueue({
			kind: 6,
			atFrame: audio.currentTime * audio.sampleRate,
			voiceId,
			frequency: enabled ? midiFrequency(target.anchor) : target.frequency
		});
	}
}

export function setEnabled(on: boolean) {
	enabled = on;
	if (!on) allNotesOff();
}

export function setMasterGain(gain: number) {
	if (!Number.isFinite(gain)) return;
	masterGain = Math.max(0, Math.min(1, gain));
	const audio = ensureAudio();
	if (audio) enqueue({ kind: 4, atFrame: audio.currentTime * audio.sampleRate, value: masterGain });
}

export function setRoleGain(role: number, gain: number) {
	if (!Number.isFinite(role) || !Number.isFinite(gain)) return;
	role = Math.max(0, Math.min(ROLE_COUNT - 1, Math.trunc(role)));
	roleGains[role] = Math.max(0, Math.min(1, gain));
	const audio = ensureAudio();
	if (audio) {
		enqueue({
			kind: 5,
			atFrame: audio.currentTime * audio.sampleRate,
			role,
			value: roleGains[role]
		});
	}
}

export function getAudioContext(): AudioContext | null {
	return ensureAudio();
}

export function destroy() {
	compareStandard = false;
	voices.clear();
	voiceTargets.clear();
	queued.length = 0;
	if (node) {
		node.disconnect();
		node.port.close();
	}
	if (ctx) void ctx.close();
	ctx = null;
	node = null;
	pending = 0;
	faulted = false;
}
