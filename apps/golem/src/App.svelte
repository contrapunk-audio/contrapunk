<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke as tauriInvoke } from '@tauri-apps/api/core';
  import { listen as tauriListen } from '@tauri-apps/api/event';
  import Knob from './Knob.svelte';

  type ParamsPayload = {
    bpm: number;
    style: string;
    intensity: number;
    complexity: number;
    swing: number;
    fill_amount: number;
    follow_amount: number;
    master_gain: number;
  };

  type EngineState = {
    running: boolean;
    params: ParamsPayload;
  };

  type MeterPayload = {
    rms: number;
    onset: number;
    density: number;
    confidence: number;
    running: boolean;
    input_device?: string;
    input_channel?: number;
    active_channel?: number;
    channel_levels?: number[];
    input_blocks: number;
    raw_rms: number;
    raw_peak: number;
    raw_rms_db: number;
    raw_peak_db: number;
    normalized_energy: number;
    energy_fast: number;
    energy_slow: number;
    noise_floor_db: number;
    clipping: boolean;
  };

  let running = false;
  let busy = false;
  let error = '';
  let audioInputs: string[] = [];
  let selectedInput = '';
  let selectedChannel = 1;
  let padEl: HTMLDivElement;

  let bpm = 110;
  let style = 'rock';
  let intensity = 0.55;
  let complexity = 0.45;
  let swing = 0.08;
  let fillAmount = 0.35;
  let followAmount = 0.65;
  let masterGain = 0.55;

  let meter: MeterPayload = {
    rms: 0,
    onset: 0,
    density: 0,
    confidence: 0,
    running: false,
    input_device: '',
    input_channel: 0,
    active_channel: 0,
    channel_levels: [],
    input_blocks: 0,
    raw_rms: 0,
    raw_peak: 0,
    raw_rms_db: -120,
    raw_peak_db: -120,
    normalized_energy: 0,
    energy_fast: 0,
    energy_slow: 0,
    noise_floor_db: -80,
    clipping: false,
  };

  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

  const styles = [
    { value: 'rock', label: 'Straight Rock' },
    { value: 'half-time', label: 'Half-Time' },
    { value: 'four-on-floor', label: 'Four on the Floor' },
  ];

  function paramsPayload(): ParamsPayload {
    return {
      bpm,
      style,
      intensity,
      complexity,
      swing,
      fill_amount: fillAmount,
      follow_amount: followAmount,
      master_gain: masterGain,
    };
  }

  function meterScale(value: number, boost = 64) {
    return Math.min(1, Math.max(0, value) * boost);
  }

  function dbMeterScale(db: number, floor = -90, ceiling = -24) {
    return Math.min(1, Math.max(0, (db - floor) / (ceiling - floor)));
  }

  function formatLevel(value: number) {
    if (value <= 0.00001) return '-∞ dB';
    return `${Math.round(20 * Math.log10(value))} dB`;
  }

  function inputStatusText() {
    if (!running) return '';
    if (meter.input_blocks <= 0) return 'Waiting for input samples…';
    if ((meter.channel_levels?.length ?? 0) > 0 && Math.max(...(meter.channel_levels ?? [0])) <= 0.00001) {
      return `Input callbacks live · ${meter.channel_levels?.length ?? 0} ch · all channels silent`;
    }
    if (meter.clipping) return 'Input clipping · turn down interface gain';
    return `Input live · ch ${(meter.active_channel ?? 0) + 1} · ${Math.round(meter.raw_rms_db)} dB · energy ${Math.round(meter.normalized_energy * 100)}%`;
  }

  function applyState(state: EngineState) {
    running = state.running;
    bpm = state.params.bpm;
    style = state.params.style;
    intensity = state.params.intensity;
    complexity = state.params.complexity;
    swing = state.params.swing;
    fillAmount = state.params.fill_amount;
    followAmount = state.params.follow_amount;
    masterGain = state.params.master_gain;
  }

  let paramsPushTimer: number | undefined;

  function schedulePushParams() {
    if (paramsPushTimer) window.clearTimeout(paramsPushTimer);
    paramsPushTimer = window.setTimeout(() => {
      paramsPushTimer = undefined;
      void pushParams();
    }, 50);
  }

  async function pushParams() {
    try {
      const state = await tauriInvoke<EngineState>('set_engine_params', { params: paramsPayload() });
      running = state.running;
      error = '';
    } catch (e) {
      error = String(e);
    }
  }

  async function start() {
    busy = true;
    error = '';
    try {
      await pushParams();
      const state = await tauriInvoke<EngineState>('start_engine', {
        config: {
          input_device: selectedInput || null,
          input_channel: Math.max(0, selectedChannel - 1),
        },
      });
      applyState(state);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function stop() {
    busy = true;
    error = '';
    try {
      const state = await tauriInvoke<EngineState>('stop_engine');
      applyState(state);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function setBpm(value: number) {
    bpm = Math.round(value);
    schedulePushParams();
  }

  function setSwing(value: number) {
    swing = value;
    schedulePushParams();
  }

  function setFillAmount(value: number) {
    fillAmount = value;
    schedulePushParams();
  }

  function setFollowAmount(value: number) {
    followAmount = value;
    schedulePushParams();
  }

  function setMasterGain(value: number) {
    masterGain = value;
    schedulePushParams();
  }

  function movePad(event: PointerEvent) {
    if (!padEl) return;
    const rect = padEl.getBoundingClientRect();
    const x = Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width));
    const y = Math.min(1, Math.max(0, (event.clientY - rect.top) / rect.height));
    complexity = Number(x.toFixed(3));
    intensity = Number((1 - y).toFixed(3));
    schedulePushParams();
  }

  async function refreshInputs() {
    try {
      if (!isTauri) {
        error = 'Preview mode: open the Tauri app to access audio devices.';
        return;
      }
      audioInputs = await tauriInvoke<string[]>('list_audio_inputs');
      if (!selectedInput && audioInputs.length > 0) selectedInput = audioInputs[0];
      error = '';
    } catch (e) {
      error = String(e);
    }
  }

  async function setSelectedChannel(channel: number) {
    selectedChannel = Math.max(1, Math.min(16, Math.round(channel)));
    if (running) {
      try {
        await tauriInvoke('set_input_channel', { channel: selectedChannel - 1 });
      } catch (e) {
        error = String(e);
      }
    }
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;
    if (isTauri) {
      void tauriListen<MeterPayload>('golem-meter', (event) => {
        meter = event.payload;
      }).then((cleanup) => {
        unlisten = cleanup;
      });
    }

    void (async () => {
      await refreshInputs();
      try {
        const state = await tauriInvoke<EngineState>('get_engine_state');
        applyState(state);
      } catch {
        // Already surfaced by refreshInputs in browser-only preview.
      }
    })();

    return () => {
      unlisten?.();
      if (paramsPushTimer) window.clearTimeout(paramsPushTimer);
    };
  });
</script>

<main class="shell">
  <section class="hero">
    <div>
      <p class="eyebrow">Contrapunk Audio Lab</p>
      <h1>Golem</h1>
      <p class="subtitle">A live drummer that listens to your guitar, keeps the pulse, and leans into your energy.</p>
    </div>

    <div class="transport-card" class:running>
      <div class="status-light"></div>
      <div>
        <span>{running ? 'Playing' : 'Standing by'}</span>
        <strong>{Math.round(bpm)} BPM</strong>
      </div>
      {#if running}
        <button class="stop" disabled={busy} on:click={stop}>Stop</button>
      {:else}
        <button class="start" disabled={busy} on:click={start}>Start</button>
      {/if}
    </div>
  </section>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <section class="grid">
    <article class="panel drummer-panel">
      <div class="panel-head">
        <div>
          <p class="label">Drummer Pad</p>
          <h2>Simple ⇄ Complex / Soft ⇄ Loud</h2>
          <div class="metric-strip">
            <span>Intensity <strong>{Math.round(intensity * 100)}%</strong></span>
            <span>Complexity <strong>{Math.round(complexity * 100)}%</strong></span>
            <span>Follow <strong>{Math.round(followAmount * 100)}%</strong></span>
          </div>
        </div>
        <div class="style-select">
          <label for="style">Style</label>
          <select id="style" bind:value={style} on:change={pushParams}>
            {#each styles as item}
              <option value={item.value}>{item.label}</option>
            {/each}
          </select>
        </div>
      </div>

      <div
        class="pad"
        bind:this={padEl}
        role="slider"
        tabindex="0"
        aria-label="Drummer pad: horizontal complexity, vertical intensity"
        aria-valuemin="0"
        aria-valuemax="100"
        aria-valuenow={Math.round(complexity * 100)}
        aria-valuetext={`complexity ${Math.round(complexity * 100)} percent, intensity ${Math.round(intensity * 100)} percent`}
        on:pointerdown={(event) => {
          padEl.setPointerCapture(event.pointerId);
          movePad(event);
        }}
        on:pointermove={(event) => {
          if (event.buttons) movePad(event);
        }}
      >
        <div class="pad-grid"></div>
        <div class="pad-label top">LOUD</div>
        <div class="pad-label bottom">SOFT</div>
        <div class="pad-label left">SIMPLE</div>
        <div class="pad-label right">COMPLEX</div>
        <div
          class="puck"
          style={`left: ${complexity * 100}%; top: ${(1 - intensity) * 100}%`}
          aria-label="Drummer pad position"
        >
          <span></span>
        </div>

      </div>
    </article>

    <aside class="panel controls-panel">
      <div class="compact-head">
        <p class="label">Clock + Feel</p>
        <span>Drag up/down</span>
      </div>
      <div class="knob-bank">
        <Knob label="BPM" value={bpm} min={60} max={180} step={1} display={`${Math.round(bpm)}`} onChange={setBpm} />
        <Knob label="Swing" value={swing} min={0} max={0.7} step={0.01} display={`${Math.round(swing * 100)}%`} onChange={setSwing} />
        <Knob label="Fills" value={fillAmount} min={0} max={1} step={0.01} display={`${Math.round(fillAmount * 100)}%`} onChange={setFillAmount} accent="red" />
        <Knob label="Follow" value={followAmount} min={0} max={1} step={0.01} display={`${Math.round(followAmount * 100)}%`} onChange={setFollowAmount} accent="green" />
        <Knob label="Volume" value={masterGain} min={0} max={1.2} step={0.01} display={`${Math.round(masterGain * 100)}%`} onChange={setMasterGain} />
      </div>
    </aside>

    <article class="panel input-panel">
      <div class="panel-head compact">
        <div>
          <p class="label">Input</p>
          <h2>Follower</h2>
          {#if meter.running && meter.input_device}
            <small class="active-input">Listening: {meter.input_device} ch {(meter.active_channel ?? meter.input_channel ?? 0) + 1}</small>
          {/if}
        </div>
        <button class="ghost" on:click={refreshInputs}>Refresh</button>
      </div>

      <label class="select-control">
        <span>Input Device</span>
        <select bind:value={selectedInput}>
          {#each audioInputs as device}
            <option value={device}>{device}</option>
          {/each}
        </select>
      </label>

      <label class="select-control small">
        <span>Preferred Channel</span>
        <input
          type="number"
          min="1"
          max="16"
          value={selectedChannel}
          on:change={(event) => void setSelectedChannel(Number((event.currentTarget as HTMLInputElement).value))}
        />
      </label>

      {#if running}
        <p class="meter-status" class:silent={meter.input_blocks > 0 && Math.max(...(meter.channel_levels ?? [0])) <= 0.00001}>
          {inputStatusText()}
        </p>
      {/if}

      <div class="meters">
        <div class="meter-row">
          <span>Raw</span>
          <div class="meter"><i style={`transform: scaleX(${dbMeterScale(meter.raw_rms_db)})`}></i></div>
        </div>
        <div class="meter-row">
          <span>Energy</span>
          <div class="meter green"><i style={`transform: scaleX(${Math.min(1, meter.normalized_energy)})`}></i></div>
        </div>
        <div class="meter-row">
          <span>Onset</span>
          <div class="meter amber"><i style={`transform: scaleX(${Math.min(1, meter.onset)})`}></i></div>
        </div>
        <div class="meter-row">
          <span>Density</span>
          <div class="meter blue"><i style={`transform: scaleX(${Math.min(1, meter.density)})`}></i></div>
        </div>
        {#if meter.channel_levels?.length}
          <div class="channel-scan" aria-label="Input channel scan">
            {#each meter.channel_levels as level, i}
              <button
                type="button"
                class:active={i === meter.active_channel}
                class:preferred={i + 1 === selectedChannel}
                title={`Channel ${i + 1}: ${formatLevel(level)}`}
                on:click={() => void setSelectedChannel(i + 1)}
              >
                <i style={`transform: scaleY(${dbMeterScale(20 * Math.log10(Math.max(level, 0.000001)))})`}></i>
                <span>{i + 1}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </article>

    <article class="panel groove-panel">
      <p class="label">What v0.1 is doing</p>
      <div class="lanes">
        <div class="lane kick"><span>Kick</span>{#each Array(16) as _, i}<b class:hit={[0, 6, 8, 10].includes(i)}></b>{/each}</div>
        <div class="lane snare"><span>Snare</span>{#each Array(16) as _, i}<b class:hit={[4, 12].includes(i)}></b>{/each}</div>
        <div class="lane hats"><span>Hats</span>{#each Array(16) as _, i}<b class:hit={complexity > 0.52 || i % 2 === 0}></b>{/each}</div>
      </div>
      <p class="note">Procedural kit now. Multisample kit, velocity layers, and choke groups come next.</p>
    </article>
  </section>
</main>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(html, body, #app) {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  :global(body) {
    margin: 0;
    min-width: 920px;
    overflow: hidden;
    color: #f8f1de;
    background:
      radial-gradient(circle at 12% 0%, rgba(240, 177, 63, 0.20), transparent 34rem),
      radial-gradient(circle at 84% 18%, rgba(44, 180, 162, 0.16), transparent 30rem),
      linear-gradient(135deg, #15110d 0%, #201912 48%, #0c1414 100%);
    font-family: ui-serif, Georgia, Cambria, 'Times New Roman', serif;
  }

  :global(body::before) {
    content: '';
    position: fixed;
    inset: 0;
    pointer-events: none;
    opacity: 0.24;
    background-image: linear-gradient(rgba(255,255,255,0.04) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255,255,255,0.035) 1px, transparent 1px);
    background-size: 28px 28px;
    mask-image: radial-gradient(circle at center, black, transparent 78%);
  }

  button, input, select {
    font: inherit;
  }

  .shell {
    width: min(1180px, calc(100vw - 32px));
    height: 100vh;
    margin: 0 auto;
    padding: 16px 0;
    position: relative;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    gap: 10px;
    overflow: hidden;
  }

  .hero {
    display: flex;
    justify-content: space-between;
    gap: 20px;
    align-items: end;
    margin-bottom: 0;
  }

  .eyebrow, .label {
    margin: 0 0 8px;
    color: #d0a14a;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.72rem;
    letter-spacing: 0.18em;
    text-transform: uppercase;
  }

  h1 {
    margin: 0;
    font-size: clamp(3.75rem, 8.6vw, 7.35rem);
    line-height: 0.80;
    letter-spacing: -0.085em;
    text-transform: uppercase;
    text-shadow: 0 12px 48px rgba(0,0,0,0.45);
  }

  h2 {
    margin: 0;
    font-size: 1.15rem;
    letter-spacing: -0.03em;
  }

  .subtitle {
    max-width: 640px;
    margin: 10px 0 0;
    color: rgba(248, 241, 222, 0.70);
    font-size: 0.96rem;
  }

  .transport-card, .panel {
    border: 1px solid rgba(248, 241, 222, 0.16);
    background: rgba(18, 16, 12, 0.68);
    box-shadow: 0 28px 90px rgba(0,0,0,0.34), inset 0 1px rgba(255,255,255,0.05);
    backdrop-filter: blur(18px);
  }

  .transport-card {
    min-width: 285px;
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-radius: 24px;
  }

  .transport-card span {
    display: block;
    color: rgba(248, 241, 222, 0.58);
    font-size: 0.82rem;
  }

  .transport-card strong {
    font-size: 1.35rem;
  }

  .status-light {
    width: 15px;
    height: 15px;
    border-radius: 50%;
    background: #70665a;
    box-shadow: 0 0 0 6px rgba(112, 102, 90, 0.14);
  }

  .transport-card.running .status-light {
    background: #42e2ae;
    box-shadow: 0 0 0 6px rgba(66, 226, 174, 0.12), 0 0 30px rgba(66,226,174,0.65);
  }

  button {
    border: 0;
    border-radius: 999px;
    padding: 11px 18px;
    color: #17110a;
    background: #f2b84b;
    cursor: pointer;
    transition: transform .16s ease, opacity .16s ease;
  }

  button:hover { transform: translateY(-1px); }
  button:disabled { opacity: .55; cursor: wait; }
  button.stop { background: #ff6b53; }
  button.ghost { color: #f8f1de; background: rgba(255,255,255,0.08); }

  .error {
    position: absolute;
    z-index: 10;
    top: 104px;
    left: 0;
    right: 0;
    margin: 0;
    padding: 10px 14px;
    border: 1px solid rgba(255, 106, 83, 0.42);
    border-radius: 16px;
    color: #ffd2ca;
    background: rgba(102, 21, 12, 0.78);
    backdrop-filter: blur(12px);
  }

  .grid {
    height: 100%;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1.38fr) minmax(280px, 0.82fr);
    grid-template-rows: minmax(0, 1fr) 192px;
    gap: 12px;
    overflow: hidden;
  }

  .grid > * {
    min-width: 0;
  }

  .panel {
    min-width: 0;
    overflow: hidden;
    align-self: stretch;
    border-radius: 24px;
    padding: 14px;
  }

  .drummer-panel {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    gap: 12px;
  }

  .panel-head {
    display: flex;
    justify-content: space-between;
    gap: 14px;
    align-items: start;
    margin-bottom: 0;
  }

  .panel-head.compact { align-items: center; }

  .active-input {
    display: block;
    max-width: 250px;
    margin-top: 2px;
    overflow: hidden;
    color: rgba(66, 226, 174, 0.72);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.62rem;
    letter-spacing: 0.04em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .metric-strip {
    display: flex;
    flex-wrap: wrap;
    gap: 7px;
    margin-top: 8px;
  }

  .metric-strip span {
    display: inline-flex;
    align-items: baseline;
    gap: 6px;
    padding: 4px 8px;
    border: 1px solid rgba(248, 241, 222, 0.08);
    border-radius: 999px;
    color: rgba(248, 241, 222, 0.50);
    background: rgba(255,255,255,0.040);
    font-size: 0.70rem;
    line-height: 1;
  }

  .metric-strip strong {
    color: #f8f1de;
    font-size: 0.82rem;
  }

  .style-select, .select-control {
    display: grid;
    min-width: 0;
    gap: 8px;
    color: rgba(248, 241, 222, 0.62);
    font-size: 0.78rem;
  }

  .style-select {
    width: min(240px, 100%);
  }

  select, input[type='number'] {
    width: 100%;
    min-width: 0;
    max-width: 100%;
    color: #f8f1de;
    border: 1px solid rgba(248, 241, 222, 0.16);
    border-radius: 14px;
    background: rgba(0,0,0,0.22);
    padding: 10px 12px;
  }

  .pad {
    position: relative;
    min-height: 0;
    height: 100%;
    overflow: hidden;
    border-radius: 22px;
    border: 1px solid rgba(248, 241, 222, 0.18);
    background:
      radial-gradient(circle at calc(var(--x, 50) * 1%) calc(var(--y, 50) * 1%), rgba(242, 184, 75, 0.22), transparent 18rem),
      linear-gradient(145deg, rgba(242,184,75,0.08), rgba(63,190,172,0.08)),
      #17140f;
    cursor: crosshair;
    user-select: none;
    touch-action: none;
  }

  .pad-grid {
    position: absolute;
    inset: 0;
    background-image: linear-gradient(rgba(248,241,222,0.08) 1px, transparent 1px),
      linear-gradient(90deg, rgba(248,241,222,0.08) 1px, transparent 1px);
    background-size: 25% 25%;
  }

  .pad::after {
    content: '';
    position: absolute;
    inset: 16px;
    border: 1px dashed rgba(248, 241, 222, 0.18);
    border-radius: 18px;
  }

  .pad-label {
    position: absolute;
    z-index: 1;
    color: rgba(248, 241, 222, 0.46);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.72rem;
    letter-spacing: 0.16em;
  }

  .pad-label.top { top: 16px; left: 50%; transform: translateX(-50%); }
  .pad-label.bottom { bottom: 16px; left: 50%; transform: translateX(-50%); }
  .pad-label.left { left: 16px; top: 50%; transform: translateY(-50%) rotate(-90deg); }
  .pad-label.right { right: 16px; top: 50%; transform: translateY(-50%) rotate(90deg); }

  .puck {
    position: absolute;
    z-index: 2;
    width: 58px;
    height: 58px;
    transform: translate(-50%, -50%);
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: rgba(242, 184, 75, 0.18);
    border: 1px solid rgba(242, 184, 75, 0.78);
    box-shadow: 0 0 45px rgba(242, 184, 75, 0.45);
  }

  .puck span {
    width: 17px;
    height: 17px;
    border-radius: 50%;
    background: #f2b84b;
  }

  .controls-panel, .input-panel, .groove-panel {
    display: grid;
    gap: 9px;
  }

  .controls-panel {
    align-content: center;
  }

  .compact-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .compact-head span {
    color: rgba(248, 241, 222, 0.42);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.63rem;
    letter-spacing: 0.10em;
    text-transform: uppercase;
  }

  .knob-bank {
    display: grid;
    grid-template-columns: repeat(5, minmax(64px, 1fr));
    gap: 10px;
    align-items: center;
  }

  .input-panel {
    grid-column: 2;
    grid-row: 1;
  }

  .groove-panel {
    grid-column: 2;
    grid-row: 2;
  }

  .controls-panel {
    grid-column: 1;
    grid-row: 2;
  }

  .drummer-panel {
    grid-column: 1;
    grid-row: 1;
  }

  .select-control.small input {
    min-width: 0;
    width: 100px;
  }

  .meter-status {
    margin: -2px 0 0;
    color: rgba(248, 241, 222, 0.58);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.68rem;
    letter-spacing: 0.03em;
  }

  .meter-status.silent {
    color: #f2b84b;
  }

  .meters {
    display: grid;
    gap: 13px;
  }

  .meter-row {
    display: grid;
    grid-template-columns: 70px 1fr;
    gap: 8px;
    align-items: center;
    color: rgba(248, 241, 222, 0.58);
    font-size: .84rem;
  }

  .meter {
    height: 10px;
    overflow: hidden;
    border-radius: 999px;
    background: rgba(255,255,255,0.08);
  }

  .meter i {
    display: block;
    height: 100%;
    transform-origin: left center;
    background: linear-gradient(90deg, #3be0ad, #f2b84b);
    border-radius: inherit;
  }

  .meter.green i { background: linear-gradient(90deg, #42e2ae, #f2b84b); }
  .meter.amber i { background: linear-gradient(90deg, #ff8d54, #f2d64b); }
  .meter.blue i { background: linear-gradient(90deg, #4fb8ff, #42e2ae); }

  .channel-scan {
    display: grid;
    grid-template-columns: repeat(12, minmax(0, 1fr));
    gap: 4px;
    align-items: end;
    min-height: 48px;
    margin-top: 2px;
  }

  .channel-scan button {
    position: relative;
    height: 42px;
    min-width: 0;
    padding: 0;
    overflow: hidden;
    border: 1px solid rgba(248, 241, 222, 0.10);
    border-radius: 7px;
    background: rgba(255,255,255,0.055);
    box-shadow: none;
  }

  .channel-scan button.active {
    border-color: rgba(66, 226, 174, 0.78);
    box-shadow: 0 0 16px rgba(66, 226, 174, 0.15);
  }

  .channel-scan button.preferred::after {
    content: '';
    position: absolute;
    top: 3px;
    right: 3px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: #f2b84b;
  }

  .channel-scan i {
    position: absolute;
    inset: auto 0 0;
    height: 100%;
    transform-origin: bottom center;
    background: linear-gradient(180deg, #42e2ae, #f2b84b);
    opacity: 0.9;
  }

  .channel-scan span {
    position: relative;
    z-index: 1;
    display: block;
    padding-top: 23px;
    color: rgba(248, 241, 222, 0.72);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.58rem;
  }

  .lanes {
    display: grid;
    gap: 10px;
  }

  .lane {
    display: grid;
    grid-template-columns: 56px repeat(16, 1fr);
    gap: 5px;
    align-items: center;
    color: rgba(248, 241, 222, 0.58);
    font-size: .78rem;
  }

  .lane b {
    height: 13px;
    border-radius: 5px;
    background: rgba(255,255,255,0.07);
  }

  .lane b.hit { background: #f2b84b; box-shadow: 0 0 16px rgba(242,184,75,.28); }
  .lane.snare b.hit { background: #ff6b53; }
  .lane.hats b.hit { background: #42e2ae; }

  .note {
    margin: 0;
    color: rgba(248, 241, 222, 0.48);
    font-size: 0.9rem;
  }

  @media (max-width: 980px) {
    :global(body) {
      min-width: 0;
    }

    .shell {
      width: min(100% - 24px, 760px);
      padding: 12px 0;
    }

    .hero {
      align-items: stretch;
      flex-direction: column;
    }

    .transport-card {
      min-width: 0;
    }

    .grid {
      grid-template-columns: 1fr;
      grid-template-rows: minmax(0, 1fr) auto auto auto;
    }

    .drummer-panel,
    .input-panel,
    .groove-panel {
      grid-column: auto;
      grid-row: auto;
    }

    .panel-head {
      flex-direction: column;
    }

    .pad {
      height: clamp(160px, 28vh, 230px);
    }
  }
</style>
