#![cfg(feature = "cli")]

use crate::engine::audio::settings::{AudioBitDepth, AudioChannels};
use anyhow::{Context, Result};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::path::Path;

pub fn write_wav(
    path: &Path,
    pcm: &[f32],
    sample_rate: u32,
    requested_bit_depth: AudioBitDepth,
    channels: AudioChannels,
) -> Result<AudioBitDepth> {
    let (bit_depth, sample_format) = match requested_bit_depth {
        AudioBitDepth::Bit32 => (AudioBitDepth::Bit32, SampleFormat::Float),
        AudioBitDepth::Bit24 => (AudioBitDepth::Bit24, SampleFormat::Int),
        AudioBitDepth::Bit16 => (AudioBitDepth::Bit16, SampleFormat::Int),
        AudioBitDepth::Bit8 => (AudioBitDepth::Bit8, SampleFormat::Int),
    };

    let spec = WavSpec {
        channels: channels.count(),
        sample_rate,
        bits_per_sample: bit_depth.bits(),
        sample_format,
    };

    let mut writer = WavWriter::create(path, spec)
        .with_context(|| format!("failed to open WAV writer for {}", path.display()))?;

    match bit_depth {
        AudioBitDepth::Bit32 => {
            for sample in pcm {
                writer
                    .write_sample(sample.clamp(-1.0, 1.0))
                    .with_context(|| {
                        format!("unable to write audio sample to {}", path.display())
                    })?;
            }
        }
        AudioBitDepth::Bit24 => {
            for sample in pcm {
                let scaled = (sample.clamp(-1.0, 1.0) * 8_388_607.0).round() as i32;
                writer.write_sample(scaled).with_context(|| {
                    format!("unable to write audio sample to {}", path.display())
                })?;
            }
        }
        AudioBitDepth::Bit16 => {
            for sample in pcm {
                let scaled = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
                writer.write_sample(scaled).with_context(|| {
                    format!("unable to write audio sample to {}", path.display())
                })?;
            }
        }
        AudioBitDepth::Bit8 => {
            for sample in pcm {
                let scaled = (sample.clamp(-1.0, 1.0) * i8::MAX as f32).round() as i8;
                writer.write_sample(scaled).with_context(|| {
                    format!("unable to write audio sample to {}", path.display())
                })?;
            }
        }
    }

    writer
        .finalize()
        .with_context(|| format!("failed to finalize WAV file writer for {}", path.display()))?;

    Ok(bit_depth)
}
