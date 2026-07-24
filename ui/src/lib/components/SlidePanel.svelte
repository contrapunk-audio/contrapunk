<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { slide } from '$lib/stores/slide.svelte';
	import type {
		SlideCurve,
		SlideOverride,
		SlideRole,
		SlideSettings,
		SlideTravel,
		SlideTrigger
	} from '$lib/adapter';
	import {
		harmonySlotLabel,
		SLIDE_PRESETS,
		SLIDE_ROLES,
		slideRoleLabel
	} from '$lib/slide/config';

	const curves: { value: SlideCurve; label: string }[] = [
		{ value: 'linear', label: 'Linear' },
		{ value: 'exponential', label: 'Exponential' },
		{ value: 'inverse_exponential', label: 'Inverse Exponential' }
	];

	function roleIndex(role: SlideRole) {
		return SLIDE_ROLES.indexOf(role);
	}

	function roleSettings(role: SlideRole): SlideSettings {
		return slide.config.roles[roleIndex(role)];
	}

	function travelMode(travel: SlideTravel | null): 'inherit' | 'off' | 'time' | 'rate' {
		return travel?.kind ?? 'inherit';
	}

	function travelValue(travel: SlideTravel | null): number {
		return travel?.kind === 'time'
			? travel.milliseconds
			: travel?.kind === 'rate'
				? travel.semitones_per_second
				: 180;
	}

	function travel(mode: string, value: number): SlideTravel | null {
		if (mode === 'inherit') return null;
		if (mode === 'time') return { kind: 'time', milliseconds: Math.max(1, Math.min(5000, value)) };
		if (mode === 'rate') return { kind: 'rate', semitones_per_second: Math.max(0.1, Math.min(96, value)) };
		return { kind: 'off' };
	}

	function setRoleTravel(role: SlideRole, mode: string, value: number) {
		slide.setRole(role, { travel: travel(mode, value) ?? { kind: 'off' } });
	}

	function voiceSlots(role: SlideRole): number[] {
		if (role === 'harmony') {
			return Array.from({ length: engine.voiceCount }, (_, index) => index).filter(
				(index) => index !== engine.voicePosition
			);
		}
		if (role === 'canon') return Array.from({ length: Math.min(8, engine.canonVoices.length) }, (_, index) => index);
		if (role === 'counterpoint') return [0];
		return [];
	}

	function voiceLabel(role: SlideRole, voice: number): string {
		if (role === 'harmony') return harmonySlotLabel(voice, engine.voiceCount);
		if (role === 'canon') return `Canon ${voice + 1}`;
		return 'Counterpoint';
	}

	function voiceOverride(role: SlideRole, voice: number): SlideOverride {
		return slide.config.voices[roleIndex(role)][voice];
	}

	function setVoiceTravel(role: SlideRole, voice: number, mode: string, value: number) {
		slide.setVoiceTravel(role, voice, travel(mode, value));
	}
</script>

