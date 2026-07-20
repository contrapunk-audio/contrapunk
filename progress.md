# Progress

## Status
In Progress

## Tasks

## Files Changed

## Notes

### Logic AU debugging rule (2026-07-11)
Do not make "lazy" moves here. Progress means deterministic evidence, not shortcuts:
- no blind rebuild/reinstall loops
- no cache wiping unless paired with a clean registration/validation plan
- no aborting scans if it prevents Contrapunk from being registered
- no patching Svelte, AU wrapper, or CLAP wrapper until the exact failing layer is proven
- every step must end with one observable fact: installed hash, codesign state, cache/scan state, real UI request path, crash report attribution, or Logic UI screenshot

Current known facts:
- signed `/Library/Audio/Plug-Ins/Components/Contrapunk.component` has matched the intended signed build during installer verification
- Contrapunk's AU wrapper contains CocoaUI and parent-deferral symbols
- Contrapunk's embedded CLAP contains Svelte assets and the debug `Not Found: path=... uri=...` response
- earlier `Not Found` proved the real webview could open; current generic sliders mean Logic is falling back to its parameter editor / stale or incomplete AU registration
- the 13:13 crash was Auto-Tune+Time_AU, not Contrapunk

Next progress step: establish a clean Contrapunk AU registration/custom-editor state, then open a fresh Contrapunk AU instance and capture either the Svelte UI or the exact debug `Not Found: path=... uri=...` request.

### REAPER x86_64 plugin UI validation (2026-07-11)
- Watcher alert at check 132/144: Logic not running, no new crash in the latest process output.
- REAPER first loaded x86_64 CLAP but showed `Not Found: path="/index.html" uri="contrapunk://localhost/"`.
- Root cause was `CONTRAPUNK_PLUGIN_UI_DIR=ui/build` resolving relative to `plugin/`, producing an empty generated asset map.
- Patched `plugin/build.rs` to resolve relative UI dirs against the workspace and fail `embed-ui` builds when the UI dir is missing/empty.
- Rebuilt x86_64 CLAP/VST3 with embedded assets, installed hash `3db9159e256bae32e481b7be0d37cbc807cfd5cfeff15482d40cb30fbcd18f77` to user CLAP/VST3 paths.
- REAPER reopened fresh CLAP and displayed the real Svelte/WebView UI (Contrapunk Harmony panel), no crash.

### Logic 12-hour watcher completed (2026-07-11)
- `contrapunk-logic-12h-watch` (`proc_3`) completed successfully after 144/144 checks.
- Run window: `Sat Jul 11 03:47:05 IST 2026` → `Sat Jul 11 15:47:52 IST 2026`.
- Final check: `check 144/144 logic_pid=46027 window=unknown`.
- No `NEW_CRASH` after the previously inspected `14:07:44` Auto-Tune+Time_AU crash.

### AU UI research/local evidence (2026-07-11)
- Apple AUv2 custom UI path is `kAudioUnitProperty_CocoaUI` → `AUCocoaUIBase` → `uiViewForAudioUnit:withSize:` returning an `NSView`; hosts show generic controls when no usable custom view is available.
- Public reports match our symptom class: `Cocoa Views Available: 0` / Logic not loading UI, wrapper class/bundle collisions, and AUHostingServiceXPC hosting crashes.
- Local current system AU wrapper advertises Cocoa UI symbols, but its nested `/Library/.../Contrapunk.component/.../Contrapunk.clap` does not contain `_app/immutable` or `Not Found: path=` markers, while the REAPER-fixed user CLAP does. Current Logic AU is therefore not yet the same embedded-asset build that REAPER validated.
