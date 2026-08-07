<script lang="ts">
	import { adapter, MAX_VOICES } from '$lib/adapter';
	import type { VoiceRouteId } from '$lib/adapter';
	import { arrangement } from '$lib/stores/arrangement.svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import ChainPanel from './ChainPanel.svelte';
	import PixelSelect from './PixelSelect.svelte';

	const SYNTH_VALUE = '__synth__';
	const OFF_VALUE = '__off__';
	type RouteRow = { route: VoiceRouteId; label: string };
	type RouteSection = { label: string; rows: RouteRow[] };

	const voiceCountOptions = Array.from({ length: MAX_VOICES }, (_, index) => index + 1).map(
		(count) => ({ value: String(count), label: count === 1 ? '1 voice' : `${count} voices` })
	);

	function onVoiceCountChange(value: string) {
		const count = Number.parseInt(value, 10);
		if (Number.isFinite(count)) void engine.setVoiceCount(count);
	}

	function registerName(slot: number): string {
		if (engine.voiceCount <= 4) return ['Soprano', 'Alto', 'Tenor', 'Bass'][slot] ?? `Voice ${slot + 1}`;
		return `Voice ${slot + 1}`;
	}

	let routeSections = $derived.by<RouteSection[]>(() => {
		const sections: RouteSection[] = [
			{
				label: 'Performed input',
				rows: [{ route: 'input', label: `You (${registerName(engine.voicePosition)})` }]
			}
		];
		const harmony = engine.mode === 'PassThrough'
			? []
			: Array.from({ length: engine.voiceCount }, (_, slot) => slot)
					.filter((slot) => slot !== engine.voicePosition)
					.map((slot) => ({
						route: `harmony:${slot}` as VoiceRouteId,
						label: `${registerName(slot)} harmony`
					}));
		if (harmony.length) sections.push({ label: 'Harmony', rows: harmony });

		if (engine.companionEnabled && engine.canonEnabled && engine.canonVoices.length) {
			sections.push({
				label: 'Canon',
				rows: engine.canonVoices.map((voice, index) => ({
					route: `canon:${index}` as VoiceRouteId,
					label: voice.preset_id ?? `Canon ${index + 1}`
				}))
			});
		}
		if (engine.companionEnabled && arrangement.counterpoint.enabled) {
			sections.push({
				label: arrangement.counterpoint.phraseAware ? 'Suspension' : 'Counterpoint',
				rows: arrangement.counterpoint.phraseAware
					? [
							{ route: 'counterpoint:0', label: 'Tied inner voice' },
							{ route: 'counterpoint:1', label: 'Moving bass' }
						]
					: [{ route: 'counterpoint:0', label: 'Counterpoint line' }]
			});
		}
		const patterns: RouteRow[] = [];
		if (engine.companionEnabled && arrangement.patterns.lowSupport.enabled) {
			patterns.push({ route: 'pattern_low', label: 'Low Support pattern' });
		}
		if (engine.companionEnabled && arrangement.patterns.counterline.enabled) {
			patterns.push({ route: 'pattern_counter', label: 'Counterline pattern' });
		}
		if (patterns.length) sections.push({ label: 'Patterns', rows: patterns });
		return sections;
	});

	let outputOptions = $derived([
		{ value: SYNTH_VALUE, label: 'Elixir Synth' },
		...midi.outputs.map((device) => ({ value: String(device.index), label: device.name })),
		{ value: OFF_VALUE, label: 'Off' }
	]);

	async function handleOutputChange(route: VoiceRouteId, value: string) {
		if (value === SYNTH_VALUE) return midi.setVoiceOutput(route, { kind: 'synth' });
		if (value === OFF_VALUE) return midi.setVoiceOutput(route, { kind: 'off' });
		const deviceIndex = Number.parseInt(value, 10);
		if (Number.isNaN(deviceIndex)) return;

		const needsConnection = !midi.selectedOutputs.includes(deviceIndex);
		const restart = needsConnection && engine.isRunning && midi.selectedInput !== null;
		try {
			if (restart) await engine.stop();
			const port = midi.ensureOutputPort(deviceIndex);
			await midi.setVoiceOutput(route, { kind: 'midi_port', port });
			if (restart && midi.selectedInput !== null) {
				await engine.start(midi.selectedInput, midi.selectedOutputs);
			}
		} catch (error) {
			midi.error = `Failed to connect output: ${error}`;
		}
	}

	function selectedOutput(route: VoiceRouteId): string {
		const target = midi.getVoiceOutput(route);
		if (target.kind === 'synth') return SYNTH_VALUE;
		if (target.kind === 'off') return OFF_VALUE;
		return midi.outputs.some((device) => device.index === target.port)
			? String(target.port)
			: SYNTH_VALUE;
	}

	function routeIsActive(route: VoiceRouteId): boolean {
		return midi.getVoiceOutput(route).kind !== 'off';
	}
