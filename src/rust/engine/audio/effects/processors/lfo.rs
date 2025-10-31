use crate::engine::audio::effects::processors::super_trait::EffectProcessor;
use crate::engine::audio::lfo::{LfoParams, LfoTarget, generate_lfo_value};

#[derive(Debug, Clone)]
pub struct LfoProcessor {
    pub params: LfoParams,
    pub bpm: f32,
    sample_offset: usize,
    // For pitch modulation (resampling)
    read_pos: f32,
    semitone_range: f32,

    // For cutoff modulation (simple one-pole filter)
    base_cutoff: f32,
    cutoff_range: f32,
    prev_l: f32,
    prev_r: f32,
}

impl LfoProcessor {
    pub fn new(
        params: LfoParams,
        bpm: f32,
        semitone_range: f32,
        base_cutoff: f32,
        cutoff_range: f32,
    ) -> Self {
        Self {
            params,
            bpm,
            sample_offset: 0,
            read_pos: 0.0,
            semitone_range,
            base_cutoff,
            cutoff_range,
            prev_l: 0.0,
            prev_r: 0.0,
        }
    }
}

impl Default for LfoProcessor {
    fn default() -> Self {
        Self::new(LfoParams::default(), 120.0, 2.0, 1000.0, 1000.0)
    }
}

/// Catmull-Rom cubic interpolation for four samples
fn cubic_catmull_rom(y0: f32, y1: f32, y2: f32, y3: f32, t: f32) -> f32 {
    // y0 = previous sample (base-1)
    // y1 = sample at base
    // y2 = sample at base+1
    // y3 = sample at base+2
    // t in [0,1]
    let t2 = t * t;
    let t3 = t2 * t;

    0.5 * ((2.0 * y1)
        + (-y0 + y2) * t
        + (2.0 * y0 - 5.0 * y1 + 4.0 * y2 - y3) * t2
        + (-y0 + 3.0 * y1 - 3.0 * y2 + y3) * t3)
}

impl EffectProcessor for LfoProcessor {
    fn process(&mut self, samples: &mut [f32], sample_rate: u32) {
        let sr = sample_rate as f32;

        // stereo interleaved
        let frames = samples.len() / 2;

        // Make a snapshot of the source for resampling when doing pitch modulation
        let src = samples.to_vec();

        for frame in 0..frames {
            let idx = frame * 2;
            let time_seconds = (self.sample_offset + frame) as f32 / sr;

            let lfo_val = generate_lfo_value(&self.params, time_seconds, self.bpm); // -depth..depth

            match self.params.target {
                LfoTarget::Volume => {
                    let mod_amp = 1.0 + lfo_val; // depth already applied
                    samples[idx] *= mod_amp;
                    if idx + 1 < samples.len() {
                        samples[idx + 1] *= mod_amp;
                    }
                }
                LfoTarget::Pan => {
                    let pan = lfo_val.clamp(-1.0, 1.0);
                    let left = (1.0 - pan) * 0.5;
                    let right = (1.0 + pan) * 0.5;
                    samples[idx] *= left;
                    if idx + 1 < samples.len() {
                        samples[idx + 1] *= right;
                    }
                }
                LfoTarget::Pitch => {
                    // semitone offset range controlled by semitone_range field
                    let semitone = lfo_val * self.semitone_range; // e.g., ±2 semitones
                    let speed = 2.0_f32.powf(semitone / 12.0);

                    // read from src at read_pos (in frames)
                    let pos = self.read_pos;
                    let base = pos.floor() as usize;
                    let frac = pos - base as f32;

                    let get_sample = |frame_idx: usize, ch: usize| -> f32 {
                        if frame_idx >= frames {
                            return 0.0;
                        }
                        src.get(frame_idx * 2 + ch).copied().unwrap_or(0.0)
                    };

                    // Left
                    // Cubic interpolation (Catmull-Rom) using four samples: y0,y1,y2,y3
                    let y_m1 = get_sample(base.wrapping_sub(1), 0);
                    let y0 = get_sample(base, 0);
                    let y1 = get_sample(base + 1, 0);
                    let y2 = get_sample(base + 2, 0);
                    let out_l = cubic_catmull_rom(y_m1, y0, y1, y2, frac);

                    // Right
                    let y_m1r = get_sample(base.wrapping_sub(1), 1);
                    let y0r = get_sample(base, 1);
                    let y1r = get_sample(base + 1, 1);
                    let y2r = get_sample(base + 2, 1);
                    let out_r = cubic_catmull_rom(y_m1r, y0r, y1r, y2r, frac);

                    samples[idx] = out_l;
                    if idx + 1 < samples.len() {
                        samples[idx + 1] = out_r;
                    }

                    self.read_pos += speed;
                }
                LfoTarget::FilterCutoff => {
                    // compute dynamic cutoff
                    let cutoff = (self.base_cutoff + lfo_val * self.cutoff_range).max(20.0);
                    // one-pole smoothing coefficient (approx)
                    let a = 1.0 - (-2.0 * std::f32::consts::PI * cutoff / sr).exp();

                    let in_l = samples[idx];
                    let in_r = if idx + 1 < samples.len() {
                        samples[idx + 1]
                    } else {
                        0.0
                    };

                    let out_l = a * in_l + (1.0 - a) * self.prev_l;
                    let out_r = a * in_r + (1.0 - a) * self.prev_r;

                    samples[idx] = out_l;
                    if idx + 1 < samples.len() {
                        samples[idx + 1] = out_r;
                    }

                    self.prev_l = out_l;
                    self.prev_r = out_r;
                }
            }
        }

        self.sample_offset = self.sample_offset.wrapping_add(frames);

        // Show example output first-frame after processing
        // Calculate max absolute difference between input snapshot and output to see if processing changed samples
        let mut max_diff = 0.0f32;
        let len = samples.len().min(src.len());
        for i in 0..len {
            let diff = (samples[i] - src[i]).abs();
            if diff > max_diff {
                max_diff = diff;
            }
        }
    }

    fn reset(&mut self) {
        self.sample_offset = 0;
    }

    fn name(&self) -> &str {
        "LFO"
    }
}
