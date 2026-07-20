## Review

- **Blocker:** `crates/contrapunk-companion/src/counterpoint_lane.rs:164-169` — changing species clears `held` without emitting NoteOffs for already-sounding Species 1–3 notes. Their later input NoteOff can no longer find ownership, leaving stuck notes. This regression is introduced by the diff.
- **Blocker:** `crates/contrapunk-companion/src/counterpoint_lane.rs:676-695` — transport rewind clears the generic `held` ledger without releasing its sounding notes, while failing to clear future `pending_off` deadlines. This can both strand old notes and later let stale NoteOffs cut notes on the new timeline.
- **Note:** The main Species 4 test at `counterpoint_lane.rs:969-1060` does substantiate weak E preparation, retained E against live F, legato ownership transfer, and downward resolution to D with balanced NoteOff/On. Disable and Species 4 rewind paths are also tested.
- **Note:** Tests do not cover the two blockers above or sounded Species 4 Hold deadline expiration.