<section class="slide-panel" aria-labelledby="slide-heading">
	<header>
		<div>
			<h3 id="slide-heading">Slide</h3>
			<p>Continuous pitch movement after harmony and tuning.</p>
		</div>
		<label>
			<span>Preset</span>
			<select value={slide.selectedPreset} onchange={(event) => slide.applyPreset(event.currentTarget.value)}>
				{#if slide.selectedPreset === 'custom'}<option value="custom">Custom</option>{/if}
				{#each SLIDE_PRESETS as preset}<option value={preset.id}>{preset.name}</option>{/each}
			</select>
		</label>
	</header>

	<div class="roles">
		{#each SLIDE_ROLES as role}
			{@const settings = roleSettings(role)}
			<details open={role === 'input' || role === 'harmony'}>
				<summary>
					<strong>{slideRoleLabel(role)}</strong>
					<span>{settings.travel.kind === 'off' ? 'Off' : settings.travel.kind === 'time' ? `${Math.round(settings.travel.milliseconds)} ms` : `${settings.travel.semitones_per_second} st/s`}</span>
				</summary>
				<div class="role-controls">
					<label>
						<span>Travel</span>
						<select value={settings.travel.kind} onchange={(event) => setRoleTravel(role, event.currentTarget.value, travelValue(settings.travel))}>
							<option value="off">Off</option><option value="time">Time</option><option value="rate">Rate</option>
						</select>
					</label>
					{#if settings.travel.kind !== 'off'}
						<label>
							<span>{settings.travel.kind === 'time' ? 'Milliseconds' : 'Semitones / second'}</span>
							<input
								type="number"
								min={settings.travel.kind === 'time' ? 1 : 0.1}
								max={settings.travel.kind === 'time' ? 5000 : 96}
								step={settings.travel.kind === 'time' ? 10 : 0.1}
								value={travelValue(settings.travel)}
								onchange={(event) => setRoleTravel(role, settings.travel.kind, Number(event.currentTarget.value))}
							/>
						</label>
					{/if}
					<label><span>Trigger</span><select value={settings.trigger} onchange={(event) => slide.setRole(role, { trigger: event.currentTarget.value as SlideTrigger })}><option value="legato">Legato</option><option value="always">Always</option></select></label>
					<label><span>Curve</span><select value={settings.curve} onchange={(event) => slide.setRole(role, { curve: event.currentTarget.value as SlideCurve })}>{#each curves as curve}<option value={curve.value}>{curve.label}</option>{/each}</select></label>
				</div>

				{#if voiceSlots(role).length}
					<div class="voices">
						<div class="voice-heading">Generated voice overrides</div>
						{#each voiceSlots(role) as voice}
							{@const override = voiceOverride(role, voice)}
							<div class="voice-row">
								<strong>{voiceLabel(role, voice)}</strong>
								<label><span>Travel</span><select value={travelMode(override.travel)} onchange={(event) => setVoiceTravel(role, voice, event.currentTarget.value, travelValue(override.travel ?? settings.travel))}><option value="inherit">Inherit</option><option value="off">Off</option><option value="time">Time</option><option value="rate">Rate</option></select></label>
								{#if override.travel?.kind === 'time' || override.travel?.kind === 'rate'}
									<input aria-label={`${voiceLabel(role, voice)} Slide value`} type="number" min={override.travel.kind === 'time' ? 1 : 0.1} max={override.travel.kind === 'time' ? 5000 : 96} step={override.travel.kind === 'time' ? 10 : 0.1} value={travelValue(override.travel)} onchange={(event) => setVoiceTravel(role, voice, override.travel!.kind, Number(event.currentTarget.value))} />
								{/if}
								<label><span>Trigger</span><select value={override.trigger ?? 'inherit'} onchange={(event) => slide.setVoiceTrigger(role, voice, event.currentTarget.value === 'inherit' ? null : event.currentTarget.value as SlideTrigger)}><option value="inherit">Inherit</option><option value="legato">Legato</option><option value="always">Always</option></select></label>
								<label><span>Curve</span><select value={override.curve ?? 'inherit'} onchange={(event) => slide.setVoiceCurve(role, voice, event.currentTarget.value === 'inherit' ? null : event.currentTarget.value as SlideCurve)}><option value="inherit">Inherit</option>{#each curves as curve}<option value={curve.value}>{curve.label}</option>{/each}</select></label>
							</div>
						{/each}
					</div>
				{/if}
			</details>
		{/each}
	</div>
	{#if slide.error}<p class="error">{slide.error}</p>{/if}
</section>

<style>
	.slide-panel { margin-top: 14px; border: 1px solid #333; background: #151515; }
	header { display: flex; align-items: end; justify-content: space-between; gap: 16px; padding: 12px; border-bottom: 1px solid #303030; }
	h3 { margin: 0; color: #eee; font-size: 14px; }
	p { margin: 3px 0 0; color: #858585; font-size: 11px; }
	label { display: grid; gap: 4px; min-width: 0; }
	label span, .voice-heading { color: #777; font: 9px var(--font-code); letter-spacing: .08em; text-transform: uppercase; }
	select, input { min-width: 0; height: 29px; border: 1px solid #3a3a3a; background: #1d1d1d; color: #ddd; font: 11px var(--font-ui); }
	.roles { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 1px; background: #303030; }
	details { min-width: 0; background: #171717; }
	summary { display: flex; justify-content: space-between; gap: 8px; padding: 10px 12px; cursor: pointer; color: #ddd; font-size: 11px; }
	summary span { color: #8fb2c8; font: 10px var(--font-code); }
	.role-controls { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 7px; padding: 0 12px 12px; }
	.voices { display: grid; gap: 5px; padding: 9px 12px 12px; border-top: 1px solid #292929; }
	.voice-row { display: grid; grid-template-columns: minmax(70px, .8fr) repeat(4, minmax(0, 1fr)); align-items: end; gap: 5px; }
	.voice-row strong { align-self: center; overflow: hidden; color: #bbb; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
	.voice-row input { margin-top: 13px; }
	.error { padding: 0 12px 10px; color: #e28b8b; }
	@media (max-width: 900px) { .roles { grid-template-columns: 1fr; } }
</style>
