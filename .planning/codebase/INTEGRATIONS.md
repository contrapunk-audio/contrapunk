# Integrations — Contrapunk
**Generated:** 2026-04-04
Contrapunk is local-first. No cloud APIs, databases, or auth.
## MIDI I/O
Native: midir 0.10 for MIDI ports. Browser: Web MIDI API via web-sys.
Virtual inputs: Computer keyboard (999998), Guitar Audio (999997), Note Generator (999999)
## Audio I/O
Native: cpal 0.15. Browser: getUserMedia + ScriptProcessorNode -> WASM DSP.
## Deployment
Fly.io (fly.toml, deploy/Dockerfile), nginx, GitHub Actions CI/CD
## Tauri IPC
list_midi_inputs/outputs, start/stop_routing, get/set engine state,
list_audio_devices, note-update event (~30fps)
## Storage
localStorage on both platforms. No external databases.
