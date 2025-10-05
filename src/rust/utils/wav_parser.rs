//! WAV file parsing utilities
//!
//! Parses WAV files directly in Rust without requiring Web Audio API.
//! Supports 8/16/24/32-bit PCM, mono/stereo, and automatic stereo→mono conversion.

/// Parse WAV file and return (channels, sample_rate, mono_i16_samples)
///
/// If stereo, automatically converts to mono by averaging channels.
/// Returns samples as i16 normalized to [-32768, 32767] range.
pub fn parse_wav_generic(data: &[u8]) -> Result<(u16, u32, Vec<i16>), String> {
    if data.len() < 44 {
        return Err("File too short for WAV header".into());
    }

    // Validate RIFF/WAVE header
    if &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Err("Invalid RIFF/WAVE header".into());
    }

    let mut pos = 12; // After RIFF header
    let mut channels = 1u16;
    let mut sample_rate = 44100u32;
    let mut bits = 16u16;
    let mut raw_bytes: Option<Vec<u8>> = None;

    // Parse chunks
    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap()) as usize;
        pos += 8;

        if pos + chunk_size > data.len() {
            break;
        }

        match chunk_id {
            b"fmt " => {
                if chunk_size < 16 {
                    return Err("fmt chunk too small".into());
                }

                let audio_format = u16::from_le_bytes(data[pos..pos + 2].try_into().unwrap());
                channels = u16::from_le_bytes(data[pos + 2..pos + 4].try_into().unwrap());
                sample_rate = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap());
                bits = u16::from_le_bytes(data[pos + 14..pos + 16].try_into().unwrap());

                if audio_format != 1 {
                    return Err("Only uncompressed PCM supported".into());
                }

                if !(bits == 8 || bits == 16 || bits == 24 || bits == 32) {
                    return Err(format!(
                        "Unsupported bit depth {} (expected 8/16/24/32)",
                        bits
                    ));
                }
            }
            b"data" => {
                raw_bytes = Some(data[pos..pos + chunk_size].to_vec());
            }
            _ => { /* Ignore other chunks */ }
        }

        pos += chunk_size;
    }

    let bytes = raw_bytes.ok_or("data chunk not found".to_string())?;

    // Convert to f32 based on bit depth
    let mut interleaved_f32: Vec<f32> = Vec::new();

    match bits {
        8 => {
            // 8-bit: unsigned, range [0, 255] → [-1.0, 1.0]
            for b in bytes.iter() {
                interleaved_f32.push((*b as f32 - 128.0) / 128.0);
            }
        }
        16 => {
            // 16-bit: signed, range [-32768, 32767] → [-1.0, 1.0]
            for ch in bytes.chunks_exact(2) {
                let v = i16::from_le_bytes([ch[0], ch[1]]);
                interleaved_f32.push(v as f32 / 32768.0);
            }
        }
        24 => {
            // 24-bit: signed, range [-8388608, 8388607] → [-1.0, 1.0]
            for ch in bytes.chunks_exact(3) {
                let assembled = (ch[0] as u32) | ((ch[1] as u32) << 8) | ((ch[2] as u32) << 16);

                // Sign extend from 24-bit to 32-bit
                let signed = if (assembled & 0x800000) != 0 {
                    (assembled | 0xFF000000) as i32
                } else {
                    assembled as i32
                };

                interleaved_f32.push(signed as f32 / 8388608.0);
            }
        }
        32 => {
            // 32-bit: signed, range [-2147483648, 2147483647] → [-1.0, 1.0]
            for ch in bytes.chunks_exact(4) {
                let v = i32::from_le_bytes([ch[0], ch[1], ch[2], ch[3]]);
                interleaved_f32.push(v as f32 / 2147483648.0);
            }
        }
        _ => return Err("Unexpected bit depth".into()),
    }

    let chn = channels as usize;

    // Convert stereo to mono if needed
    if chn > 1 {
        let frames = interleaved_f32.len() / chn;
        let mut mono_f32 = Vec::with_capacity(frames);

        for f in 0..frames {
            let mut acc = 0.0;
            for c in 0..chn {
                acc += interleaved_f32[f * chn + c];
            }
            mono_f32.push(acc / chn as f32);
        }

        // Convert to i16
        let mut out = Vec::with_capacity(mono_f32.len());
        for s in mono_f32 {
            out.push((s.clamp(-1.0, 1.0) * 32767.0) as i16);
        }

        Ok((1, sample_rate, out))
    } else {
        // Already mono, just convert to i16
        let mut out = Vec::with_capacity(interleaved_f32.len());
        for s in interleaved_f32 {
            out.push((s.clamp(-1.0, 1.0) * 32767.0) as i16);
        }

        Ok((1, sample_rate, out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_wav() {
        let data = vec![0u8; 10];
        assert!(parse_wav_generic(&data).is_err());
    }

    #[test]
    fn test_valid_wav_header() {
        // Minimal WAV header (44 bytes)
        let mut data = vec![0u8; 44];
        data[0..4].copy_from_slice(b"RIFF");
        data[8..12].copy_from_slice(b"WAVE");
        data[12..16].copy_from_slice(b"fmt ");
        data[16..20].copy_from_slice(&16u32.to_le_bytes()); // fmt size
        data[20..22].copy_from_slice(&1u16.to_le_bytes()); // PCM
        data[22..24].copy_from_slice(&1u16.to_le_bytes()); // mono
        data[24..28].copy_from_slice(&44100u32.to_le_bytes()); // sample rate
        data[34..36].copy_from_slice(&16u16.to_le_bytes()); // bit depth
        data[36..40].copy_from_slice(b"data");
        data[40..44].copy_from_slice(&0u32.to_le_bytes()); // data size

        let result = parse_wav_generic(&data);
        assert!(result.is_ok());
    }
}
