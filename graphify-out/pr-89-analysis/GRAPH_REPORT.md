# Graph Report - /tmp/pr89-graphify  (2026-04-29)

## Corpus Check
- Corpus is ~20,442 words - fits in a single context window. You may not need a graph.

## Summary
- 230 nodes · 581 edges · 11 communities detected
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 36 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Counterpoint Tests + Setters|Counterpoint Tests + Setters]]
- [[_COMMUNITY_HarmonyEngine Core|HarmonyEngine Core]]
- [[_COMMUNITY_Pattern Store (Frontend)|Pattern Store (Frontend)]]
- [[_COMMUNITY_Note OnOff + Engine Mutation|Note On/Off + Engine Mutation]]
- [[_COMMUNITY_Tauri Router Runtime|Tauri Router Runtime]]
- [[_COMMUNITY_Tauri Command Surface (raise_panic cluster)|Tauri Command Surface (raise_panic cluster)]]
- [[_COMMUNITY_Octave Mode + last_port_map|Octave Mode + last_port_map]]
- [[_COMMUNITY_VoiceLeading + take_pending_releases|VoiceLeading + take_pending_releases]]
- [[_COMMUNITY_Transport ↔ Pattern Coupling|Transport ↔ Pattern Coupling]]
- [[_COMMUNITY_Knob Component|Knob Component]]
- [[_COMMUNITY_HistoryStrip Component|HistoryStrip Component]]

## God Nodes (most connected - your core abstractions)
1. `HarmonyEngine` - 50 edges
2. `PatternStore` - 18 edges
3. `raise_panic()` - 12 edges
4. `run_tauri_router()` - 11 edges
5. `TransportStore` - 10 edges
6. `get_engine_state()` - 8 edges
7. `VoiceLeadingProcessor` - 7 edges
8. `handle_note_on()` - 7 edges
9. `test_mirror_port_map_restored_on_note_off()` - 6 edges
10. `counterpoint_species2_bass_position_harmony_above()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `set_interchange()` --calls--> `raise_panic()`  [EXTRACTED]
  /tmp/pr89-graphify/src-tauri/src/commands/harmony.rs → /tmp/pr89-graphify/src-tauri/src/commands/harmony.rs  _Bridges community 5 → community 1_

## Communities

### Community 0 - "Counterpoint Tests + Setters"
Cohesion: 0.11
Nodes (37): auto_key_off_never_populates_pending_releases(), block_chord_alto_position_splits_around_input(), block_chord_bass_position_redistributes_voicing_above(), chain_continues_at_low_register_edge(), counterpoint_species2_bass_position_harmony_above(), test_all_modes_with_barry_harris_scale(), test_all_modes_with_exotic_scales(), test_barry_harris_chord_tone_parity() (+29 more)

### Community 1 - "HarmonyEngine Core"
Cohesion: 0.09
Nodes (8): HarmonyEngine, test_borrowing_range_propagates(), test_interchange_enabled_propagates(), test_interchange_produces_borrowed_harmonies(), test_scale_mode_default_is_ionian(), test_set_voice_count_changes_output(), get_engine_state(), set_interchange()

### Community 2 - "Pattern Store (Frontend)"
Cohesion: 0.12
Nodes (4): PatternStore, onInputModeChange(), onLengthChange(), onSubdivisionChange()

### Community 3 - "Note On/Off + Engine Mutation"
Cohesion: 0.14
Nodes (22): test_barry_harris_note_tracking(), test_barry_harris_passing_tone_parity(), test_chained_harmonies_tracks_note_off(), test_contrary_motion_mode(), test_counterpoint_mode(), test_engine_creation(), test_engine_diatonic_thirds(), test_engine_pass_through() (+14 more)

### Community 4 - "Tauri Router Runtime"
Cohesion: 0.11
Nodes (13): get_note_state(), GuitarSignalPayload, handle_note_off(), handle_note_on(), NoteUpdatePayload, process_midi_message(), run_tauri_router(), start_routing() (+5 more)

### Community 5 - "Tauri Command Surface (raise_panic cluster)"
Cohesion: 0.14
Nodes (19): EngineStateResponse, parse_counterpoint_species(), parse_counterpoint_strictness(), parse_harmony_mode(), parse_key(), parse_octave_mode(), parse_scale_mode(), parse_voice_leading_style() (+11 more)

### Community 6 - "Octave Mode + last_port_map"
Cohesion: 0.17
Nodes (12): test_mirror_mode_anchor_aware(), test_mirror_out_of_range_skipped(), test_mirror_port_map_assignments(), test_non_mirror_port_map_is_identity(), test_octave_mode_bass_treble_split(), test_octave_mode_none_no_change(), test_octave_mode_spread(), test_vl_before_octave_mode() (+4 more)

### Community 7 - "VoiceLeading + take_pending_releases"
Cohesion: 0.2
Nodes (5): auto_key_change_queues_old_harmonies_for_release(), take_pending_releases_drains_the_queue(), test_vl_resets_on_style_change(), test_vl_style_getter_setter(), VoiceLeadingProcessor

### Community 8 - "Transport ↔ Pattern Coupling"
Cohesion: 0.18
Nodes (2): set_pattern_enabled(), TransportStore

### Community 9 - "Knob Component"
Cohesion: 1.0
Nodes (1): width

### Community 10 - "HistoryStrip Component"
Cohesion: 1.0
Nodes (0): 

## Knowledge Gaps
- **6 isolated node(s):** `width`, `PatternInputMode`, `VoiceOutputTarget`, `EngineStateResponse`, `NoteUpdatePayload` (+1 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Knob Component`** (2 nodes): `width`, `Knob.svelte`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `HistoryStrip Component`** (1 nodes): `HistoryStrip.svelte`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `HarmonyEngine` connect `HarmonyEngine Core` to `Counterpoint Tests + Setters`, `Note On/Off + Engine Mutation`, `Tauri Router Runtime`, `Octave Mode + last_port_map`, `VoiceLeading + take_pending_releases`?**
  _High betweenness centrality (0.301) - this node is a cross-community bridge._
- **Why does `PatternStore` connect `Pattern Store (Frontend)` to `Tauri Router Runtime`?**
  _High betweenness centrality (0.138) - this node is a cross-community bridge._
- **Why does `run_tauri_router()` connect `Tauri Router Runtime` to `Counterpoint Tests + Setters`, `Note On/Off + Engine Mutation`, `Octave Mode + last_port_map`?**
  _High betweenness centrality (0.136) - this node is a cross-community bridge._
- **Are the 8 inferred relationships involving `run_tauri_router()` (e.g. with `.new()` and `.set_counterpoint_beat_phase()`) actually correct?**
  _`run_tauri_router()` has 8 INFERRED edges - model-reasoned connections that need verification._
- **What connects `width`, `PatternInputMode`, `VoiceOutputTarget` to the rest of the system?**
  _6 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Counterpoint Tests + Setters` be split into smaller, more focused modules?**
  _Cohesion score 0.11 - nodes in this community are weakly interconnected._
- **Should `HarmonyEngine Core` be split into smaller, more focused modules?**
  _Cohesion score 0.09 - nodes in this community are weakly interconnected._