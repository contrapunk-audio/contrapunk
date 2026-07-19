use contrapunk_audio::guitar_input::{GuitarInput, GuitarInputConfig, MidiEvent};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const STRING_BASE_MIDI: [u8; 6] = [40, 45, 50, 55, 59, 64];
const CHUNK_SIZE: usize = 128;
const RELEASE_TAIL_SECONDS: usize = 1;
const HOLDOUT: &str = include_str!("guitar_corpus_holdout.txt");
const ONSET_ANNOTATIONS: &str = include_str!("guitar_corpus_onsets.tsv");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Partition {
    Development,
    Holdout,
}

impl Partition {
    fn label(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Holdout => "sealed holdout",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TimedEvent {
    sample: usize,
    event: MidiEvent,
}

#[derive(Debug)]
struct PipelineRun {
    events: Vec<TimedEvent>,
    state_transitions: Vec<(usize, u8, u8, bool)>,
    processing_time: Duration,
    active_note: Option<u8>,
    note_state: u8,
    final_rms: f32,
}

#[derive(Clone, Debug)]
struct RecordingResult {
    name: String,
    string: usize,
    fret: usize,
    expected: u8,
    sample_rate: usize,
    onset_sample: usize,
    events: Vec<TimedEvent>,
    state_transitions: Vec<(usize, u8, u8, bool)>,
    note_ons: Vec<(usize, u8, u8)>,
    exact_first: bool,
    eventual_correct: bool,
    octave_error: bool,
    clean_release: bool,
    stuck_note: Option<u8>,
    final_note_state: u8,
    final_rms: f32,
    deterministic: bool,
    audio_duration: Duration,
    processing_time: Duration,
}

impl RecordingResult {
    fn first_correct_note_latency_ms(&self) -> Option<f64> {
        self.note_ons
            .iter()
            .find(|(_, _, note)| *note == self.expected)
            .map(|(sample, _, _)| {
                sample.saturating_sub(self.onset_sample) as f64 * 1_000.0 / self.sample_rate as f64
            })
    }
}

#[derive(Default)]
struct Summary<'a> {
    rows: Vec<&'a RecordingResult>,
}

impl<'a> Summary<'a> {
    fn exact(&self) -> usize {
        self.rows.iter().filter(|r| r.exact_first).count()
    }

    fn retriggers(&self) -> usize {
        self.rows.iter().filter(|r| r.note_ons.len() > 1).count()
    }

    fn clean_releases(&self) -> usize {
        self.rows
            .iter()
            .filter(|r| r.clean_release && r.stuck_note.is_none())
            .count()
    }

    fn deterministic(&self) -> usize {
        self.rows.iter().filter(|r| r.deterministic).count()
    }

    fn processing_time(&self) -> Duration {
        self.rows.iter().map(|r| r.processing_time).sum()
    }

    fn audio_duration(&self) -> Duration {
        self.rows.iter().map(|r| r.audio_duration).sum()
    }
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ui/static/samples/by_class")
}

fn holdout_names() -> HashSet<&'static str> {
    HOLDOUT
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

fn onset_annotations() -> HashMap<&'static str, usize> {
    let annotations: HashMap<_, _> = ONSET_ANNOTATIONS
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (name, sample) = line
                .split_once('\t')
                .unwrap_or_else(|| panic!("invalid onset annotation: {line}"));
            (name, sample.parse().unwrap())
        })
        .collect();
    assert_eq!(annotations.len(), 138, "every corpus file needs one onset");
    annotations
}

