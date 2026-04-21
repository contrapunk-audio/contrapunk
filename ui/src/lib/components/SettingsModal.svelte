<script lang="ts">
	import { ui, FONT_OPTIONS } from '$lib/stores/ui.svelte';

	let fontOpen = $state(false);
	let fontTriggerEl: HTMLDivElement | undefined = $state();

	let currentFont = $derived(FONT_OPTIONS.find((o) => o.value === ui.fontMode) ?? FONT_OPTIONS[0]);

	function toggleFont() {
		fontOpen = !fontOpen;
	}

	function pickFont(value: typeof currentFont.value) {
		ui.setFontMode(value);
		fontOpen = false;
	}

	function onFontBlur(e: FocusEvent) {
		if (fontTriggerEl && !fontTriggerEl.contains(e.relatedTarget as Node)) {
			fontOpen = false;
		}
	}

	/**
	 * Preview value used while the user drags the scale slider. Committed
	 * to the actual UI scale only on `change` (pointer release) so the
	 * page doesn't re-zoom frame-by-frame during the drag.
	 */
	let previewScale = $state(ui.uiScale);

	// Keep the preview in sync when the store is updated from elsewhere
	// (e.g. localStorage restore on init).
	$effect(() => {
		previewScale = ui.uiScale;
	});

	let previewPercent = $derived(Math.round(previewScale * 100));
	let currentPercent = $derived(Math.round(ui.uiScale * 100));

	function onScaleInput(e: Event) {
		const v = Number((e.target as HTMLInputElement).value);
		if (Number.isFinite(v)) previewScale = v;
	}

	function onScaleCommit() {
		ui.setUiScale(previewScale);
	}

	function toggleFx() {
		ui.toggleAnimations();
		try {
			localStorage.setItem('contrapunk-fx', ui.animationsEnabled ? 'on' : 'off');
		} catch {
			/* localStorage unavailable */
		}
	}

	function close() {
		ui.closeSettings();
	}

	function onBackdrop(e: MouseEvent) {
		if (e.target === e.currentTarget) close();
	}

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape') close();
	}
</script>

<svelte:window onkeydown={onKey} />

