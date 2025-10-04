//! Data conversion utilities for WASM

use anyhow::{Result, anyhow};

/// Convert f32 PCM to i16 PCM
pub fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect()
}

/// Convert i16 PCM to f32 PCM
pub fn i16_to_f32(samples: &[i16]) -> Vec<f32> {
    samples
        .iter()
        .map(|&sample| sample as f32 / i16::MAX as f32)
        .collect()
}

/// Convert f32 buffer to WAV bytes (mono, 16-bit)
pub fn pcm_to_wav_bytes(pcm: &[f32], sample_rate: u32) -> Result<Vec<u8>> {
    pcm_to_wav_bytes_with_depth(pcm, sample_rate, 16)
}

/// Convert f32 buffer to WAV bytes with configurable bit depth
pub fn pcm_to_wav_bytes_with_depth(
    pcm: &[f32],
    sample_rate: u32,
    bit_depth: u8,
) -> Result<Vec<u8>> {
    use std::io::Cursor;

    if ![16, 24, 32].contains(&bit_depth) {
        return Err(anyhow!(
            "Unsupported bit depth: {}. Use 16, 24, or 32.",
            bit_depth
        ));
    }

    // Create in-memory WAV writer
    let mut cursor = Cursor::new(Vec::new());

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: bit_depth as u16,
        sample_format: if bit_depth == 32 {
            hound::SampleFormat::Float
        } else {
            hound::SampleFormat::Int
        },
    };

    let mut writer = hound::WavWriter::new(&mut cursor, spec)
        .map_err(|e| anyhow!("Failed to create WAV writer: {}", e))?;

    // Write samples based on bit depth
    match bit_depth {
        16 => {
            for &sample in pcm {
                let clamped = sample.clamp(-1.0, 1.0);
                let i16_sample = (clamped * 32767.0) as i16;
                writer
                    .write_sample(i16_sample)
                    .map_err(|e| anyhow!("Failed to write sample: {}", e))?;
            }
        }
        24 => {
            for &sample in pcm {
                let clamped = sample.clamp(-1.0, 1.0);
                let i24_sample = (clamped * 8388607.0) as i32; // 24-bit max
                writer
                    .write_sample(i24_sample)
                    .map_err(|e| anyhow!("Failed to write sample: {}", e))?;
            }
        }
        32 => {
            for &sample in pcm {
                writer
                    .write_sample(sample)
                    .map_err(|e| anyhow!("Failed to write sample: {}", e))?;
            }
        }
        _ => unreachable!(),
    }

    writer
        .finalize()
        .map_err(|e| anyhow!("Failed to finalize WAV: {}", e))?;

    Ok(cursor.into_inner())
}
