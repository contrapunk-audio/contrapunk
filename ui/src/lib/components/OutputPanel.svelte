<script lang="ts">
	import { adapter, MAX_VOICES } from '$lib/adapter';
	import type { VoiceRouteId } from '$lib/adapter';
	import { arrangement } from '$lib/stores/arrangement.svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { stableVoiceRoutes } from '$lib/routing/stable-voice-routes.mjs';
	import ChainPanel from './ChainPanel.svelte';
	import PixelSelect from './PixelSelect.svelte';

	const SYNTH_VALUE = '__synth__';
	const OFF_VALUE = '__off__';
	const UNAVAILABLE_VALUE = '__unavailable__';
	type RouteRow = { route: VoiceRouteId; label: string; active: boolean };
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
		const rows = stableVoiceRoutes({
			voiceCount: engine.voiceCount,
			voicePosition: engine.voicePosition,
			harmonyEnabled: engine.mode !== 'PassThrough',
			companionEnabled: engine.companionEnabled,
			canonVoiceCount: engine.canonVoices.length,
			canonEnabled: engine.canonEnabled,
			counterpointEnabled: arrangement.counterpoint.enabled,
			phraseAware: arrangement.counterpoint.phraseAware,
			patternLowEnabled: arrangement.patterns.lowSupport.enabled,
			patternCounterEnabled: arrangement.patterns.counterline.enabled
		}) as Array<{ section: string; route: VoiceRouteId; active: boolean }>;

		const rowsFor = (section: string, label: (route: VoiceRouteId) => string): RouteRow[] =>
			rows.filter((row) => row.section === section).map((row) => ({ ...row, label: label(row.route) }));
		const routeIndex = (route: VoiceRouteId) => Number(route.split(':')[1]);

		return [
			{
				label: 'Performed input',
				rows: rowsFor('input', () => `You (${registerName(engine.voicePosition)})`)
			},
			{
				label: 'Harmony',
				rows: rowsFor('harmony', (route) => `${registerName(routeIndex(route))} harmony`)
			},
			{
				label: 'Canon',
				rows: rowsFor('canon', (route) => {
					const index = routeIndex(route);
					return engine.canonVoices[index]?.preset_id ?? `Canon ${index + 1}`;
				})
			},
			{
				label: arrangement.counterpoint.phraseAware ? 'Suspension' : 'Counterpoint',
				rows: rowsFor('counterpoint', (route) => route === 'counterpoint:0'
					? arrangement.counterpoint.phraseAware ? 'Tied inner voice' : 'Counterpoint line'
					: 'Moving bass')
			},
			{
				label: 'Patterns',
				rows: rowsFor('patterns', (route) => route === 'pattern_low'
					? 'Low Support pattern'
					: 'Counterline pattern')
			}
		].filter(({ rows }) => rows.length > 0);
	});

	let outputOptions = $derived([
		{ value: SYNTH_VALUE, label: 'Elixir Synth' },
		...midi.outputs.map((device) => ({ value: String(device.index), label: device.name })),
		{ value: OFF_VALUE, label: 'Off' }
	]);

	function outputOptionsFor(route: VoiceRouteId) {
		const unavailable = midi.getUnavailableVoiceOutputName(route);
		return unavailable
			? [...outputOptions, { value: UNAVAILABLE_VALUE, label: `Unavailable — ${unavailable}` }]
			: outputOptions;
	}

	async function handleOutputChange(route: VoiceRouteId, value: string) {
		if (value === UNAVAILABLE_VALUE) return;
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
		if (midi.getUnavailableVoiceOutputName(route)) return UNAVAILABLE_VALUE;
		const target = midi.getVoiceOutput(route);
		if (target.kind === 'synth') return SYNTH_VALUE;
		if (target.kind === 'off') return OFF_VALUE;
		return midi.outputs.some((device) => device.index === target.port)
			? String(target.port)
			: SYNTH_VALUE;
	}

	function routeHasDestination(route: VoiceRouteId): boolean {
		return midi.allVoiceOutputsToSynth || midi.getVoiceOutput(route).kind !== 'off';
	}

	function routeStatus(row: RouteRow): string {
		const unavailable = midi.getUnavailableVoiceOutputName(row.route);
		if (unavailable) return `Saved destination ${unavailable} is unavailable; using Elixir Synth`;
		if (midi.allVoiceOutputsToSynth) {
			return row.active
				? 'Part active; global destination is Elixir Synth'
				: 'Part inactive; per-voice route assignment is preserved';
		}
		if (!row.active) return 'Part inactive; route assignment is preserved';
		return routeHasDestination(row.route) ? 'Part active and routed' : 'Part active; destination is Off';
	}

	function routeSymbol(row: RouteRow): string {
		if (!row.active) return '◇';
		return routeHasDestination(row.route) ? '●' : '○';
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
				<div class="header-controls">
					<label class="all-synth-toggle font-ui" title="Temporarily send every part to Elixir without replacing the destinations below">
						<input
							type="checkbox"
							checked={midi.allVoiceOutputsToSynth}
							onchange={(event) => void midi.setAllVoiceOutputsToSynth(event.currentTarget.checked)}
						/>
						<span>All to Synth</span>
					</label>
					<div title="Number of voices the engine generates">
						<PixelSelect
							options={voiceCountOptions}
							value={String(engine.voiceCount)}
							small={true}
							onchange={onVoiceCountChange}
						/>
					</div>
				</div>
			</div>
			{#if midi.error}<p class="routing-error font-ui" role="alert">{midi.error}</p>{/if}
			<div class="output-slots">
				{#each routeSections as section (section.label)}
					<div class="route-group font-code">{section.label}</div>
					{#each section.rows as row (row.route)}
						<div class="output-slot" class:slot-off={!routeHasDestination(row.route)} class:part-inactive={!row.active}>
							<span class="slot-status" aria-label={routeStatus(row)} title={routeStatus(row)}>
								{routeSymbol(row)}
							</span>
							<span class="slot-label font-ui">{row.label}</span>
							<span class="part-state font-code" class:inactive={!row.active}>{row.active ? 'Active' : 'Inactive'}</span>
							<PixelSelect
								options={outputOptionsFor(row.route)}
								value={selectedOutput(row.route)}
								placeholder="None"
								small={true}
								disabled={midi.allVoiceOutputsToSynth}
								help={midi.allVoiceOutputsToSynth ? 'Turn off All to Synth to edit this saved destination' : undefined}
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

	.header-controls,
	.all-synth-toggle {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.all-synth-toggle {
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		white-space: nowrap;
		cursor: pointer;
	}

	.all-synth-toggle input { accent-color: var(--color-accent-cyan); }

	.routing-error {
		margin: 4px 0 6px;
		color: #ff7a91;
		font-size: var(--font-size-xs);
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
	.output-slot.slot-off .slot-status,
	.output-slot.part-inactive .slot-status {
		color: var(--color-text-dim);
		text-shadow: none;
	}
	.output-slot.slot-off { opacity: 0.55; }
	.output-slot.slot-off .slot-label,
	.output-slot.part-inactive .slot-label { color: var(--color-text-dim); }

	.slot-label {
		min-width: 96px;
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		white-space: nowrap;
	}

	.part-state {
		min-width: 38px;
		color: var(--color-text-secondary);
		font-size: 7px;
		letter-spacing: .06em;
		text-transform: uppercase;
	}
	.part-state.inactive { color: var(--color-text-dim); }

	.surface-unavailable {
		padding: 16px 12px;
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		line-height: 1.5;
		background: rgba(15, 14, 26, 0.5);
		border: 1px dashed var(--color-border);
	}
</style>