fn corpus_paths(partition: Partition) -> Vec<PathBuf> {
    let holdout = holdout_names();
    assert_eq!(
        holdout.len(),
        30,
        "holdout list must remain fixed at 30 files"
    );

    let mut all: Vec<_> = std::fs::read_dir(corpus_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("wav"))
        .collect();
    all.sort();
    assert_eq!(all.len(), 138, "expected the complete labeled corpus");

    let corpus_names: HashSet<_> = all
        .iter()
        .map(|path| path.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(
        holdout.iter().all(|name| corpus_names.contains(name)),
        "fixed holdout contains a file absent from the corpus"
    );
    let annotations = onset_annotations();
    assert_eq!(
        annotations.keys().copied().collect::<HashSet<_>>(),
        corpus_names,
        "onset annotations must match the frozen corpus exactly"
    );

    all.into_iter()
        .filter(|path| {
            let name = path.file_name().unwrap().to_str().unwrap();
            let is_holdout = holdout.contains(name);
            match partition {
                Partition::Development => !is_holdout,
                Partition::Holdout => is_holdout,
            }
        })
        .collect()
}

fn parse_label(path: &Path) -> (usize, usize) {
    let name = path.file_stem().unwrap().to_str().unwrap();
    let parts: Vec<_> = name.split('_').collect();
    assert_eq!(parts.len(), 4, "unexpected corpus filename: {name}");
    assert_eq!(parts[0], "string");
    assert_eq!(parts[2], "fret");
    (parts[1].parse().unwrap(), parts[3].parse().unwrap())
}

fn read_wav(path: &Path) -> (usize, Vec<f32>) {
    let mut reader = hound::WavReader::open(path).unwrap();
    let spec = reader.spec();
    assert_eq!(spec.channels, 1, "{} must be mono", path.display());
    assert_eq!(
        spec.bits_per_sample,
        16,
        "{} must be 16-bit",
        path.display()
    );
    assert_eq!(spec.sample_format, hound::SampleFormat::Int);
    let samples = reader
        .samples::<i16>()
        .map(|sample| sample.unwrap() as f32 / i16::MAX as f32)
        .collect();
    (spec.sample_rate as usize, samples)
}

fn run_pipeline(sample_rate: usize, samples: &[f32]) -> PipelineRun {
    // Canonical shipping behavior: expression, legato, slides, pressure,
    // brightness, per-string channels, and the frozen attack minimum stay at
    // their public defaults.
    let config = GuitarInputConfig {
        sample_rate,
        buffer_size: 1024,
        hop_size: 256,
        cooldown_samples: sample_rate / 10,
        ..GuitarInputConfig::default()
    };
    let mut pipeline = GuitarInput::new(config);
    let mut events = Vec::new();
    let mut state_transitions = Vec::new();
    let started = Instant::now();
    let mut processed = 0;

    for chunk in samples.chunks(CHUNK_SIZE) {
        let before = pipeline.note_state_name();
        processed += chunk.len();
        let chunk_events = pipeline.process_block(chunk);
        if pipeline.note_state_name() != before {
            state_transitions.push((
                processed,
                before,
                pipeline.note_state_name(),
                pipeline.last_debug_onset,
            ));
        }
        for event in chunk_events {
            events.push(TimedEvent {
                sample: processed,
                event,
            });
        }
    }

    let silence = vec![0.0; sample_rate * RELEASE_TAIL_SECONDS];
    for chunk in silence.chunks(CHUNK_SIZE) {
        let before = pipeline.note_state_name();
        processed += chunk.len();
        let chunk_events = pipeline.process_block(chunk);
        if pipeline.note_state_name() != before {
            state_transitions.push((
                processed,
                before,
                pipeline.note_state_name(),
                pipeline.last_debug_onset,
            ));
        }
        for event in chunk_events {
            events.push(TimedEvent {
                sample: processed,
                event,
            });
        }
    }

    PipelineRun {
        events,
        state_transitions,
        processing_time: started.elapsed(),
        active_note: pipeline.current_note().map(|note| note.midi_note),
        note_state: pipeline.note_state_name(),
        final_rms: pipeline.prev_rms(),
    }
}

fn clean_release(events: &[TimedEvent]) -> bool {
    let mut active: HashMap<(u8, u8), usize> = HashMap::new();
    let mut unmatched_off = false;
    for timed in events {
        match timed.event {
            MidiEvent::NoteOn { channel, note, .. } => {
                *active.entry((channel, note)).or_default() += 1;
            }
            MidiEvent::NoteOff { channel, note, .. } => match active.get_mut(&(channel, note)) {
                Some(count) if *count > 1 => *count -= 1,
                Some(_) => {
                    active.remove(&(channel, note));
                }
                None => unmatched_off = true,
            },
            _ => {}
        }
    }
    !unmatched_off && active.is_empty()
}

fn evaluate(partition: Partition) -> Vec<RecordingResult> {
    let annotations = onset_annotations();
    corpus_paths(partition)
        .into_iter()
        .map(|path| {
            let name = path.file_name().unwrap().to_str().unwrap().to_owned();
            let (string, fret) = parse_label(&path);
            let expected = STRING_BASE_MIDI[string] + fret as u8;
            let (sample_rate, samples) = read_wav(&path);
            let onset_sample = *annotations
                .get(name.as_str())
                .unwrap_or_else(|| panic!("missing onset annotation for {name}"));
            let run = run_pipeline(sample_rate, &samples);
            let repeat = run_pipeline(sample_rate, &samples);
            let note_ons: Vec<_> = run
                .events
                .iter()
                .filter_map(|timed| match timed.event {
                    MidiEvent::NoteOn { channel, note, .. } => Some((timed.sample, channel, note)),
                    _ => None,
                })
                .collect();
            let first = note_ons.first().map(|(_, _, note)| *note);

            RecordingResult {
                name,
                string,
                fret,
                expected,
                sample_rate,
                onset_sample,
                exact_first: first == Some(expected),
                eventual_correct: note_ons.iter().any(|(_, _, note)| *note == expected),
                octave_error: first
                    .is_some_and(|note| note != expected && note % 12 == expected % 12),
                clean_release: clean_release(&run.events),
                stuck_note: run.active_note,
                final_note_state: run.note_state,
                final_rms: run.final_rms,
                deterministic: run.events == repeat.events,
                audio_duration: Duration::from_secs_f64(
                    (samples.len() + sample_rate * RELEASE_TAIL_SECONDS) as f64
                        / sample_rate as f64,
                ),
                processing_time: run.processing_time,
                events: run.events,
                state_transitions: run.state_transitions,
                note_ons,
            }
        })
        .collect()
}

fn print_summary(label: &str, rows: &[&RecordingResult]) {
    let summary = Summary {
        rows: rows.to_vec(),
    };
    let n = rows.len();
    let eventual = rows.iter().filter(|r| r.eventual_correct).count();
    let octave = rows.iter().filter(|r| r.octave_error).count();
    let ratio = summary.processing_time().as_secs_f64() / summary.audio_duration().as_secs_f64();
    let mut latencies: Vec<_> = rows
        .iter()
        .filter_map(|recording| recording.first_correct_note_latency_ms())
        .collect();
    let latency_misses = n - latencies.len();
    latencies.sort_by(f64::total_cmp);
    let percentile = |p: f64| -> f64 {
        if latencies.is_empty() {
            return 0.0;
        }
        latencies[((latencies.len() - 1) as f64 * p).round() as usize]
    };

    println!(
        "{label}: exact first {}/{} ({:.1}%), eventual {}/{} ({:.1}%), retrigger files {} ({:.1}%), octave errors {}, clean releases {}/{}, deterministic {}/{}, speed {:.3}x realtime, first-correct onset latency p50 {:.1}ms p95 {:.1}ms, misses {}",
        summary.exact(), n, summary.exact() as f64 * 100.0 / n as f64,
        eventual, n, eventual as f64 * 100.0 / n as f64,
        summary.retriggers(), summary.retriggers() as f64 * 100.0 / n as f64,
        octave, summary.clean_releases(), n, summary.deterministic(), n, ratio,
        percentile(0.50), percentile(0.95), latency_misses,
    );
}

fn print_report(partition: Partition, results: &[RecordingResult]) {
    println!("\n=== Guitar corpus: {} ===", partition.label());
    let rows: Vec<_> = results.iter().collect();
    print_summary(partition.label(), &rows);

    for string in 0..6 {
        let rows: Vec<_> = results.iter().filter(|r| r.string == string).collect();
        print_summary(&format!("string {string}"), &rows);
    }

    let mut by_fret: BTreeMap<usize, Vec<&RecordingResult>> = BTreeMap::new();
    for result in results {
        by_fret.entry(result.fret).or_default().push(result);
    }
    for (fret, rows) in by_fret {
        let exact = rows.iter().filter(|r| r.exact_first).count();
        println!("fret {fret:>2}: exact first {exact}/{}", rows.len());
    }

    println!("\nRelease failures:");
    for result in results.iter().filter(|r| !r.clean_release) {
        let lifecycle: Vec<_> = result
            .events
            .iter()
            .filter_map(|timed| match timed.event {
                MidiEvent::NoteOn { channel, note, .. } => Some((
                    timed.sample * 1_000 / result.sample_rate,
                    "on",
                    channel,
                    note,
                )),
                MidiEvent::NoteOff { channel, note, .. } => Some((
                    timed.sample * 1_000 / result.sample_rate,
                    "off",
                    channel,
                    note,
                )),
                _ => None,
            })
            .collect();
        println!(
            "  {} {lifecycle:?}, active={:?} state={} rms={:.6}",
            result.name, result.stuck_note, result.final_note_state, result.final_rms
        );
    }

    println!("\nRetriggers:");
    for result in results.iter().filter(|r| r.note_ons.len() > 1) {
        println!(
            "  {} {:?}, transitions {:?}",
            result.name,
            result
                .note_ons
                .iter()
                .map(|(sample, _, note)| (sample * 1_000 / result.sample_rate, *note))
                .collect::<Vec<_>>(),
            result
                .state_transitions
                .iter()
                .map(|(sample, from, to, onset)| {
                    (sample * 1_000 / result.sample_rate, *from, *to, *onset)
                })
                .collect::<Vec<_>>()
        );
    }

    println!("\nLatency misses / over 120 ms:");
    for result in results.iter().filter(|recording| {
        recording
            .first_correct_note_latency_ms()
            .is_none_or(|latency| latency > 120.0)
    }) {
        println!(
            "  {} expected {} latency {:?}",
            result.name,
            result.expected,
            result.first_correct_note_latency_ms()
        );
    }

    println!("\nFirst-note failures:");
    for result in results.iter().filter(|r| !r.exact_first) {
        let detected = result.note_ons.first().map(|(_, _, note)| *note);
        println!(
            "  {} expected {} got {:?}, NoteOns {:?}",
            result.name,
            result.expected,
            detected,
            result
                .note_ons
                .iter()
                .map(|(sample, _, note)| (sample * 1_000 / result.sample_rate, *note))
                .collect::<Vec<_>>(),
        );
    }
}

fn assert_partition(partition: Partition, results: &[RecordingResult]) {
    let rows: Vec<_> = results.iter().collect();
    let summary = Summary { rows };
    let n = results.len();
    let exact_rate = summary.exact() as f64 / n as f64;
    let retrigger_rate = summary.retriggers() as f64 / n as f64;
    let mut latencies: Vec<_> = results
        .iter()
        .filter_map(RecordingResult::first_correct_note_latency_ms)
        .collect();
    latencies.sort_by(f64::total_cmp);
    assert!(
        !latencies.is_empty(),
        "{} has no correct NoteOn",
        partition.label()
    );
    let latency_p95 = latencies[((latencies.len() - 1) as f64 * 0.95).round() as usize];
    assert!(
        exact_rate >= 0.95,
        "{} exact first-note accuracy {:.1}% is below 95%",
        partition.label(),
        exact_rate * 100.0
    );
    assert!(
        retrigger_rate < 0.05,
        "{} false-retrigger rate {:.1}% is not below 5%",
        partition.label(),
        retrigger_rate * 100.0
    );
    assert_eq!(
        summary.clean_releases(),
        n,
        "{} has unmatched NoteOn/NoteOff events",
        partition.label()
    );
    assert_eq!(
        summary.deterministic(),
        n,
        "{} output is not deterministic",
        partition.label()
    );
    assert!(
        summary.processing_time() < summary.audio_duration(),
        "{} corpus processing is not faster than real time",
        partition.label()
    );
    assert!(
        latency_p95 <= 120.0,
        "{} first-correct NoteOn p95 {:.1}ms exceeds 120ms",
        partition.label(),
        latency_p95
    );
}

fn run_gate(partition: Partition) {
    let results = evaluate(partition);
    print_report(partition, &results);
    assert_partition(partition, &results);
}

#[test]
fn development_corpus_gate() {
    run_gate(Partition::Development);
}

#[test]
#[ignore = "sealed holdout; run explicitly only after freezing a complete candidate"]
fn sealed_holdout_corpus_gate() {
    run_gate(Partition::Holdout);
}

#[test]
#[ignore = "report-only evaluator; development runs in normal CI and holdout stays sealed"]
fn full_corpus_report() {
    for partition in [Partition::Development, Partition::Holdout] {
        print_report(partition, &evaluate(partition));
    }
}
