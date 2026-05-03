//! Pattern lane — rhythmic gate on held inputs.
//!
//! Owns the cell-grid pattern that decides which beats trigger harmony
//! playback for currently-held inputs. The router thread reads it per
//! tick to fire NoteOn/NoteOff on cell boundaries.
//!
//! The cell-index math is mirrored bit-identically in
//! `ui/src/lib/stores/pattern.svelte.ts`. The reference table in this
//! module's tests pins both implementations together — change one,
//! change both, run the tests.
//!

/// Beat-aligned chord-trigger pattern config. Mirrors the frontend
/// pattern store. Pushed via the `set_pattern_config` Tauri command
/// when the user edits cells / subdivision / length / input mode.
/// Read by the router thread per loop iteration to decide whether to
/// fire harmony NoteOn on cell boundaries.
#[derive(Clone, Debug)]
pub struct CompanionPattern {
    pub cells: Vec<bool>,
    pub subdivision: u8,
    pub length: u8,
    pub beats_per_bar: u8,
    pub input_mode: CompanionInputMode,
}

impl Default for CompanionPattern {
    fn default() -> Self {
        // 4 subdivision × 4 beats × 1 bar = 16 cells, all on.
        Self {
            cells: vec![true; 16],
            subdivision: 4,
            length: 1,
            beats_per_bar: 4,
            input_mode: CompanionInputMode::Live,
        }
    }
}

/// How input notes interact with the pattern.
///
/// `Quantized` is currently treated identically to `Live` in the
/// router (the input-onset MIDI buffer hasn't shipped yet). Hidden
/// from the UI picker; persisted "quantized" silently migrates to
/// "live" on hydrate. Documented intent: re-enable in INPUT_MODE_OPTIONS
/// once the buffer ships.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CompanionInputMode {
    /// Input plays freely; harmony fires only on pattern-active beats.
    #[default]
    Live,
    /// Input + harmony snap to next pattern beat. NOT YET IMPLEMENTED —
    /// router treats it as Live.
    Quantized,
    /// Input continuous; harmony NoteOff on pattern-off cells.
    Gated,
}

impl CompanionPattern {
    pub fn cell_count(&self) -> usize {
        self.subdivision.max(1) as usize
            * self.beats_per_bar.max(1) as usize
            * self.length.max(1) as usize
    }

    pub fn cell_index_at(&self, total_beats: f64) -> usize {
        let beats_per_loop = (self.beats_per_bar.max(1) as f64) * (self.length.max(1) as f64);
        if beats_per_loop <= 0.0 {
            return 0;
        }
        let position_in_loop = ((total_beats % beats_per_loop) + beats_per_loop) % beats_per_loop;
        let idx = (position_in_loop * self.subdivision.max(1) as f64).floor() as usize;
        let total = self.cell_count();
        if total == 0 {
            return 0;
        }
        idx % total
    }
}

#[cfg(test)]
mod companion_pattern_tests {
    use super::*;

    fn cfg(subdivision: u8, beats_per_bar: u8, length: u8) -> CompanionPattern {
        let cells_count = (subdivision.max(1) as usize)
            * (beats_per_bar.max(1) as usize)
            * (length.max(1) as usize);
        CompanionPattern {
            cells: vec![true; cells_count],
            subdivision,
            length,
            beats_per_bar,
            input_mode: CompanionInputMode::Live,
        }
    }

    /// Reference table pinning `cell_index_at` outputs for known
    /// `(subdivision, beats_per_bar, length, total_beats) -> idx`
    /// tuples. The same table is mirrored in
    /// `ui/src/lib/stores/pattern.svelte.ts`'s `cellIndexAt` doc
    /// comment and dev-mode self-check — keep both in lockstep.
    /// If you change this table, change both.
    #[test]
    fn cell_index_at_matches_reference_table() {
        let cases: &[(u8, u8, u8, f64, usize)] = &[
            // 4×4×1 = 16 cells (16th-note bar in 4/4)
            (4, 4, 1, 0.0, 0),
            (4, 4, 1, 0.5, 2),
            (4, 4, 1, 1.0, 4),
            (4, 4, 1, 2.0, 8),
            (4, 4, 1, 3.75, 15),
            (4, 4, 1, 4.0, 0), // wraps to start
            (4, 4, 1, 4.25, 1),
            (4, 4, 1, -0.25, 15), // negative wraps backward
            // 1×4×1 = 4 cells (quarter-note bar)
            (1, 4, 1, 0.0, 0),
            (1, 4, 1, 1.0, 1),
            (1, 4, 1, 3.5, 3),
            (1, 4, 1, 4.0, 0),
            // 8×4×2 = 64 cells (32nd-note 2-bar loop)
            (8, 4, 2, 0.0, 0),
            (8, 4, 2, 7.5, 60),
            (8, 4, 2, 8.0, 0),
            // 2×3×1 = 6 cells (eighth-note 3/4 bar)
            (2, 3, 1, 0.0, 0),
            (2, 3, 1, 1.5, 3),
            (2, 3, 1, 3.0, 0),
        ];
        for &(s, bpb, l, tb, expected) in cases {
            let got = cfg(s, bpb, l).cell_index_at(tb);
            assert_eq!(
                got, expected,
                "subdivision={}, bpb={}, length={}, total_beats={}",
                s, bpb, l, tb
            );
        }
    }

    /// Reference table for `cell_count`. Same lockstep contract as
    /// `cell_index_at_matches_reference_table` above.
    #[test]
    fn cell_count_matches_reference_table() {
        let cases: &[(u8, u8, u8, usize)] = &[
            (4, 4, 1, 16),
            (1, 4, 1, 4),
            (8, 4, 2, 64),
            (2, 3, 1, 6),
            (4, 4, 4, 64),
        ];
        for &(s, bpb, l, expected) in cases {
            let got = cfg(s, bpb, l).cell_count();
            assert_eq!(got, expected, "s={}, bpb={}, l={}", s, bpb, l);
        }
    }

    #[test]
    fn cell_index_at_handles_pathological_inputs() {
        // Zero subdivision/length/bpb get clamped to 1 by max(1).
        // Pattern with 1 cell is degenerate but stable.
        let mut c = cfg(1, 1, 1);
        c.cells = vec![true];
        assert_eq!(c.cell_index_at(0.0), 0);
        assert_eq!(c.cell_index_at(100.0), 0);
        assert_eq!(c.cell_index_at(-100.0), 0);
    }
}
