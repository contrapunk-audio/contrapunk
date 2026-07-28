<script lang="ts">
	import { onMount } from 'svelte';
	import type {
		ArrangementPatternEventConfig,
		ArrangementPatternLaneConfig,
		ArrangementPatternLaneId,
		ArrangementPatternPitchAnchor
	} from '$lib/arrangement/presets';
	import { arrangement } from '$lib/stores/arrangement.svelte';

	type PatternRole = 'lowSupport' | 'counterline';

	const ROLES: {
		role: PatternRole;
		laneId: ArrangementPatternLaneId;
		title: string;
		description: string;
		accent: string;
	}[] = [
		{
			role: 'lowSupport',
			laneId: 'pattern_low',
			title: 'Low support',
			description: 'Independent bass-register pattern',
			accent: '#ffdd44'
		},
		{
			role: 'counterline',
			laneId: 'pattern_counter',
			title: 'Counterline',
			description: 'Independent answering pattern',
			accent: '#a3e635'
		}
	];

	let saving = $state<PatternRole | null>(null);
	let error = $state('');

	onMount(() => {
		void arrangement.syncFromBackend();
	});

	function clone(config: ArrangementPatternLaneConfig): ArrangementPatternLaneConfig {
		return { ...config, events: config.events.map((event) => ({ ...event })) };
	}

	function clamp(value: number, min: number, max: number): number {
		return Math.max(min, Math.min(max, Number.isFinite(value) ? value : min));
	}

	async function update(
		role: PatternRole,
		laneId: ArrangementPatternLaneId,
		change: (config: ArrangementPatternLaneConfig) => void
	) {
		const next = clone(arrangement.patterns[role]);
		change(next);
		next.events.sort((left, right) => left.beat - right.beat);
		saving = role;
		error = '';
		try {
			await arrangement.setPattern(role, laneId, next);
		} catch (cause) {
			error = `Could not update ${role === 'lowSupport' ? 'low support' : 'counterline'}: ${cause}`;
		} finally {
			saving = null;
		}
	}

	function registerOf(config: ArrangementPatternLaneConfig): number {
		return config.events[0]?.octave ?? 0;
	}

	function setCycle(
		role: PatternRole,
		laneId: ArrangementPatternLaneId,
		value: number
	) {
		void update(role, laneId, (config) => {
			const cycleBeats = clamp(value, 0.25, 32);
			const ratio = cycleBeats / config.cycleBeats;
			config.cycleBeats = cycleBeats;
			config.events = config.events.map((event) => ({
				...event,
				beat: Math.min(cycleBeats - 0.03125, event.beat * ratio),
				durationBeats: clamp(event.durationBeats * ratio, 0.03125, 32)
			}));
		});
	}

	function setEvent(
		role: PatternRole,
		laneId: ArrangementPatternLaneId,
		index: number,
		patch: Partial<ArrangementPatternEventConfig>
	) {
		void update(role, laneId, (config) => {
			const current = config.events[index];
			if (!current) return;
			config.events[index] = {
				...current,
				...patch,
				beat: clamp(patch.beat ?? current.beat, 0, config.cycleBeats - 0.03125),
				degree: Math.round(clamp(patch.degree ?? current.degree, 0, 6)),
				octave: Math.round(clamp(patch.octave ?? current.octave, -4, 4)),
				durationBeats: clamp(patch.durationBeats ?? current.durationBeats, 0.03125, 32),
				velocity: Math.round(clamp(patch.velocity ?? current.velocity, 1, 127))
			};
		});
	}

	function addEvent(role: PatternRole, laneId: ArrangementPatternLaneId) {
		void update(role, laneId, (config) => {
			if (config.events.length >= 16) return;
			const lastBeat = config.events.at(-1)?.beat ?? -0.5;
			config.events.push({
				beat: Math.min(config.cycleBeats - 0.03125, Math.max(0, lastBeat + 0.5)),
				degree: 0,
				octave: registerOf(config),
				durationBeats: 0.5,
				velocity: 72
			});
		});
	}
</script>

