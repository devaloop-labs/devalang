//! Audio format encoders for export
//!
//! This module provides encoding functionality for various audio formats.
//! Currently supported:
//! - WAV (via hound) - 16/24/32-bit
//! - MP3 (via mp3lame-encoder) - 128/192/256/320 kbps
//!
//! Planned: OGG Vorbis, FLAC, Opus

use anyhow::{Result, anyhow};

/// Supported export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Wav,
    Mp3,
    Ogg,
    Flac,
    Opus,
}

impl AudioFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "wav" => Some(Self::Wav),
            "mp3" => Some(Self::Mp3),
            "ogg" | "vorbis" => Some(Self::Ogg),
            "flac" => Some(Self::Flac),
            "opus" => Some(Self::Opus),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
            Self::Ogg => "ogg",
            Self::Flac => "flac",
            Self::Opus => "opus",
        }
    }

    pub fn is_supported(&self) -> bool {
        matches!(self, Self::Wav | Self::Mp3)
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
            Self::Ogg => "ogg",
            Self::Flac => "flac",
            Self::Opus => "opus",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Wav => "audio/wav",
            Self::Mp3 => "audio/mpeg",
            Self::Ogg => "audio/ogg",
            Self::Flac => "audio/flac",
            Self::Opus => "audio/opus",
        }
    }
}

/// Encoder options for each format
#[derive(Debug, Clone)]
pub struct EncoderOptions {
    pub format: AudioFormat,
    pub sample_rate: u32,
    pub bit_depth: u8,     // For WAV/FLAC: 16, 24, 32
    pub bitrate_kbps: u32, // For MP3/OGG/Opus: 128, 192, 256, 320
    pub quality: f32,      // For OGG/Opus: 0.0-10.0 (quality scale)
}

impl Default for EncoderOptions {
    fn default() -> Self {
        Self {
            format: AudioFormat::Wav,
            sample_rate: 44100,
            bit_depth: 16,
            bitrate_kbps: 192,
            quality: 5.0,
        }
    }
}

impl EncoderOptions {
    pub fn wav(sample_rate: u32, bit_depth: u8) -> Self {
        Self {
            format: AudioFormat::Wav,
            sample_rate,
            bit_depth,
            ..Default::default()
        }
    }

    pub fn mp3(sample_rate: u32, bitrate_kbps: u32) -> Self {
        Self {
            format: AudioFormat::Mp3,
            sample_rate,
            bitrate_kbps,
            ..Default::default()
        }
    }

    pub fn ogg(sample_rate: u32, quality: f32) -> Self {
        Self {
            format: AudioFormat::Ogg,
            sample_rate,
            quality,
            ..Default::default()
        }
    }

    pub fn flac(sample_rate: u32, bit_depth: u8) -> Self {
        Self {
            format: AudioFormat::Flac,
            sample_rate,
            bit_depth,
            ..Default::default()
        }
    }
}

/// Encode PCM samples (f32, normalized -1.0 to 1.0) to the specified format
pub fn encode_audio(pcm_samples: &[f32], options: &EncoderOptions) -> Result<Vec<u8>> {
    match options.format {
        AudioFormat::Wav => encode_wav(pcm_samples, options),
        AudioFormat::Mp3 => {
            #[cfg(feature = "cli")]
            {
                return encode_mp3(pcm_samples, options);
            }
            #[cfg(not(feature = "cli"))]
            {
                return Err(anyhow!(
                    "MP3 export not available in this build (disabled for WASM)."
                ));
            }
        }
        AudioFormat::Ogg => encode_ogg(pcm_samples, options),
        AudioFormat::Flac => encode_flac(pcm_samples, options),
        AudioFormat::Opus => encode_opus(pcm_samples, options),
    }
}

