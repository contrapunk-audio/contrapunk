<script lang="ts">
  type Props = {
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    display?: string;
    accent?: 'gold' | 'green' | 'red';
    onChange?: (value: number) => void;
  };

  let {
    label,
    value,
    min,
    max,
    step = 0.01,
    display,
    accent = 'gold',
    onChange,
  }: Props = $props();

  let dragging = $state(false);
  let startY = 0;
  let startX = 0;
  let startValue = 0;
  let movedDuringDrag = false;
  let suppressNextClick = false;

  const normalized = $derived(Math.min(1, Math.max(0, (value - min) / Math.max(0.000001, max - min))));
  const angle = $derived(-132 + normalized * 264);
  const fill = $derived(normalized * 264);

  function clamp(v: number) {
    return Math.min(max, Math.max(min, v));
  }

  function snap(v: number) {
    const snapped = Math.round(v / step) * step;
    return Number(clamp(snapped).toFixed(4));
  }

  function commit(v: number) {
    onChange?.(snap(v));
  }

  function onPointerDown(event: PointerEvent) {
    dragging = true;
    startY = event.clientY;
    startX = event.clientX;
    startValue = value;
    movedDuringDrag = false;
    window.addEventListener('pointermove', onWindowPointerMove);
    window.addEventListener('pointerup', onWindowPointerUp, { once: true });
  }

  function onWindowPointerMove(event: PointerEvent) {
    if (!dragging) return;
    const range = max - min;
    const y = startY - event.clientY;
    const x = event.clientX - startX;
    if (Math.hypot(x, y) > 3) movedDuringDrag = true;
    const delta = (y + x * 0.35) / 145 * range;
    commit(startValue + delta);
  }

  function onWindowPointerUp() {
    dragging = false;
    suppressNextClick = movedDuringDrag;
    window.removeEventListener('pointermove', onWindowPointerMove);
  }

  function onClickStep(event: MouseEvent) {
    if (dragging || suppressNextClick) {
      suppressNextClick = false;
      return;
    }
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const direction = event.clientY < rect.top + rect.height / 2 ? 1 : -1;
    commit(value + direction * step);
  }

  function onWheel(event: WheelEvent) {
    event.preventDefault();
    const direction = event.deltaY < 0 ? 1 : -1;
    const amount = step * (event.shiftKey ? 10 : 1);
    commit(value + direction * amount);
  }

  function onKeydown(event: KeyboardEvent) {
    const large = step * 10;
    if (event.key === 'ArrowUp' || event.key === 'ArrowRight') {
      event.preventDefault();
      commit(value + step);
    } else if (event.key === 'ArrowDown' || event.key === 'ArrowLeft') {
      event.preventDefault();
      commit(value - step);
    } else if (event.key === 'PageUp') {
      event.preventDefault();
      commit(value + large);
    } else if (event.key === 'PageDown') {
      event.preventDefault();
      commit(value - large);
    } else if (event.key === 'Home') {
      event.preventDefault();
      commit(min);
    } else if (event.key === 'End') {
      event.preventDefault();
      commit(max);
    }
  }
</script>

<div class="knob" class:green={accent === 'green'} class:red={accent === 'red'}>
  <button
    type="button"
    class="dial"
    aria-label={`${label}: ${display ?? value}`}
    title={`${label}: ${display ?? value}`}
    style={`--angle:${angle}deg; --fill:${fill}deg`}
    onpointerdown={onPointerDown}
    onclick={onClickStep}
    onwheel={onWheel}
    onkeydown={onKeydown}
  >
    <span class="indicator"></span>
  </button>
  <div class="readout">
    <span>{label}</span>
    <strong>{display ?? value}</strong>
  </div>
</div>

<style>
  .knob {
    --accent: #f2b84b;
    --accent-soft: rgba(242, 184, 75, 0.18);
    display: grid;
    justify-items: center;
    gap: 6px;
    min-width: 0;
  }

  .knob.green {
    --accent: #42e2ae;
    --accent-soft: rgba(66, 226, 174, 0.16);
  }

  .knob.red {
    --accent: #ff6b53;
    --accent-soft: rgba(255, 107, 83, 0.16);
  }

  .dial {
    position: relative;
    width: 58px;
    height: 58px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    cursor: ns-resize;
    touch-action: none;
    outline: none;
    background:
      radial-gradient(circle at 45% 38%, rgba(255,255,255,0.14), transparent 24%),
      radial-gradient(circle at 50% 52%, #29251d 0 49%, transparent 50%),
      conic-gradient(from 228deg, var(--accent) 0deg var(--fill), rgba(255,255,255,0.11) var(--fill) 264deg, transparent 264deg 360deg);
    box-shadow:
      inset 0 1px 3px rgba(255,255,255,0.12),
      inset 0 -10px 18px rgba(0,0,0,0.34),
      0 10px 22px rgba(0,0,0,0.26),
      0 0 0 1px rgba(248, 241, 222, 0.13);
  }

  .dial::before {
    content: '';
    position: absolute;
    inset: 8px;
    border-radius: 50%;
    background: linear-gradient(145deg, #373025, #15130f);
    box-shadow: inset 0 1px rgba(255,255,255,0.12), inset 0 -8px 14px rgba(0,0,0,0.45);
  }

  .dial:focus-visible {
    box-shadow:
      inset 0 1px 3px rgba(255,255,255,0.12),
      inset 0 -10px 18px rgba(0,0,0,0.34),
      0 0 0 2px rgba(248, 241, 222, 0.42),
      0 0 30px var(--accent-soft);
  }

  .indicator {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 3px;
    height: 20px;
    transform-origin: 50% 85%;
    transform: translate(-50%, -84%) rotate(var(--angle));
    border-radius: 999px;
    background: var(--accent);
    box-shadow: 0 0 12px var(--accent-soft);
  }

  .indicator::after {
    content: '';
    position: absolute;
    left: 50%;
    top: 17px;
    width: 7px;
    height: 7px;
    transform: translateX(-50%);
    border-radius: 50%;
    background: var(--accent);
  }

  .readout {
    display: grid;
    justify-items: center;
    gap: 1px;
    line-height: 1;
    text-align: center;
  }

  .readout span {
    color: rgba(248, 241, 222, 0.54);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.61rem;
    letter-spacing: 0.10em;
    text-transform: uppercase;
    white-space: nowrap;
  }

  .readout strong {
    color: #f8f1de;
    font-size: 0.94rem;
    white-space: nowrap;
  }
</style>
