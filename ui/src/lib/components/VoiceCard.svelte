<script lang="ts">
	import type { HoldMode } from '$lib/adapter/types';

	type VoiceKind = 'melody' | 'canon' | 'counterpoint';

	type Props = {
		kind: VoiceKind;
		label: string;
		index?: number;
		transpose: number;
		/** Beats. For melody = 0 (live). */
		timeOffsetBeats?: number;
		holdMode?: HoldMode | null | undefined;
		/** "Synth" / "Port 2" / "Off" / etc. */
		output: string;
		onclick?: () => void;
	};

	let {
		kind,
		label,
		index,
		transpose,
		timeOffsetBeats,
		holdMode,
		output,
		onclick
	}: Props = $props();

	function holdModeShort(h: HoldMode | null | undefined): string {
		if (!h) return '-';
		switch (h.kind) {
			case 'cancel':
				return 'C';
			case 'near_future':
				// Always include the beat unit so "NF" alone (when
				// tail_beats=1) doesn't look like a different state
				// than "NF0.5"/"NF2"; trailing zeros stripped.
				return `NF${parseFloat(h.tail_beats.toFixed(2))}b`;
			case 'phrase_end':
				return 'PE';
			case 'forever':
				return 'F';
		}
	}

	function formatBeats(b: number | undefined): string {
		if (b === undefined || b === 0) return 'live';
		// Strip trailing zeros (1.50 -> 1.5) for tighter card layout.
		return `+${parseFloat(b.toFixed(2))}b`;
	}

	function transposeText(t: number): string {
		if (t === 0) return '±0';
		return t > 0 ? `+${t}` : String(t);
	}

	/** Composed accessible label so screen-reader users hear the same
	 *  info sighted users get from the card body (the button consumes
	 *  inner-span semantics by default). */
	let ariaLabel = $derived.by(() => {
		const parts = [
			`${kind} voice`,
			label,
			`transpose ${transposeText(transpose)}`,
			`output ${output}`
		];
		if (kind !== 'melody') {
			parts.push(`time offset ${formatBeats(timeOffsetBeats)}`);
			parts.push(`hold ${holdModeShort(holdMode)}`);
		}
		return parts.join(', ');
	});

	let cardTitle = $derived(
		onclick ? `${label} — click to configure` : `${label} (read-only)`
	);
</script>

<button
	class="voice-card pixel-card kind-{kind}"
	type="button"
	onclick={onclick}
	title={cardTitle}
	aria-label={ariaLabel}
>
	<div class="card-header">
		<span class="kind-badge font-ui">
			{kind === 'melody' ? 'MEL' : kind === 'canon' ? 'CAN' : 'CTP'}
			{#if kind === 'canon' && typeof index === 'number'}
				<span class="kind-index">{index + 1}</span>
			{/if}
		</span>
		<span class="card-label font-ui">{label}</span>
	</div>
	<div class="card-body font-code">
		<span class="card-row" title="Transpose (diatonic degrees)">
			<span class="card-key">T</span>{transposeText(transpose)}
		</span>
		{#if kind !== 'melody'}
			<span class="card-row" title="Time offset (beats)">
				<span class="card-key">@</span>{formatBeats(timeOffsetBeats)}
			</span>
			<span class="card-row" title="Hold mode">
				<span class="card-key">H</span>{holdModeShort(holdMode)}
			</span>
		{/if}
	</div>
	<div class="card-footer font-ui" title="Output destination">
		→ {output}
	</div>
</button>

<style>
	.voice-card {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 6px 8px;
		min-width: 110px;
		max-width: 160px;
		background: rgba(15, 14, 26, 0.7);
		border: 1px solid var(--color-border);
		cursor: pointer;
		text-align: left;
		font: inherit;
		color: inherit;
	}

	.voice-card:hover {
		background: rgba(20, 18, 36, 0.85);
	}

	.voice-card.kind-melody {
		border-color: var(--color-accent-cyan-dim, var(--color-accent-cyan));
		box-shadow: 0 0 6px rgba(51, 221, 255, 0.18);
	}

	.voice-card.kind-canon {
		border-color: var(--color-accent-gold);
		box-shadow: 0 0 6px rgba(212, 175, 55, 0.18);
	}

	.voice-card.kind-counterpoint {
		border-color: var(--color-accent-teal, #6affb8);
		box-shadow: 0 0 6px rgba(106, 255, 184, 0.18);
	}

	.card-header {
		display: flex;
		align-items: center;
		gap: 6px;
		padding-bottom: 3px;
		border-bottom: 1px solid var(--color-border);
	}

	.kind-badge {
		font-size: var(--font-size-xs);
		letter-spacing: 1px;
		text-transform: uppercase;
		padding: 1px 5px;
		background: var(--color-bg-deep);
		border: 1px solid var(--color-border);
		color: var(--color-text-secondary);
		white-space: nowrap;
	}

	.kind-melody .kind-badge {
		color: var(--color-accent-cyan);
	}
	.kind-canon .kind-badge {
		color: var(--color-accent-gold);
	}
	.kind-counterpoint .kind-badge {
		color: var(--color-accent-teal, #6affb8);
	}

	.kind-index {
		margin-left: 3px;
		opacity: 0.85;
	}

	.card-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.card-body {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 2px 6px;
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
	}

	.card-row {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		white-space: nowrap;
	}

	.card-key {
		color: var(--color-text-dim);
		min-width: 8px;
	}

	.card-footer {
		font-size: var(--font-size-xs);
		color: var(--color-accent-amber);
		border-top: 1px solid var(--color-border);
		padding-top: 3px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
</style>