/// Encode to WAV format (using hound)
#[cfg(any(feature = "cli", feature = "wasm"))]
fn encode_wav(pcm_samples: &[f32], options: &EncoderOptions) -> Result<Vec<u8>> {
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::io::Cursor;

    let spec = WavSpec {
        channels: 2,
        sample_rate: options.sample_rate,
        bits_per_sample: options.bit_depth as u16,
        sample_format: if options.bit_depth == 32 {
            SampleFormat::Float
        } else {
            SampleFormat::Int
        },
    };

    let mut cursor = Cursor::new(Vec::new());
    let mut writer = WavWriter::new(&mut cursor, spec)
        .map_err(|e| anyhow!("Failed to create WAV writer: {}", e))?;

    // Convert mono f32 to stereo with specified bit depth
    match options.bit_depth {
        16 => {
            for &sample in pcm_samples {
                let clamped = sample.clamp(-1.0, 1.0);
                let i16_sample = (clamped * 32767.0) as i16;
                writer
                    .write_sample(i16_sample)
                    .map_err(|e| anyhow!("Failed to write sample: {}", e))?;
                writer
                    .write_sample(i16_sample)
                    .map_err(|e| anyhow!("Failed to write sample: {}", e))?;
            }
        }
        24 => {
            for &sample in pcm_samples {
                let clamped = sample.clamp(-1.0, 1.0);
                let i24_sample = (clamped * 8388607.0) as i32;
                writer
                    .write_sample(i24_sample)
                    .map_err(|e| anyhow!("Failed to write sample: {}", e))?;
                writer
                    .write_sample(i24_sample)
                    .map_err(|e| anyhow!("Failed to write sample: {}", e))?;
            }
        }
        32 => {
            for &sample in pcm_samples {
                writer
                    .write_sample(sample)
                    .map_err(|e| anyhow!("Failed to write sample: {}", e))?;
                writer
                    .write_sample(sample)
                    .map_err(|e| anyhow!("Failed to write sample: {}", e))?;
            }
        }
        _ => {
            return Err(anyhow!(
                "Unsupported bit depth: {} (expected 16, 24, or 32)",
                options.bit_depth
            ));
        }
    }

    writer
        .finalize()
        .map_err(|e| anyhow!("Failed to finalize WAV: {}", e))?;

    Ok(cursor.into_inner())
}

/// Fallback stub when `hound` isn't enabled (e.g., plugin-only build without features)
#[cfg(not(any(feature = "cli", feature = "wasm")))]
fn encode_wav(_pcm_samples: &[f32], _options: &EncoderOptions) -> Result<Vec<u8>> {
    Err(anyhow!(
        "WAV export not available in this build: missing 'hound' dependency. Enable the 'cli' or 'wasm' feature to include WAV support."
    ))
}

/// Encode to MP3 format using LAME encoder
#[cfg(feature = "cli")]
fn encode_mp3(pcm_samples: &[f32], options: &EncoderOptions) -> Result<Vec<u8>> {
    use mp3lame_encoder::{Builder, FlushNoGap, InterleavedPcm};
    use std::mem::MaybeUninit;

    // Convert mono f32 samples to stereo i16 for LAME
    let mut stereo_samples: Vec<i16> = Vec::with_capacity(pcm_samples.len() * 2);
    for &sample in pcm_samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let i16_sample = (clamped * 32767.0) as i16;
        stereo_samples.push(i16_sample); // Left channel
        stereo_samples.push(i16_sample); // Right channel (duplicate for stereo)
    }

    // Create MP3 encoder with specified settings
    let mut builder = Builder::new().ok_or_else(|| anyhow!("Failed to create MP3 encoder"))?;

    builder
        .set_num_channels(2)
        .map_err(|_| anyhow!("Failed to set channels"))?;
    builder
        .set_sample_rate(options.sample_rate)
        .map_err(|_| anyhow!("Failed to set sample rate"))?;

    // Set bitrate - convert from kbps to the bitrate enum
    let bitrate = match options.bitrate_kbps {
        128 => mp3lame_encoder::Bitrate::Kbps128,
        192 => mp3lame_encoder::Bitrate::Kbps192,
        256 => mp3lame_encoder::Bitrate::Kbps256,
        320 => mp3lame_encoder::Bitrate::Kbps320,
        _ => mp3lame_encoder::Bitrate::Kbps192, // Default to 192 if not standard
    };
    builder
        .set_brate(bitrate)
        .map_err(|_| anyhow!("Failed to set bitrate"))?;

    builder
        .set_quality(mp3lame_encoder::Quality::Best)
        .map_err(|_| anyhow!("Failed to set quality"))?;

    let mut encoder = builder
        .build()
        .map_err(|_| anyhow!("Failed to build MP3 encoder"))?;

    // Allocate output buffer for MP3 data
    // MP3 compression typically reduces size by ~10x, but we allocate more to be safe
    let max_output_size = (stereo_samples.len() * 5 / 4) + 7200;
    let mut output_buffer: Vec<MaybeUninit<u8>> = vec![MaybeUninit::uninit(); max_output_size];

    // Encode the audio data
    let input = InterleavedPcm(&stereo_samples);
    let encoded_size = encoder
        .encode(input, &mut output_buffer)
        .map_err(|_| anyhow!("Failed to encode MP3"))?;

    // Convert the written portion to initialized bytes
    let mut mp3_buffer = Vec::with_capacity(encoded_size + 7200);
    unsafe {
        mp3_buffer.extend(
            output_buffer[..encoded_size]
                .iter()
                .map(|b| b.assume_init()),
        );
    }

    // Flush the encoder to get remaining data
    let mut flush_buffer: Vec<MaybeUninit<u8>> = vec![MaybeUninit::uninit(); 7200];
    let flushed_size = encoder
        .flush::<FlushNoGap>(&mut flush_buffer)
        .map_err(|_| anyhow!("Failed to flush MP3 encoder"))?;

    unsafe {
        mp3_buffer.extend(flush_buffer[..flushed_size].iter().map(|b| b.assume_init()));
    }

    Ok(mp3_buffer)
}