{#if ui.settingsOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="backdrop" onclick={onBackdrop}>
		<div class="modal pixel-card" role="dialog" aria-modal="true" aria-label="Settings">
			<div class="modal-header">
				<span class="title font-pixel">Settings</span>
				<button class="close-btn pixel-btn font-pixel" onclick={close} title="Close (Esc)">
					×
				</button>
			</div>

			<div class="modal-body">
				<!-- Appearance section -->
				<section class="section">
					<h3 class="section-title font-pixel">Appearance</h3>

					<div class="row">
						<label class="row-label font-pixel" for="settings-ui-scale">UI scale</label>
						<div class="row-control">
							<input
								id="settings-ui-scale"
								class="scale-slider"
								type="range"
								min="0.75"
								max="2"
								step="0.05"
								value={previewScale}
								oninput={onScaleInput}
								onchange={onScaleCommit}
							/>
							<span class="value font-pixel">
								{previewPercent}%{#if previewPercent !== currentPercent}
									<span class="pending">*</span>
								{/if}
							</span>
						</div>
					</div>

					<div class="row">
						<span class="row-label font-pixel">Font</span>
						<div
							class="row-control font-select"
							bind:this={fontTriggerEl}
							onfocusout={onFontBlur}
							role="listbox"
							tabindex="-1"
						>
							<button
								class="font-trigger pixel-btn"
								class:open={fontOpen}
								onclick={toggleFont}
								type="button"
							>
								<span class="font-trigger-label font-pixel">{currentFont.label}</span>
								<span class="font-trigger-sample font-{currentFont.value}">Aa 123</span>
								<span class="font-trigger-arrow">{fontOpen ? '▴' : '▾'}</span>
							</button>

							{#if fontOpen}
								<div class="font-dropdown">
									{#each FONT_OPTIONS as opt}
										<button
											class="font-option"
											class:active={opt.value === ui.fontMode}
											onclick={() => pickFont(opt.value)}
											type="button"
										>
											<span class="font-option-label font-pixel">{opt.label}</span>
											<span class="font-option-sample font-{opt.value}">Aa 123</span>
										</button>
									{/each}
								</div>
							{/if}
						</div>
					</div>
				</section>

				<!-- Effects section -->
				<section class="section">
					<h3 class="section-title font-pixel">Effects</h3>

					<div class="row">
						<span class="row-label font-pixel">Visual FX</span>
						<div class="row-control">
							<button
								class="pixel-btn font-pixel"
								class:active={ui.animationsEnabled}
								onclick={toggleFx}
							>
								{ui.animationsEnabled ? 'On' : 'Off'}
							</button>
							<span class="hint font-pixel">
								{ui.animationsEnabled ? 'Glows + particles' : 'Reduced motion'}
							</span>
						</div>
					</div>
				</section>
			</div>
		</div>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: rgba(10, 10, 26, 0.72);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		min-width: 360px;
		max-width: 520px;
		width: 50vw;
		background: var(--color-bg-panel);
		border: 2px solid var(--color-border);
		box-shadow: 0 0 20px rgba(255, 51, 136, 0.2);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border);
	}

	.title {
		font-size: var(--font-size-sm);
		color: var(--color-accent-magenta);
		letter-spacing: 1px;
	}

	.close-btn {
		padding: 2px 8px;
		font-size: var(--font-size-sm);
		min-width: 28px;
	}

	.modal-body {
		padding: 12px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.section-title {
		font-size: var(--font-size-xs);
		color: var(--color-accent-cyan);
		letter-spacing: 1px;
		text-transform: uppercase;
		padding-bottom: 4px;
		border-bottom: 1px solid var(--color-border);
		margin: 0;
	}

	.row {
		display: grid;
		grid-template-columns: 80px 1fr;
		align-items: center;
		gap: 10px;
	}

	.row-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
	}

	.row-control {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.scale-slider {
		-webkit-appearance: none;
		appearance: none;
		flex: 1;
		height: 4px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		outline: none;
	}
	.scale-slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 10px;
		height: 14px;
		background: var(--color-accent-cyan);
		border: none;
		cursor: pointer;
	}
	.scale-slider::-moz-range-thumb {
		width: 10px;
		height: 14px;
		background: var(--color-accent-cyan);
		border: none;
		cursor: pointer;
	}

	.value {
		font-size: var(--font-size-xs);
		color: var(--color-accent-cyan);
		min-width: 42px;
		text-align: right;
	}

	.pending {
		color: var(--color-accent-magenta);
		margin-left: 2px;
	}

	/* Font dropdown */
	.font-select {
		position: relative;
		flex: 1;
	}

	.font-trigger {
		width: 100%;
		display: grid;
		grid-template-columns: 1fr auto auto;
		align-items: center;
		gap: 10px;
		padding: 6px 10px;
		font-size: var(--font-size-xs);
		text-align: left;
		background: var(--color-widget-bg);
	}

	.font-trigger.open {
		border-color: var(--color-accent-cyan);
		box-shadow: var(--glow-cyan);
	}

	.font-trigger-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.font-trigger-sample {
		font-size: var(--font-size-sm);
		color: var(--color-accent-cyan);
		white-space: nowrap;
	}

	.font-trigger-arrow {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-xs);
	}

	.font-dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		z-index: 10;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-accent-cyan);
		border-top: none;
		max-height: 240px;
		overflow-y: auto;
		box-shadow: var(--glow-cyan);
	}

	.font-option {
		width: 100%;
		display: grid;
		grid-template-columns: 1fr auto;
		align-items: center;
		gap: 10px;
		background: transparent;
		border: none;
		color: var(--color-text-primary);
		font-size: var(--font-size-xs);
		padding: 5px 10px;
		cursor: pointer;
		text-align: left;
	}

	.font-option:hover {
		background: var(--color-accent-cyan);
		color: var(--color-bg-deep);
	}

	.font-option:hover .font-option-sample {
		color: var(--color-bg-deep);
	}

	.font-option.active {
		color: var(--color-accent-magenta);
	}

	.font-option-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.font-option-sample {
		font-size: var(--font-size-sm);
		color: var(--color-accent-cyan);
		white-space: nowrap;
	}

	/* Per-face preview rules for the font selector tiles. Each preview
	   renders "Aa 123" in its own face so you can compare at a glance. */
	.font-press-start { font-family: var(--font-press-start); -webkit-font-smoothing: none; text-rendering: optimizeSpeed; }
	.font-vt323 { font-family: var(--font-vt323); -webkit-font-smoothing: antialiased; letter-spacing: 1px; }
	.font-silkscreen { font-family: var(--font-silkscreen); -webkit-font-smoothing: none; text-rendering: optimizeSpeed; }
	.font-pixelify { font-family: var(--font-pixelify); -webkit-font-smoothing: antialiased; }
	.font-dotgothic { font-family: var(--font-dotgothic); -webkit-font-smoothing: none; text-rendering: optimizeSpeed; }
	.font-jersey { font-family: var(--font-jersey); -webkit-font-smoothing: antialiased; }
	.font-tiny5 { font-family: var(--font-tiny5); -webkit-font-smoothing: none; text-rendering: optimizeSpeed; }
	.font-workbench { font-family: var(--font-workbench); -webkit-font-smoothing: antialiased; }
	.font-jetbrains { font-family: var(--font-jetbrains); -webkit-font-smoothing: antialiased; }
	.font-fira { font-family: var(--font-fira); -webkit-font-smoothing: antialiased; }
	.font-plex { font-family: var(--font-plex); -webkit-font-smoothing: antialiased; }
	.font-clean { font-family: var(--font-reading); -webkit-font-smoothing: antialiased; letter-spacing: normal; }

	.hint {
		font-size: var(--font-size-xs);
		color: var(--color-text-dim);
	}
</style>
