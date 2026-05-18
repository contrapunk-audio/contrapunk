---
name: vital-preset-import
description: Import or map Vital synth presets/banks into Elixir. Use when touching .vital/.vitalbank files, Downloads Vital assets, B5 preset format, wavetable import, or Vital compatibility.
---

# Vital Preset Import

## Observed Files

`~/Downloads` currently contains individual `.vital` JSON presets and `Vital Account.vitalbank`.

- `.vital` = JSON object with top-level metadata (`author`, `comments`, `preset_name`, `preset_style`, `synth_version`) and a large `settings` dictionary.
- `.vitalbank` = ZIP archive containing folders, `.vital` presets, `.vitaltable` wavetables, and `details.json`.

## Import Strategy

1. Parse metadata first; never require full DSP parity for basic import.
2. Preserve raw Vital JSON/settings for round-trip/debug even when Elixir only maps a subset.
3. Map common scalar controls into Elixir's current model:
   - `chorus_*`, `delay_*`, `reverb_*`, `compressor_*` → Elixir FX params
   - filter cutoff/resonance/drive-ish fields → Elixir filter params
   - oscillator/wavetable references → queued for B7 full wavetable parity
4. Treat `.vitalbank` as a batch source of `.vital` files plus `.vitaltable` assets.
5. Wavetable import is not complete until B7 adds real FFT/IFFT/wavetable frame infrastructure.

## Validation

- Always test against the actual Downloads assets:
  - six standalone `.vital` files from May 18
  - `Vital Account.vitalbank` with 75 `.vital` entries
- Count imported presets and surface skipped/unsupported settings honestly.
- Do not silently claim full Vital sound parity while scalar A6 spectral approximations are still in place.