/// Encode to OGG Vorbis format
/// TODO: Implement using vorbis encoder
fn encode_ogg(_pcm_samples: &[f32], options: &EncoderOptions) -> Result<Vec<u8>> {
    Err(anyhow!(
        "OGG Vorbis export not yet implemented. \n\
        OGG encoding is planned for v2.1.\n\
        Workaround: Export to WAV and convert with ffmpeg:\n\
        - ffmpeg -i output.wav -c:a libvorbis -q:a {} output.ogg\n\
        \n\
        Supported formats: WAV (16/24/32-bit)\n\
        Coming soon: OGG Vorbis, FLAC, Opus",
        options.quality
    ))
}

/// Encode to FLAC format
/// TODO: Implement using FLAC encoder
fn encode_flac(_pcm_samples: &[f32], _options: &EncoderOptions) -> Result<Vec<u8>> {
    Err(anyhow!(
        "FLAC export not yet implemented. \n\
        FLAC encoding is planned for v2.1.\n\
        Workaround: Export to WAV and convert with ffmpeg:\n\
        - ffmpeg -i output.wav -c:a flac output.flac\n\
        \n\
        Supported formats: WAV (16/24/32-bit)\n\
        Coming soon: OGG Vorbis, FLAC, Opus"
    ))
}

/// Encode to Opus format
/// TODO: Implement using opus encoder
fn encode_opus(_pcm_samples: &[f32], options: &EncoderOptions) -> Result<Vec<u8>> {
    Err(anyhow!(
        "Opus export not yet implemented. \n\
        Opus encoding is planned for v2.1.\n\
        Workaround: Export to WAV and convert with ffmpeg:\n\
        - ffmpeg -i output.wav -c:a libopus -b:a {}k output.opus\n\
        \n\
        Supported formats: WAV (16/24/32-bit)\n\
        Coming soon: OGG Vorbis, FLAC, Opus",
        options.bitrate_kbps
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_parsing() {
        assert_eq!(AudioFormat::from_str("wav"), Some(AudioFormat::Wav));
        assert_eq!(AudioFormat::from_str("MP3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_str("ogg"), Some(AudioFormat::Ogg));
        assert_eq!(AudioFormat::from_str("vorbis"), Some(AudioFormat::Ogg));
        assert_eq!(AudioFormat::from_str("flac"), Some(AudioFormat::Flac));
        assert_eq!(AudioFormat::from_str("opus"), Some(AudioFormat::Opus));
        assert_eq!(AudioFormat::from_str("unknown"), None);
    }

    #[test]
    fn test_wav_encoding() {
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let options = EncoderOptions::wav(44100, 16);
        let result = encode_audio(&samples, &options);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 44); // WAV header + data
    }

    #[test]
    fn test_mp3_encoding() {
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0, 0.25, -0.75, 0.8];
        let options = EncoderOptions::mp3(44100, 192);
        let result = encode_audio(&samples, &options);
        assert!(result.is_ok(), "MP3 encoding should succeed");
        let bytes = result.unwrap();
        assert!(bytes.len() > 0, "MP3 output should not be empty");
        // MP3 files should have proper headers
        assert!(bytes.len() > 100, "MP3 file should have reasonable size");
    }
}
