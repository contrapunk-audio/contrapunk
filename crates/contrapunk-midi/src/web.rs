//! Web MIDI API backend for WASM builds.
//!
//! Provides MIDI device access in the browser via the Web MIDI API (web-sys).

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MidiAccess;

/// Requests MIDI access from the browser via `navigator.requestMIDIAccess()`.
pub async fn request_midi_access() -> Result<MidiAccess, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let navigator = window.navigator();

    let opts = web_sys::MidiOptions::new();
    opts.set_sysex(false);
    let promise = navigator.request_midi_access_with_options(&opts)?;

    let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    result.dyn_into::<MidiAccess>()
}

/// Lists available MIDI input devices as (id, name) pairs.
pub fn list_midi_inputs(access: &MidiAccess) -> Vec<(String, String)> {
    let mut inputs = Vec::new();
    let input_map = access.inputs();

    if let Some(iter) = js_sys::try_iter(&input_map).ok().flatten() {
        for entry in iter {
            if let Ok(val) = entry {
                let arr = js_sys::Array::from(&val);
                let port = arr.get(1);
                if let Ok(midi_port) = port.dyn_into::<web_sys::MidiPort>() {
                    let id = midi_port.id();
                    let name = midi_port.name().unwrap_or_else(|| "Unknown".to_string());
                    inputs.push((id, name));
                }
            }
        }
    }

    inputs
}

/// Lists available MIDI output devices as (id, name) pairs.
pub fn list_midi_outputs(access: &MidiAccess) -> Vec<(String, String)> {
    let mut outputs = Vec::new();
    let output_map = access.outputs();

    if let Some(iter) = js_sys::try_iter(&output_map).ok().flatten() {
        for entry in iter {
            if let Ok(val) = entry {
                let arr = js_sys::Array::from(&val);
                let port = arr.get(1);
                if let Ok(midi_port) = port.dyn_into::<web_sys::MidiPort>() {
                    let id = midi_port.id();
                    let name = midi_port.name().unwrap_or_else(|| "Unknown".to_string());
                    outputs.push((id, name));
                }
            }
        }
    }

    outputs
}

/// Connects to a MIDI input device and sets up a callback that pushes messages
/// to a shared queue.
pub fn connect_input(
    access: &MidiAccess,
    id: &str,
    midi_queue: Rc<RefCell<Vec<Vec<u8>>>>,
) -> Result<(), JsValue> {
    let input_map = access.inputs();
    let input: web_sys::MidiInput = input_map
        .get(id)
        .ok_or_else(|| JsValue::from_str("input port not found"))?;

    let queue = midi_queue.clone();
    let callback = Closure::<dyn FnMut(web_sys::MidiMessageEvent)>::new(
        move |event: web_sys::MidiMessageEvent| {
            if let Ok(data) = event.data() {
                queue.borrow_mut().push(data);
            }
        },
    );

    input.set_onmidimessage(Some(callback.as_ref().unchecked_ref()));
    callback.forget();

    Ok(())
}

/// Sends MIDI data to an output device by ID.
pub fn send_to_output(access: &MidiAccess, id: &str, data: &[u8]) -> Result<(), JsValue> {
    let output_map = access.outputs();
    let output: web_sys::MidiOutput = output_map
        .get(id)
        .ok_or_else(|| JsValue::from_str("output port not found"))?;

    let js_data = js_sys::Uint8Array::from(data);
    output.send(&js_data)?;

    Ok(())
}
