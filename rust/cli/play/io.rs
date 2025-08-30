use hound;

pub fn wav_duration_seconds(path: &str) -> Option<f32> {
    if let Ok(reader) = hound::WavReader::open(path) {
        let spec = reader.spec();
        let len = reader.len();
        if spec.sample_rate == 0 {
            return None;
        }
        let channels = spec.channels.max(1) as u32;
        let frames = len / channels;
        let dur = (frames as f32) / (spec.sample_rate as f32);
        Some(dur)
    } else {
        None
    }
}
