# 01 Raw Analysis — What Each Visualization Shows

This stage generates diagnostic plots from the raw captured dataset. The goal is to sanity-check the data *before* any feature extraction or training begins. Bad data in = bad model out, so catching problems here saves days of wasted training.

---

## waveform_grid.png

**What it shows:** A 6x4 grid. Each row is one guitar string (E2 through E4). Each column is a randomly chosen sample from that string. The raw audio waveform is plotted with time on the x-axis and amplitude on the y-axis.

**Good patterns:**
- Clear attack transient at the start (sharp spike), followed by exponential decay
- Amplitude stays within [-1, 1] with headroom (peaks around 0.3-0.7)
- Visible periodicity in the waveform (you can see the fundamental frequency)
- Higher strings (B3, E4) show faster oscillation than lower strings (E2, A2)
- Consistent shape across the 4 samples for each string

**Bad patterns:**
- Flat lines (silent/missed captures)
- Clipping: waveform hitting +1.0 or -1.0 and staying flat (square tops)
- Onset not at the start of the window (capture timing is off)
- Waveform starting partway through the window (late onset detection)
- Excessive noise floor visible before the attack
- Wildly different amplitudes across samples of the same string (inconsistent playing)

---

## amplitude_histogram.png

**What it shows:** Overlaid histograms of RMS amplitude for each string plus noise. Each color represents one string or the noise class.

**Good patterns:**
- Each string has a roughly normal (bell-shaped) distribution
- String distributions are separated from the noise distribution
- Noise distribution is clustered near zero (low RMS)
- Higher strings may have slightly lower RMS than lower strings (thinner strings)
- No samples at RMS=0 (would indicate empty captures)

**Bad patterns:**
- Noise and string distributions overlapping heavily (hard to separate)
- Bimodal distributions (two peaks) for a single string (inconsistent playing)
- Any samples with RMS > 0.5 (likely clipping or very hot input)
- String distributions clustered at very low RMS (input gain too low)
- Wide spread in noise RMS (noisy environment or cable problems)

---

## pitch_accuracy.png

**What it shows:** Scatter plot with expected MIDI note on the x-axis and detected MIDI note on the y-axis. Each dot is one sample, colored by string. The dashed diagonal line represents perfect pitch detection.

**Good patterns:**
- All dots clustered tightly along the diagonal (detected = expected)
- Points within +/-1 semitone of the diagonal are acceptable
- Clear separation between strings (no color mixing in unexpected areas)
- High-confidence detections form a tight band

**Bad patterns:**
- Dots scattered far from the diagonal (pitch detection failing)
- Octave errors: dots at expected +12 or -12 (detecting harmonics instead of fundamental)
- Clusters of points at detected=0 (pitch detection returned nothing)
- One string consistently off-diagonal (tuning issue during capture)
- Points at very low or very high MIDI values that don't belong (noise contamination)

---

## duration_histogram.png

**What it shows:** Distribution of sample durations in seconds.

**Good patterns:**
- A single tight spike at the configured duration (e.g., 0.5s)
- All samples exactly the same length (capture pipeline working correctly)

**Bad patterns:**
- Multiple peaks (inconsistent capture lengths)
- Samples shorter than expected (capture buffer underrun)
- Wide spread (variable-length captures)
- Samples at very short durations (< 0.1s) indicate failed captures

---

## sample_counts.png

**What it shows:** Bar chart with one bar per class (label), showing how many samples exist. Bars are colored by string, with noise in gray.

**Good patterns:**
- All string/fret positions have the same count (e.g., 100 each)
- Noise categories have reasonable counts (50+ each)
- Uniform bar heights across the entire chart
- No missing positions (no gaps in the bar sequence)

**Bad patterns:**
- Missing bars (skipped positions during capture)
- Uneven heights (some positions have far fewer samples)
- Zero-count positions (critical: no training data for that note)
- Noise having far fewer samples than note classes (class imbalance for noise)

---

## clipping_report.txt

**What it shows:** A text file listing every sample that is either clipped (peak amplitude > 0.99) or silent (RMS < 0.005).

**Good patterns:**
- "No clipped samples found" and "No silent samples found"
- If silent samples exist, they are all noise class (expected for ambient noise)

**Bad patterns:**
- Clipped note samples: input gain was too high, waveform is distorted
- Silent note samples: capture missed the pluck, or onset detection failed
- Many clipped samples on one string: that string is too loud relative to others
- Silent samples across multiple strings: possible cable or input issue

---

## General Workflow

1. Run the analysis
2. Check `clipping_report.txt` first -- if many samples are clipped, the entire dataset may need recapture with lower gain
3. Look at `waveform_grid.png` to visually confirm audio quality
4. Check `pitch_accuracy.png` for systematic tuning or detection problems
5. Verify `sample_counts.png` for completeness
6. Use `amplitude_histogram.png` to assess dynamic range and noise separation
7. Confirm `duration_histogram.png` shows uniform sample lengths

If everything looks clean, proceed to feature extraction (stage 02).
