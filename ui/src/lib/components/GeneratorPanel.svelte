<script lang="ts">
	// Generator modes and chord types matching src/generator/config.rs
	type GeneratorModeName =
		| 'HeldNotes'
		| 'Chord'
		| 'ArpeggioUp'
		| 'ArpeggioDown'
		| 'ArpeggioUpDown'
		| 'ScaleRunner'
		| 'RandomDiatonic';

	type ChordTypeName =
		| 'Major'
		| 'Minor'
		| 'Dim'
		| 'Aug'
		| 'Maj7'
		| 'Min7'
		| 'Dom7'
		| 'Dim7'
		| 'HalfDim7';

	const GENERATOR_MODES: { name: GeneratorModeName; label: string }[] = [
		{ name: 'HeldNotes', label: 'Held' },
		{ name: 'Chord', label: 'Chord' },
		{ name: 'ArpeggioUp', label: 'Arp Up' },
		{ name: 'ArpeggioDown', label: 'Arp Dn' },
		{ name: 'ArpeggioUpDown', label: 'Arp UD' },
		{ name: 'ScaleRunner', label: 'Scale' },
		{ name: 'RandomDiatonic', label: 'Random' },
	];

	const CHORD_TYPES: { name: ChordTypeName; label: string }[] = [
		{ name: 'Major', label: 'Maj' },
		{ name: 'Minor', label: 'Min' },
		{ name: 'Dom7', label: '7th' },
		{ name: 'Maj7', label: 'Maj7' },
		{ name: 'Min7', label: 'Min7' },
		{ name: 'Dim', label: 'Dim' },
		{ name: 'Aug', label: 'Aug' },
		{ name: 'Dim7', label: 'Dim7' },
		{ name: 'HalfDim7', label: 'Half7' },
	];

	let generatorEnabled = $state(false);
	let selectedMode = $state<GeneratorModeName>('HeldNotes');
	let selectedChordType = $state<ChordTypeName>('Major');

	function toggleGenerator() {
		generatorEnabled = !generatorEnabled;
	}

	function selectMode(mode: GeneratorModeName) {
		selectedMode = mode;
	}

	function selectChordType(ct: ChordTypeName) {
		selectedChordType = ct;
	}

	// Is a chord mode
	function isChordMode(): boolean {
		return selectedMode === 'Chord';
	}

	// Is an arpeggio mode
	function isArpMode(): boolean {
		return selectedMode === 'ArpeggioUp' || selectedMode === 'ArpeggioDown' || selectedMode === 'ArpeggioUpDown';
	}
</script>

<div class="card">
	<div class="card-header font-pixel">Generator</div>
	<div class="toggle-row">
		<button
			class="pixel-btn toggle-btn"
			class:toggle-on={generatorEnabled}
			onclick={toggleGenerator}
		>
			{generatorEnabled ? 'ON' : 'OFF'}
		</button>
	</div>

	{#if generatorEnabled}
		<!-- Mode selector grid -->
		<div class="mode-grid">
			{#each GENERATOR_MODES as mode}
				<button
					class="pixel-btn gen-btn"
					class:gen-active={selectedMode === mode.name}
					onclick={() => selectMode(mode.name)}
				>
					{mode.label}
				</button>
			{/each}
		</div>

		<!-- Chord type selector (when Chord mode) -->
		{#if isChordMode()}
			<div class="sub-section">
				<div class="sub-header font-pixel">Chord Type</div>
				<div class="chord-grid">
					{#each CHORD_TYPES as ct}
						<button
							class="pixel-btn gen-btn"
							class:gen-active={selectedChordType === ct.name}
							onclick={() => selectChordType(ct.name)}
						>
							{ct.label}
						</button>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Arpeggio direction indicator (when Arp mode) -->
		{#if isArpMode()}
			<div class="sub-section">
				<div class="sub-header font-pixel">Direction</div>
				<div class="direction-grid">
					<button
						class="pixel-btn gen-btn"
						class:gen-active={selectedMode === 'ArpeggioUp'}
						onclick={() => selectMode('ArpeggioUp')}
					>
						Up
					</button>
					<button
						class="pixel-btn gen-btn"
						class:gen-active={selectedMode === 'ArpeggioDown'}
						onclick={() => selectMode('ArpeggioDown')}
					>
						Down
					</button>
					<button
						class="pixel-btn gen-btn"
						class:gen-active={selectedMode === 'ArpeggioUpDown'}
						onclick={() => selectMode('ArpeggioUpDown')}
					>
						UpDn
					</button>
				</div>
			</div>
		{/if}
	{/if}
</div>

<style>
	.card {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		padding: 6px;
		margin-bottom: 4px;
		border-radius: 0;
	}

	.card-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		margin-bottom: 4px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.toggle-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.toggle-btn {
		min-width: 40px;
	}

	.toggle-btn.toggle-on {
		background: var(--color-accent-teal);
		border-color: var(--color-accent-cyan);
		box-shadow: var(--glow-teal);
		color: #ffffff;
	}

	.mode-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 2px;
		margin-top: 6px;
	}

	.gen-btn {
		font-size: 6px !important;
		padding: 4px 4px !important;
	}

	.gen-btn.gen-active {
		background: var(--color-accent-teal);
		border-color: var(--color-accent-cyan);
		box-shadow: var(--glow-teal);
		color: #ffffff;
	}

	.sub-section {
		margin-top: 6px;
		padding-top: 4px;
		border-top: 1px solid var(--color-border);
	}

	.sub-header {
		color: var(--color-text-secondary);
		font-size: 6px;
		margin-bottom: 4px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.chord-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 2px;
	}

	.direction-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 2px;
	}
</style>
