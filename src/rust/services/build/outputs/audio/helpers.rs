#![cfg(feature = "cli")]

pub fn calculate_rms(pcm: &[f32]) -> f32 {
    if pcm.is_empty() {
        return 0.0;
    }
    let sum_sqr: f64 = pcm.iter().map(|s| (*s as f64) * (*s as f64)).sum();
    let mean = sum_sqr / (pcm.len() as f64);
    (mean.sqrt() as f32).abs()
}
