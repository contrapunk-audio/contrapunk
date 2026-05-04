# Graph Report - /tmp/pr89-graphify-v2  (2026-04-29)

## Corpus Check
- Corpus is ~21,996 words - fits in a single context window. You may not need a graph.

## Summary
- 235 nodes · 591 edges · 15 communities detected
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 35 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_HarmonyEngine Core|HarmonyEngine Core]]
- [[_COMMUNITY_Pattern Store (Frontend)|Pattern Store (Frontend)]]
- [[_COMMUNITY_Tauri Command Surface (raise_panic cluster)|Tauri Command Surface (raise_panic cluster)]]
- [[_COMMUNITY_Engine Setters + Borrowing|Engine Setters + Borrowing]]
- [[_COMMUNITY_Tauri Router Runtime|Tauri Router Runtime]]
- [[_COMMUNITY_Engine Tests + Voice Count|Engine Tests + Voice Count]]
- [[_COMMUNITY_Auto-Key + Note OnOff + Pending Releases|Auto-Key + Note On/Off + Pending Releases]]
- [[_COMMUNITY_Octave Mode + Mode Tests|Octave Mode + Mode Tests]]
- [[_COMMUNITY_Block Chord + Voice Position|Block Chord + Voice Position]]
- [[_COMMUNITY_PatternConfig + HeldVoice + Cell Tests|PatternConfig + HeldVoice + Cell Tests]]
- [[_COMMUNITY_Transport Store|Transport Store]]
- [[_COMMUNITY_VoiceLeading|VoiceLeading]]
- [[_COMMUNITY_Counterpoint Species|Counterpoint Species]]
- [[_COMMUNITY_Knob Component|Knob Component]]
- [[_COMMUNITY_HistoryStrip Component|HistoryStrip Component]]

