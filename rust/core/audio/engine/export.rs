use crate::core::audio::engine::driver::MidiNoteEvent;
use std::fs::File;

pub fn generate_midi_file_impl(
    midi_events: &Vec<MidiNoteEvent>,
    output_path: &String,
    bpm: Option<f32>,
    tpqn: Option<u16>,
) -> Result<(), String> {
    use midly::num::{u7, u15, u24};
    use midly::{
        Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind,
    };

    if midi_events.is_empty() {
        return Ok(());
    }

    let bpm = bpm.unwrap_or(120.0_f32);
    let tpqn: u16 = tpqn.unwrap_or(480u16);
    let header = Header {
        format: Format::SingleTrack,
        timing: Timing::Metrical(u15::from(tpqn)),
    };

    #[derive(Clone)]
    struct AbsEvent {
        tick: u64,
        kind: TrackEventKind<'static>,
    }

    let mut abs_events: Vec<AbsEvent> = Vec::new();
    let microsecs_per_quarter = (60_000_000.0 / bpm) as u32;
    abs_events.push(AbsEvent {
        tick: 0,
        kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::from(microsecs_per_quarter))),
    });

    for ev in midi_events {
        let start_secs = (ev.start_ms as f32) / 1000.0;
        let dur_secs = (ev.duration_ms as f32) / 1000.0;
        let start_ticks_f = start_secs * (bpm / 60.0) * (tpqn as f32);
        let dur_ticks_f = dur_secs * (bpm / 60.0) * (tpqn as f32);
        let start_tick = start_ticks_f.max(0.0).round() as u64;
        let off_tick = (start_ticks_f + dur_ticks_f).max(start_tick as f32).round() as u64;

        let key = u7::from(ev.key.min(127));
        let vel = u7::from(ev.vel.min(127));

        abs_events.push(AbsEvent {
            tick: start_tick,
            kind: TrackEventKind::Midi {
                channel: (ev.channel as u8).into(),
                message: MidiMessage::NoteOn { key, vel },
            },
        });

        abs_events.push(AbsEvent {
            tick: off_tick,
            kind: TrackEventKind::Midi {
                channel: (ev.channel as u8).into(),
                message: MidiMessage::NoteOff {
                    key,
                    vel: u7::from(0),
                },
            },
        });
    }

    let max_tick = abs_events.iter().map(|e| e.tick).max().unwrap_or(0);
    abs_events.push(AbsEvent {
        tick: max_tick + 0,
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });
    abs_events.sort_by_key(|e| e.tick);

    let mut track: Vec<TrackEvent> = Vec::new();
    let mut last_tick: u64 = 0;
    for e in abs_events {
        let delta = (e.tick - last_tick) as u32;
        track.push(TrackEvent {
            delta: delta.into(),
            kind: e.kind,
        });
        last_tick = e.tick;
    }

    let smf = Smf {
        header,
        tracks: vec![track],
    };

    if let Ok(mut file) = File::create(output_path) {
        if let Err(e) = smf.write_std(&mut file) {
            return Err(format!("Error writing MIDI file: {}", e));
        }
    } else {
        return Err(format!("Cannot create MIDI file at {}", output_path));
    }

    Ok(())
}

pub fn generate_wav_file_impl(
    buffer: &mut Vec<i16>,
    output_dir: &String,
    audio_format: Option<String>,
    sample_rate: Option<u32>,
) -> Result<(), String> {
    if buffer.len() % (crate::core::audio::engine::CHANNELS as usize) != 0 {
        buffer.push(0);
    }

    let sr = sample_rate.unwrap_or(crate::core::audio::engine::SAMPLE_RATE);
    let format_str = audio_format.unwrap_or_else(|| "Wav16".to_string());
    let fmt_low = format_str.to_lowercase();

    match fmt_low.as_str() {
        "wav16" => {
            let spec = hound::WavSpec {
                channels: crate::core::audio::engine::CHANNELS,
                sample_rate: sr,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };

            let mut writer = hound::WavWriter::create(output_dir, spec)
                .map_err(|e| format!("Error creating WAV file: {}", e))?;

            for sample in buffer.iter() {
                writer
                    .write_sample(*sample)
                    .map_err(|e| format!("Error writing sample: {:?}", e))?;
            }

            writer
                .finalize()
                .map_err(|e| format!("Error finalizing WAV: {:?}", e))?;
        }
        "wav24" | "wav32" => {
            let bits = if fmt_low.contains("24") { 24 } else { 32 };
            let spec = hound::WavSpec {
                channels: crate::core::audio::engine::CHANNELS,
                sample_rate: sr,
                bits_per_sample: bits,
                sample_format: hound::SampleFormat::Int,
            };

            let mut writer = hound::WavWriter::create(output_dir, spec)
                .map_err(|e| format!("Error creating WAV file: {}", e))?;

            for &s in buffer.iter() {
                let v32 = (s as i32) << (bits - 16);
                writer
                    .write_sample(v32)
                    .map_err(|e| format!("Error writing sample: {:?}", e))?;
            }

            writer
                .finalize()
                .map_err(|e| format!("Error finalizing WAV: {:?}", e))?;
        }
        _ => {
            return Err(format!("Unsupported audio format: {}", format_str));
        }
    }

    Ok(())
}