</script>

<div class="output-panel">
	{#if adapter.capabilities.perVoicePortRouting}
		<!-- Per-voice MIDI port routing — replicates the OUTPUTS section
		     that previously lived in the Play tab. Hidden in plugin
		     mode where the DAW owns routing. -->
		<div class="routing-section pixel-card">
			<div class="output-header-row">
				<span class="section-header font-ui">Per-voice routing</span>
				<div title="Number of voices the engine generates">
					<PixelSelect
						options={voiceCountOptions}
						value={String(engine.voiceCount)}
						small={true}
						onchange={onVoiceCountChange}
					/>
				</div>
			</div>
			<div class="output-slots">
				{#each routeSections as section (section.label)}
					<div class="route-group font-code">{section.label}</div>
					{#each section.rows as row (row.route)}
						<div class="output-slot" class:slot-off={!routeIsActive(row.route)}>
							<span
								class="slot-status"
								aria-hidden="true"
								title={routeIsActive(row.route) ? 'Will produce output' : 'Silent'}
							>
								{routeIsActive(row.route) ? '●' : '○'}
							</span>
							<span class="slot-label font-ui">{row.label}</span>
							<PixelSelect
								options={outputOptions}
								value={selectedOutput(row.route)}
								placeholder="None"
								small={true}
								onchange={(value) => void handleOutputChange(row.route, value)}
							/>
						</div>
					{/each}
				{/each}
			</div>
		</div>
	{/if}

	{#if adapter.capabilities.builtInFx || adapter.capabilities.chainEditor}
		<!-- The Synth view owns synth levels. Output renders the chain only
		     when this surface can edit built-in effects or hosted plug-ins. -->
		<ChainPanel />
	{:else if !adapter.capabilities.perVoicePortRouting}
		<div class="surface-unavailable font-ui">
			{#if adapter.capabilities.pluginMidiOutputMode}
				Your DAW owns audio output and MIDI routing in plugin mode.
				Use the host's mixer + plugin chain instead of these controls.
			{:else}
				The browser uses its built-in audio output and does not expose per-part MIDI destinations.
			{/if}
		</div>
	{/if}
</div>

<style>
	.output-panel {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 8px 10px 14px;
		height: 100%;
		overflow-y: auto;
	}

	.routing-section {
		padding: 6px 8px;
	}

	.section-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		letter-spacing: 1.5px;
		text-transform: uppercase;
	}

	.output-header-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 6px;
	}

	.output-slots {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.route-group {
		margin-top: 6px;
		padding-top: 5px;
		border-top: 1px solid var(--color-border);
		color: var(--color-text-dim);
		font-size: 8px;
		letter-spacing: .08em;
		text-transform: uppercase;
	}

	.route-group:first-child { margin-top: 0; padding-top: 0; border-top: 0; }

	.output-slot {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.slot-status {
		font-size: var(--font-size-xs);
		line-height: 1;
		width: 10px;
		text-align: center;
		color: var(--color-accent-cyan);
		text-shadow: 0 0 4px var(--color-accent-cyan);
	}
	.output-slot.slot-off .slot-status {
		color: var(--color-text-dim);
		text-shadow: none;
	}
	.output-slot.slot-off {
		opacity: 0.55;
	}
	.output-slot.slot-off .slot-label {
		color: var(--color-text-dim);
	}

	.slot-label {
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		white-space: nowrap;
		min-width: 96px;
	}

	.surface-unavailable {
		padding: 16px 12px;
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		line-height: 1.5;
		background: rgba(15, 14, 26, 0.5);
		border: 1px dashed var(--color-border);
	}
</style>