## God Nodes (most connected - your core abstractions)
1. `HarmonyEngine` - 50 edges
2. `PatternStore` - 18 edges
3. `raise_panic()` - 13 edges
4. `run_tauri_router()` - 11 edges
5. `TransportStore` - 10 edges
6. `get_engine_state()` - 8 edges
7. `VoiceLeadingProcessor` - 7 edges
8. `handle_note_on()` - 7 edges
9. `test_mirror_port_map_restored_on_note_off()` - 6 edges
10. `counterpoint_species2_bass_position_harmony_above()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `set_interchange()` --calls--> `raise_panic()`  [EXTRACTED]
  /tmp/pr89-graphify-v2/src-tauri/src/commands/harmony.rs → /tmp/pr89-graphify-v2/src-tauri/src/commands/harmony.rs  _Bridges community 2 → community 3_

## Communities

### Community 0 - "HarmonyEngine Core"
Cohesion: 0.1
Nodes (3): HarmonyEngine, test_set_counterpoint_strictness_propagates_to_states(), get_engine_state()

### Community 1 - "Pattern Store (Frontend)"
Cohesion: 0.12
Nodes (4): PatternStore, onInputModeChange(), onLengthChange(), onSubdivisionChange()

### Community 2 - "Tauri Command Surface (raise_panic cluster)"
Cohesion: 0.13
Nodes (20): EngineStateResponse, parse_counterpoint_species(), parse_counterpoint_strictness(), parse_harmony_mode(), parse_key(), parse_octave_mode(), parse_scale_mode(), parse_voice_leading_style() (+12 more)

### Community 3 - "Engine Setters + Borrowing"
Cohesion: 0.16
Nodes (11): test_barry_harris_chromatic(), test_barry_harris_produces_5_notes(), test_barry_harris_scale_guard_auto_switch(), test_barry_harris_scale_guard_minor(), test_barry_harris_scale_guard_restore(), test_borrowing_range_propagates(), test_interchange_enabled_propagates(), test_scale_mode_default_is_ionian() (+3 more)

### Community 4 - "Tauri Router Runtime"
Cohesion: 0.14
Nodes (12): get_note_state(), GuitarSignalPayload, handle_note_off(), handle_note_on(), NoteUpdatePayload, process_midi_message(), run_tauri_router(), start_routing() (+4 more)

### Community 5 - "Engine Tests + Voice Count"
Cohesion: 0.19
Nodes (15): test_barry_harris_chord_tone_parity(), test_barry_harris_passing_tone_parity(), test_contrary_motion_mode(), test_counterpoint_beat_phase_roundtrip(), test_counterpoint_mode(), test_engine_creation(), test_engine_diatonic_thirds(), test_engine_pass_through() (+7 more)

### Community 6 - "Auto-Key + Note On/Off + Pending Releases"
Cohesion: 0.18
Nodes (15): auto_key_change_queues_old_harmonies_for_release(), auto_key_off_never_populates_pending_releases(), take_pending_releases_drains_the_queue(), test_barry_harris_note_tracking(), test_chained_harmonies_tracks_note_off(), test_interchange_produces_borrowed_harmonies(), test_key_change_clears_tracking(), test_mirror_note_off_releases_all_duplicates() (+7 more)

### Community 7 - "Octave Mode + Mode Tests"
Cohesion: 0.14
Nodes (16): test_all_modes_with_barry_harris_scale(), test_all_modes_with_exotic_scales(), test_chained_counterpoint_has_independent_state(), test_chained_harmonies_with_thirds(), test_diatonic_thirds_on_bh_scale_chord_tone_parity(), test_diatonic_thirds_on_bh_scale_passing_tone_parity(), test_mirror_mode_anchor_aware(), test_mirror_out_of_range_skipped() (+8 more)

### Community 8 - "Block Chord + Voice Position"
Cohesion: 0.24
Nodes (12): block_chord_alto_position_splits_around_input(), block_chord_bass_position_redistributes_voicing_above(), chain_continues_at_low_register_edge(), test_vl_disabled_by_default(), test_voice_position_clamped_on_set(), test_voice_position_clamped_on_voice_count_change(), test_voice_position_default_is_bass(), test_voice_position_middle_generates_both_directions() (+4 more)

### Community 9 - "PatternConfig + HeldVoice + Cell Tests"
Cohesion: 0.26
Nodes (8): cell_count_matches_reference_table(), cell_index_at_handles_pathological_inputs(), cell_index_at_matches_reference_table(), cfg(), HeldVoice, PatternConfig, PatternInputMode, VoiceOutputTarget

### Community 10 - "Transport Store"
Cohesion: 0.2
Nodes (1): TransportStore

### Community 11 - "VoiceLeading"
Cohesion: 0.22
Nodes (6): test_vl_enabled_produces_output(), test_vl_melody_never_modified(), test_vl_resets_on_key_change(), test_vl_resets_on_mode_change(), test_vl_resets_on_style_change(), test_vl_works_with_all_modes()

### Community 12 - "Counterpoint Species"
Cohesion: 0.4
Nodes (4): counterpoint_species2_bass_position_harmony_above(), test_set_counterpoint_species_propagates_to_states(), test_species1_ignores_beat_phase(), test_species_change_alters_harmony_output()

### Community 13 - "Knob Component"
Cohesion: 1.0
Nodes (1): width

### Community 14 - "HistoryStrip Component"
Cohesion: 1.0
Nodes (0): 

## Knowledge Gaps
- **7 isolated node(s):** `width`, `PatternInputMode`, `VoiceOutputTarget`, `HeldVoice`, `EngineStateResponse` (+2 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Knob Component`** (2 nodes): `width`, `Knob.svelte`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `HistoryStrip Component`** (1 nodes): `HistoryStrip.svelte`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `HarmonyEngine` connect `HarmonyEngine Core` to `Engine Setters + Borrowing`, `Tauri Router Runtime`, `Engine Tests + Voice Count`, `Auto-Key + Note On/Off + Pending Releases`, `Octave Mode + Mode Tests`, `Block Chord + Voice Position`, `VoiceLeading`, `Counterpoint Species`?**
  _High betweenness centrality (0.275) - this node is a cross-community bridge._
- **Why does `run_tauri_router()` connect `Tauri Router Runtime` to `PatternConfig + HeldVoice + Cell Tests`, `Counterpoint Species`, `Auto-Key + Note On/Off + Pending Releases`?**
  _High betweenness centrality (0.162) - this node is a cross-community bridge._
- **Why does `PatternStore` connect `Pattern Store (Frontend)` to `Tauri Router Runtime`?**
  _High betweenness centrality (0.129) - this node is a cross-community bridge._
- **Are the 8 inferred relationships involving `run_tauri_router()` (e.g. with `.new()` and `.set_counterpoint_beat_phase()`) actually correct?**
  _`run_tauri_router()` has 8 INFERRED edges - model-reasoned connections that need verification._
- **What connects `width`, `PatternInputMode`, `VoiceOutputTarget` to the rest of the system?**
  _7 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `HarmonyEngine Core` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Should `Pattern Store (Frontend)` be split into smaller, more focused modules?**
  _Cohesion score 0.12 - nodes in this community are weakly interconnected._