<div class="pattern-panel">
	<header>
		<div>
			<p>PATTERN ROLES</p>
			<h3>Low support + counterline</h3>
		</div>
		<span>Transport-synced · key-, phrase-, or note-relative</span>
	</header>

	{#if error}<p class="error" role="alert">{error}</p>{/if}

	<div class="lanes">
		{#each ROLES as definition}
			{@const pattern = arrangement.patterns[definition.role]}
			<article style={`--lane-accent:${definition.accent}`} class:disabled={!pattern.enabled}>
				<div class="lane-heading">
					<div><h4>{definition.title}</h4><p>{definition.description}</p></div>
					<label class="enable">
						<input
							type="checkbox"
							checked={pattern.enabled}
							disabled={saving === definition.role}
							onchange={(event) => void update(definition.role, definition.laneId, (config) => (config.enabled = event.currentTarget.checked))}
						/>
						<span>{pattern.enabled ? 'On' : 'Off'}</span>
					</label>
				</div>

				<div class="lane-controls">
					<label title="Shorter cycles repeat the whole role pattern faster">
						<span>Cycle / rate</span>
						<input type="range" min="0.25" max="32" step="0.25" value={pattern.cycleBeats} disabled={saving === definition.role} onchange={(event) => setCycle(definition.role, definition.laneId, Number(event.currentTarget.value))} />
						<output>{pattern.cycleBeats.toFixed(2)} beats</output>
					</label>
					<label title="How long the role keeps cycling after the latest played note">
						<span>Tail</span>
						<input type="range" min="0.25" max="32" step="0.25" value={pattern.tailBeats} disabled={saving === definition.role} onchange={(event) => void update(definition.role, definition.laneId, (config) => (config.tailBeats = clamp(Number(event.currentTarget.value), 0.25, 32)))} />
						<output>{pattern.tailBeats.toFixed(2)} beats</output>
					</label>
					<label>
						<span>Register</span>
						<select value={registerOf(pattern)} disabled={saving === definition.role || pattern.events.length === 0} onchange={(event) => void update(definition.role, definition.laneId, (config) => { const octave = Number(event.currentTarget.value); config.events = config.events.map((item) => ({ ...item, octave })); })}>
							{#each [-4, -3, -2, -1, 0, 1, 2, 3, 4] as octave}
								<option value={octave}>{octave === 0 ? 'Middle' : `${octave > 0 ? '+' : ''}${octave} octave${Math.abs(octave) === 1 ? '' : 's'}`}</option>
							{/each}
						</select>
					</label>
					<label title="Key follows the tonic; phrase follows the opening; latest follows your most recent attack">
						<span>Pitch anchor</span>
						<select value={pattern.pitchAnchor ?? 'key'} disabled={saving === definition.role} onchange={(event) => void update(definition.role, definition.laneId, (config) => (config.pitchAnchor = event.currentTarget.value as ArrangementPatternPitchAnchor))}>
							<option value="key">Current key</option>
							<option value="phrase_start">Phrase opening</option>
							<option value="latest_input">Latest note</option>
						</select>
					</label>
					<label class="idle-only" title="Skip scheduled attacks while the player holds a note, and yield immediately to a new attack">
						<span>Player space</span>
						<span class="check-row"><input type="checkbox" checked={pattern.onlyWhenInputIdle ?? false} disabled={saving === definition.role} onchange={(event) => void update(definition.role, definition.laneId, (config) => (config.onlyWhenInputIdle = event.currentTarget.checked))} /><output>Gaps only</output></span>
					</label>
				</div>

				<div class="step-heading"><span>STEPS</span><span>BEAT</span><span>DEGREE</span><span>LENGTH</span><span>VELOCITY</span><span></span></div>
				<div class="steps">
					{#each pattern.events as event, index}
						<div class="step">
							<b>{String(index + 1).padStart(2, '0')}</b>
							<label><span>Beat</span><input type="number" min="0" max={pattern.cycleBeats - 0.03125} step="0.25" value={event.beat} disabled={saving === definition.role} onchange={(change) => setEvent(definition.role, definition.laneId, index, { beat: Number(change.currentTarget.value) })} /></label>
							<label><span>Degree</span><select value={event.degree} disabled={saving === definition.role} onchange={(change) => setEvent(definition.role, definition.laneId, index, { degree: Number(change.currentTarget.value) })}>{#each [0, 1, 2, 3, 4, 5, 6] as degree}<option value={degree}>{degree + 1}</option>{/each}</select></label>
							<label><span>Length</span><input type="number" min="0.03125" max="32" step="0.125" value={event.durationBeats} disabled={saving === definition.role} onchange={(change) => setEvent(definition.role, definition.laneId, index, { durationBeats: Number(change.currentTarget.value) })} /></label>
							<label class="velocity"><span>Velocity</span><input type="range" min="1" max="127" step="1" value={event.velocity} disabled={saving === definition.role} onchange={(change) => setEvent(definition.role, definition.laneId, index, { velocity: Number(change.currentTarget.value) })} /><output>{event.velocity}</output></label>
							<button aria-label={`Remove ${definition.title} step ${index + 1}`} disabled={saving === definition.role} onclick={() => void update(definition.role, definition.laneId, (config) => config.events.splice(index, 1))}>×</button>
						</div>
					{/each}
					{#if pattern.events.length === 0}<p class="empty">No steps. Add one to build this role.</p>{/if}
				</div>
				<button class="add" disabled={saving === definition.role || pattern.events.length >= 16} onclick={() => addEvent(definition.role, definition.laneId)}>+ Add step</button>
			</article>
		{/each}
	</div>
	<p class="hint">Degree follows the chosen key or phrase-opening anchor through the current scale. “Gaps only” makes a role yield while you play. Role volume, mute, and solo stay in the Arrangement mixer.</p>
</div>

<style>
	.pattern-panel { margin-top: 16px; border: 1px solid var(--proto-line); background: var(--proto-panel); color: var(--proto-text); }
	.pattern-panel > header { display: flex; min-height: 54px; align-items: center; justify-content: space-between; gap: 16px; padding: 10px 12px; border-bottom: 1px solid var(--proto-line); }
	.pattern-panel > header p { margin: 0 0 3px; color: var(--proto-muted); font: 700 8px var(--font-code); letter-spacing: .14em; }
	.pattern-panel h3 { margin: 0; font-size: 14px; font-weight: 650; }
	.pattern-panel > header > span { color: var(--proto-dim); font: 9px var(--font-code); }
	.error { margin: 0; padding: 8px 12px; border-bottom: 1px solid #7b3030; color: #ffaaaa; font: 10px var(--font-code); }
	.lanes { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); }
	article { min-width: 0; border-right: 1px solid var(--proto-line); box-shadow: inset 2px 0 var(--lane-accent); }
	article:last-child { border-right: 0; }
	article.disabled .lane-controls, article.disabled .steps, article.disabled .step-heading { opacity: .55; }
	.lane-heading { display: flex; min-height: 50px; align-items: center; justify-content: space-between; gap: 12px; padding: 8px 12px; border-bottom: 1px solid var(--proto-line); }
	h4 { margin: 0 0 3px; font-size: 12px; }
	.lane-heading p { margin: 0; color: var(--proto-muted); font-size: 9px; }
	.enable { display: flex; align-items: center; gap: 6px; color: var(--proto-text); font: 700 9px var(--font-code); }
	.enable input { accent-color: var(--lane-accent); }
	.lane-controls { display: grid; grid-template-columns: 1fr 1fr .8fr 1fr .8fr; gap: 8px; padding: 10px 12px; border-bottom: 1px solid var(--proto-line); }
	.lane-controls label { display: grid; min-width: 0; gap: 5px; }
	.lane-controls label > span { color: var(--proto-muted); font: 700 8px var(--font-code); letter-spacing: .08em; }
	.lane-controls input[type='range'] { width: 100%; accent-color: var(--lane-accent); }
	.lane-controls output { color: var(--proto-text); font: 9px var(--font-code); }
	.check-row { display: flex; min-height: 26px; align-items: center; gap: 6px; }
	.check-row input { accent-color: var(--lane-accent); }
	select, input[type='number'] { box-sizing: border-box; width: 100%; min-width: 0; height: 26px; border: 1px solid var(--proto-line-strong); background: var(--proto-surface); color: var(--proto-text); font: 10px var(--font-code); }
	.step-heading, .step { display: grid; grid-template-columns: 30px .7fr .75fr .8fr 1.15fr 26px; align-items: center; gap: 6px; }
	.step-heading { padding: 7px 12px; border-bottom: 1px solid var(--proto-line); color: var(--proto-dim); font: 700 7px var(--font-code); letter-spacing: .08em; }
	.steps { min-height: 42px; max-height: 190px; overflow-y: auto; }
	.step { padding: 6px 12px; border-bottom: 1px solid var(--proto-line); }
	.step b { color: var(--lane-accent); font: 700 9px var(--font-code); }
	.step label { min-width: 0; }
	.step label > span { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
	.velocity { display: grid; grid-template-columns: 1fr 24px; align-items: center; gap: 4px; }
	.velocity input { width: 100%; accent-color: var(--lane-accent); }
	.velocity output { color: var(--proto-muted); font: 8px var(--font-code); text-align: right; }
	.step button, .add { border: 1px solid var(--proto-line-strong); background: transparent; color: var(--proto-muted); font: 700 11px var(--font-code); }
	.step button { width: 26px; height: 26px; }
	.step button:hover, .add:hover { border-color: var(--proto-text); color: var(--proto-text); }
	.add { min-height: 28px; margin: 8px 12px 10px; padding: 0 9px; font-size: 9px; }
	button:disabled, input:disabled, select:disabled { cursor: default; opacity: .45; }
	.empty { margin: 0; padding: 14px 12px; color: var(--proto-dim); font: 9px var(--font-code); }
	.hint { margin: 0; padding: 8px 12px; border-top: 1px solid var(--proto-line); color: var(--proto-dim); font: 9px/1.4 var(--font-code); }
	@media (max-width: 900px) {
		.lanes { grid-template-columns: 1fr; }
		article { border-right: 0; border-bottom: 1px solid var(--proto-line); }
		.lane-controls { grid-template-columns: 1fr; }
	}
</style>
