//! `ClapAudioBlock` — an [`AudioBlock`] that drives a live CLAP plugin.
//!
//! Receives a `StartedPluginAudioProcessor` from the main thread and
//! pumps audio + MIDI through it every buffer.
//!
//! # Port layout
//!
//! The block is constructed with a per-port channel-count list for
//! both sides (obtained from the plugin's `audio_ports` extension on
//! the main thread). That's non-negotiable — if we hand the plugin
//! fewer ports or channels than it declared, its `process` will
//! dereference garbage and segfault (FabFilter Pro-C 2 hit this with
//! its sidechain input).
//!
//! Main port = index 0. Audio from the upstream chain goes into main
//! input's first channels; everything else (sidechains, multi-out
//! busses) is silent. Main output's first channels go back into the
//! chain buffer.

use std::sync::atomic::{AtomicUsize, Ordering};

use clack_host::events::event_types::{NoteOffEvent, NoteOnEvent};
use clack_host::events::{Match, Pckn};
use clack_host::prelude::*;

use crate::chain::{AudioBlock, MidiBlockEvent};

use super::controller::PortLayout;
use super::host::ContrapunkHost;

const MIDI_RING_CAPACITY: usize = 256;

struct MidiRing {
    buf: [MidiBlockEvent; MIDI_RING_CAPACITY],
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl MidiRing {
    const fn new() -> Self {
        Self {
            buf: [MidiBlockEvent::AllNotesOff; MIDI_RING_CAPACITY],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    fn push(&mut self, ev: MidiBlockEvent) {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        let next = (head + 1) % MIDI_RING_CAPACITY;
        if next == tail {
            let new_tail = (tail + 1) % MIDI_RING_CAPACITY;
            self.tail.store(new_tail, Ordering::Release);
        }
        self.buf[head] = ev;
        self.head.store(next, Ordering::Release);
    }

    fn drain_into(&mut self, out: &mut Vec<MidiBlockEvent>) {
        let head = self.head.load(Ordering::Acquire);
        let mut tail = self.tail.load(Ordering::Relaxed);
        while tail != head {
            out.push(self.buf[tail]);
            tail = (tail + 1) % MIDI_RING_CAPACITY;
        }
        self.tail.store(tail, Ordering::Release);
    }
}

pub struct ClapAudioBlock {
    name: String,
    processor: StartedPluginAudioProcessor<ContrapunkHost>,
    sample_rate: u32,
    max_frames: usize,
    layout: PortLayout,
    input_ports: AudioPorts,
    output_ports: AudioPorts,
    /// One flat buffer per port. Sized to `channel_count * max_frames`.
    input_port_bufs: Vec<Vec<f32>>,
    output_port_bufs: Vec<Vec<f32>>,
    event_buf: EventBuffer,
    midi_ring: MidiRing,
    drained: Vec<MidiBlockEvent>,
    steady_counter: u64,
    enabled: bool,
}

impl ClapAudioBlock {
    pub fn new(
        name: String,
        processor: StartedPluginAudioProcessor<ContrapunkHost>,
        sample_rate: u32,
        max_frames: usize,
        layout: PortLayout,
    ) -> Self {
        let total_in_channels: usize = layout.inputs.iter().map(|&c| c as usize).sum();
        let total_out_channels: usize = layout.outputs.iter().map(|&c| c as usize).sum();
        let input_port_bufs = layout
            .inputs
            .iter()
            .map(|&c| vec![0.0f32; c as usize * max_frames])
            .collect();
        let output_port_bufs = layout
            .outputs
            .iter()
            .map(|&c| vec![0.0f32; c as usize * max_frames])
            .collect();
        eprintln!(
            "[clap/audio_block] new: name={} in_ports={} ({} ch) out_ports={} ({} ch)",
            name,
            layout.inputs.len(),
            total_in_channels,
            layout.outputs.len(),
            total_out_channels
        );
        Self {
            name,
            processor,
            sample_rate,
            max_frames,
            input_ports: AudioPorts::with_capacity(
                total_in_channels.max(1),
                layout.inputs.len().max(1),
            ),
            output_ports: AudioPorts::with_capacity(
                total_out_channels.max(1),
                layout.outputs.len().max(1),
            ),
            layout,
            input_port_bufs,
            output_port_bufs,
            event_buf: EventBuffer::with_capacity(MIDI_RING_CAPACITY),
            midi_ring: MidiRing::new(),
            drained: Vec::with_capacity(MIDI_RING_CAPACITY),
            steady_counter: 0,
            enabled: true,
        }
    }

    fn ensure_scratch(&mut self, frames: usize) {
        if frames > self.max_frames {
            self.max_frames = frames;
            for (buf, &ch) in self.input_port_bufs.iter_mut().zip(&self.layout.inputs) {
                buf.resize(ch as usize * frames, 0.0);
            }
            for (buf, &ch) in self.output_port_bufs.iter_mut().zip(&self.layout.outputs) {
                buf.resize(ch as usize * frames, 0.0);
            }
        }
    }

    fn translate_events(&mut self) {
        self.event_buf.clear();
        self.drained.clear();
        self.midi_ring.drain_into(&mut self.drained);
        for ev in self.drained.drain(..) {
            match ev {
                MidiBlockEvent::NoteOn { note, velocity } => {
                    let pckn = Pckn::new(0u16, 0u16, note as u16, Match::All);
                    let velocity = (velocity as f64 / 127.0).clamp(0.0, 1.0);
                    self.event_buf.push(&NoteOnEvent::new(0, pckn, velocity));
                }
                MidiBlockEvent::NoteOff { note } => {
                    let pckn = Pckn::new(0u16, 0u16, note as u16, Match::All);
                    self.event_buf.push(&NoteOffEvent::new(0, pckn, 0.0));
                }
                MidiBlockEvent::AllNotesOff => {
                    let pckn = Pckn::new(0u16, 0u16, Match::All, Match::All);
                    self.event_buf.push(&NoteOffEvent::new(0, pckn, 0.0));
                }
            }
        }
    }
}

impl AudioBlock for ClapAudioBlock {
    fn name(&self) -> &str {
        &self.name
    }

    fn type_id(&self) -> &str {
        "clap.audio"
    }

    fn process(&mut self, buffer: &mut [f32], channels: usize) {
        if channels == 0 || buffer.is_empty() || !self.enabled {
            return;
        }

        let frames = buffer.len() / channels;
        self.ensure_scratch(frames);

        // Deinterleave host buffer → main input port's first channels.
        if let Some(&main_in_ch) = self.layout.inputs.first() {
            let stride = self.max_frames;
            let main_in_buf = &mut self.input_port_bufs[0];
            let use_ch = (main_in_ch as usize).min(channels);
            for ch in 0..use_ch {
                let dst_start = ch * stride;
                for f in 0..frames {
                    main_in_buf[dst_start + f] = buffer[f * channels + ch];
                }
            }
            // If the plugin's main input has more channels than host,
            // fill remaining channels from the last host channel
            // (mono→stereo style).
            let src_ch = channels.min(use_ch).saturating_sub(1);
            for ch in use_ch..(main_in_ch as usize) {
                let dst_start = ch * stride;
                for f in 0..frames {
                    main_in_buf[dst_start + f] = buffer[f * channels + src_ch.min(channels - 1)];
                }
            }
        }
        // Silence non-main input ports (sidechains unused for now).
        for port_idx in 1..self.input_port_bufs.len() {
            for v in &mut self.input_port_bufs[port_idx] {
                *v = 0.0;
            }
        }
        // Zero output buffers (plugin writes into them).
        for out in self.output_port_bufs.iter_mut() {
            for v in out.iter_mut() {
                *v = 0.0;
            }
        }

        self.translate_events();

        let max_frames = self.max_frames;
        let layout = &self.layout;
        let input_audio = self.input_ports.with_input_buffers(
            self.input_port_bufs
                .iter_mut()
                .zip(&layout.inputs)
                .map(|(port_buf, &ch)| AudioPortBuffer {
                    latency: 0,
                    channels: AudioPortBufferType::f32_input_only(
                        port_buf.chunks_exact_mut(max_frames).map(|c| InputChannel {
                            buffer: &mut c[..frames],
                            is_constant: false,
                        }),
                    ),
                }),
        );
        let mut output_audio = self.output_ports.with_output_buffers(
            self.output_port_bufs
                .iter_mut()
                .zip(&layout.outputs)
                .map(|(port_buf, _ch)| AudioPortBuffer {
                    latency: 0,
                    channels: AudioPortBufferType::f32_output_only(
                        port_buf
                            .chunks_exact_mut(max_frames)
                            .map(|c| &mut c[..frames]),
                    ),
                }),
        );

        let events = InputEvents::from_buffer(&self.event_buf);
        let mut out_events = EventBuffer::new();
        let mut out_events_view = OutputEvents::from_buffer(&mut out_events);
        let _ = self.processor.process(
            &input_audio,
            &mut output_audio,
            &events,
            &mut out_events_view,
            Some(self.steady_counter),
            None,
        );

        self.steady_counter = self.steady_counter.wrapping_add(frames as u64);

        // Interleave main output port → host buffer.
        if let Some(&main_out_ch) = self.layout.outputs.first() {
            let stride = self.max_frames;
            let main_out_buf = &self.output_port_bufs[0];
            let plugin_ch = main_out_ch as usize;
            for f in 0..frames {
                for host_ch in 0..channels {
                    let plug_ch = host_ch.min(plugin_ch.saturating_sub(1));
                    buffer[f * channels + host_ch] = main_out_buf[plug_ch * stride + f];
                }
            }
        }
    }

    fn midi_event(&mut self, event: MidiBlockEvent) {
        self.midi_ring.push(event);
    }

    fn reset(&mut self) {
        self.midi_ring = MidiRing::new();
        self.drained.clear();
        self.event_buf.clear();
        self.steady_counter = 0;
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

const _: fn() = || {
    fn assert_send<T: Send>() {}
    assert_send::<ClapAudioBlock>();
};
