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
