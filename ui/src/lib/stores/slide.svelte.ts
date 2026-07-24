import { adapter } from '$lib/adapter';
import type {
	SlideConfig,
	SlideCurve,
	SlideRole,
	SlideSettings,
	SlideTravel,
	SlideTrigger,
	SlideVoiceState
} from '$lib/adapter';
import {
	cloneSlideConfig,
	defaultSlideConfig,
	SLIDE_PRESETS,
	SLIDE_ROLES
} from '$lib/slide/config';

const STORAGE_KEY = 'contrapunk-slide-v1';

class SlideStore {
	config = $state<SlideConfig>(defaultSlideConfig());
	loaded = $state(false);
	error = $state<string | null>(null);
	selectedPreset = $state('off');
	voices = $state<SlideVoiceState[]>([]);
	private operation: Promise<void> = Promise.resolve();
	private telemetryTimer: ReturnType<typeof setTimeout> | null = null;

	async init() {
		if (this.loaded) return;
		let config: SlideConfig | null = null;
		if (typeof localStorage !== 'undefined') {
			try {
				config = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null') as SlideConfig | null;
			} catch {
				localStorage.removeItem(STORAGE_KEY);
			}
		}
		if (!config) {
			try {
				config = await adapter.getSlideConfig();
			} catch {
				config = defaultSlideConfig();
			}
		}
		try {
			this.config = cloneSlideConfig(config);
		} catch {
			this.config = defaultSlideConfig();
		}
		this.loaded = true;
		await this.push();
		this.startTelemetry();
	}

	applyPreset(id: string) {
		const preset = SLIDE_PRESETS.find((candidate) => candidate.id === id);
		if (!preset) return;
		this.selectedPreset = id;
		this.config = cloneSlideConfig(preset.config);
		void this.push();
	}

	setRole(role: SlideRole, patch: Partial<SlideSettings>) {
		const config = cloneSlideConfig(this.config);
		const index = SLIDE_ROLES.indexOf(role);
		config.roles[index] = { ...config.roles[index], ...patch };
		this.selectedPreset = 'custom';
		this.config = config;
		void this.push();
	}

	setVoiceTravel(role: SlideRole, voice: number, travel: SlideTravel | null) {
		this.setVoice(role, voice, { travel });
	}

	setVoiceTrigger(role: SlideRole, voice: number, trigger: SlideTrigger | null) {
		this.setVoice(role, voice, { trigger });
	}

	setVoiceCurve(role: SlideRole, voice: number, curve: SlideCurve | null) {
		this.setVoice(role, voice, { curve });
	}

	private setVoice(
		role: SlideRole,
		voice: number,
		patch: {
			travel?: SlideTravel | null;
			trigger?: SlideTrigger | null;
			curve?: SlideCurve | null;
		}
	) {
		if (voice < 0 || voice >= 8) return;
		const config = cloneSlideConfig(this.config);
		const index = SLIDE_ROLES.indexOf(role);
		config.voices[index][voice] = { ...config.voices[index][voice], ...patch };
		this.selectedPreset = 'custom';
		this.config = config;
		void this.push();
	}

	private startTelemetry() {
		if (this.telemetryTimer !== null || typeof window === 'undefined') return;
		const poll = async () => {
			try {
				this.voices = await adapter.getSlideVoices();
			} catch {
				this.voices = [];
			}
			this.telemetryTimer = setTimeout(poll, 33);
		};
		this.telemetryTimer = setTimeout(poll, 0);
	}

	private async push() {
		const config = cloneSlideConfig(this.config);
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
		}
		this.operation = this.operation
			.then(() => adapter.setSlideConfig(config))
			.then(() => {
				this.error = null;
			})
			.catch((error) => {
				this.error = error instanceof Error ? error.message : String(error);
			});
		await this.operation;
	}
}

export const slide = new SlideStore();
