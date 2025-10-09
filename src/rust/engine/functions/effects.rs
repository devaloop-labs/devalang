/// Additional arrow call functions: velocity, duration, pan, detune, spread, gain, attack, release, delay, reverb, drive
use super::{FunctionContext, FunctionExecutor};
use crate::language::syntax::ast::nodes::Value;
use anyhow::{Result, anyhow};

/// Velocity function: modifies note velocity (0-127)
pub struct VelocityFunction;

impl FunctionExecutor for VelocityFunction {
    fn name(&self) -> &str {
        "velocity"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!(
                "velocity() requires 1 argument (velocity value 0-127)"
            ));
        }

        let velocity = match &args[0] {
            Value::Number(v) => *v,
            _ => return Err(anyhow!("velocity() argument must be a number")),
        };

        context.set("velocity", Value::Number(velocity));
        Ok(())
    }
}

/// Duration function: modifies note duration
pub struct DurationFunction;

impl FunctionExecutor for DurationFunction {
    fn name(&self) -> &str {
        "duration"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!("duration() requires 1 argument (duration in ms)"));
        }

        let duration_ms = match &args[0] {
            Value::Number(d) => *d,
            _ => return Err(anyhow!("duration() argument must be a number")),
        };

        context.set("duration", Value::Number(duration_ms));
        context.duration = duration_ms / 1000.0; // Convert to seconds
        Ok(())
    }
}

/// Pan function: stereo positioning (-1.0 = left, 0.0 = center, 1.0 = right)
pub struct PanFunction;

impl FunctionExecutor for PanFunction {
    fn name(&self) -> &str {
        "pan"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!("pan() requires 1 argument (pan value -1.0 to 1.0)"));
        }

        let pan = match &args[0] {
            Value::Number(p) => *p,
            _ => return Err(anyhow!("pan() argument must be a number")),
        };

        context.set("pan", Value::Number(pan));
        Ok(())
    }
}

/// Detune function: pitch adjustment in cents
pub struct DetuneFunction;

impl FunctionExecutor for DetuneFunction {
    fn name(&self) -> &str {
        "detune"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!("detune() requires 1 argument (cents)"));
        }

        let detune = match &args[0] {
            Value::Number(d) => *d,
            _ => return Err(anyhow!("detune() argument must be a number")),
        };

        context.set("detune", Value::Number(detune));
        Ok(())
    }
}

/// Spread function: stereo spread for chords (0.0 = mono, 1.0 = full stereo)
pub struct SpreadFunction;

impl FunctionExecutor for SpreadFunction {
    fn name(&self) -> &str {
        "spread"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!(
                "spread() requires 1 argument (spread value 0.0 to 1.0)"
            ));
        }

        let spread = match &args[0] {
            Value::Number(s) => *s,
            _ => return Err(anyhow!("spread() argument must be a number")),
        };

        context.set("spread", Value::Number(spread));
        Ok(())
    }
}

/// Gain function: volume multiplier
pub struct GainFunction;

impl FunctionExecutor for GainFunction {
    fn name(&self) -> &str {
        "gain"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!("gain() requires 1 argument (gain multiplier)"));
        }

        let gain = match &args[0] {
            Value::Number(g) => *g,
            _ => return Err(anyhow!("gain() argument must be a number")),
        };

        context.set("gain", Value::Number(gain));
        Ok(())
    }
}

/// Attack function: envelope attack time override in ms
pub struct AttackFunction;

impl FunctionExecutor for AttackFunction {
    fn name(&self) -> &str {
        "attack"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!("attack() requires 1 argument (attack time in ms)"));
        }

        let attack = match &args[0] {
            Value::Number(a) => *a,
            _ => return Err(anyhow!("attack() argument must be a number")),
        };

        context.set("attack", Value::Number(attack));
        Ok(())
    }
}

/// Release function: envelope release time override in ms
pub struct ReleaseFunction;

impl FunctionExecutor for ReleaseFunction {
    fn name(&self) -> &str {
        "release"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!(
                "release() requires 1 argument (release time in ms)"
            ));
        }

        let release = match &args[0] {
            Value::Number(r) => *r,
            _ => return Err(anyhow!("release() argument must be a number")),
        };

        context.set("release", Value::Number(release));
        Ok(())
    }
}

/// Delay function: echo effect with time in ms (default feedback: 0.3, mix: 0.5)
/// Usage: -> delay(400) or -> delay(400, 0.5) or -> delay(400, 0.5, 0.7)
pub struct DelayFunction;

impl FunctionExecutor for DelayFunction {
    fn name(&self) -> &str {
        "delay"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!(
                "delay() requires at least 1 argument (delay time in ms)"
            ));
        }
        // Support both positional args: delay(time, feedback?, mix?)
        // and map-style: delay({ time: 300, feedback: 0.3, mix: 0.5 })
        let mut time_val: Option<f32> = None;
        let mut feedback: f32 = 0.3;
        let mut mix: f32 = 0.5;

        if let Some(Value::Map(params)) = args.first() {
            if let Some(Value::Number(t)) = params.get("time") {
                time_val = Some(*t);
            }
            if let Some(Value::Number(f)) = params.get("feedback") {
                feedback = *f;
            }
            if let Some(Value::Number(m)) = params.get("mix") {
                mix = *m;
            }
        } else {
            // Positional
            // Time in ms (required)
            time_val = match &args[0] {
                Value::Number(t) => Some(*t),
                _ => None,
            };

            if args.len() > 1 {
                if let Value::Number(f) = &args[1] {
                    feedback = *f;
                } else {
                    return Err(anyhow!(
                        "delay() second argument must be a number (feedback)"
                    ));
                }
            }

            if args.len() > 2 {
                if let Value::Number(m) = &args[2] {
                    mix = *m;
                } else {
                    return Err(anyhow!("delay() third argument must be a number (mix)"));
                }
            }
        }

        let time = if let Some(t) = time_val {
            t
        } else {
            return Err(anyhow!(
                "delay() requires a time parameter either as first numeric arg or as {{ time: ... }}"
            ));
        };

        context.set("delay_time", Value::Number(time));
        context.set("delay_feedback", Value::Number(feedback));
        context.set("delay_mix", Value::Number(mix));

        Ok(())
    }
}

/// Reverb function: reverb effect with amount (0.0-1.0, default: 0.5)
/// Usage: -> reverb(0.3) for 30% reverb
pub struct ReverbFunction;

impl FunctionExecutor for ReverbFunction {
    fn name(&self) -> &str {
        "reverb"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        // Accept either reverb(0.3) or reverb({ size: 0.9 })
        let mut amount = 0.5; // default

        if let Some(Value::Map(params)) = args.first() {
            if let Some(Value::Number(s)) = params.get("size") {
                amount = *s;
            } else if let Some(Value::Number(a)) = params.get("amount") {
                amount = *a;
            }
        } else if let Some(Value::Number(a)) = args.get(0) {
            amount = *a;
        } else {
            return Err(anyhow!(
                "reverb() requires either a numeric argument or a parameter map {{ size: ... }}"
            ));
        }

        context.set("reverb_amount", Value::Number(amount));
        context.set("reverb_size", Value::Number(amount)); // Keep compatibility

        Ok(())
    }
}

/// Drive function: tube-style saturation (amount: 0.0-1.0, color: 0.0-1.0 default 0.5)
/// Usage: -> drive(0.6) or -> drive(0.6, 0.7)
pub struct DriveFunction;

impl FunctionExecutor for DriveFunction {
    fn name(&self) -> &str {
        "drive"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!(
                "drive() requires at least 1 argument (drive amount 0.0-1.0)"
            ));
        }
        // Support both positional: drive(amount, color?) and map: drive({ gain: 2.0, color: 0.5 })
        let mut amount = 0.5;
        let mut color = 0.5;

        if let Some(Value::Map(params)) = args.first() {
            if let Some(Value::Number(g)) = params.get("gain") {
                amount = *g;
            } else if let Some(Value::Number(a)) = params.get("amount") {
                amount = *a;
            }

            if let Some(Value::Number(c)) = params.get("color") {
                color = *c;
            }
        } else {
            // Positional
            amount = match &args[0] {
                Value::Number(a) => *a,
                _ => return Err(anyhow!("drive() first argument must be a number (amount)")),
            };

            if args.len() > 1 {
                color = match &args[1] {
                    Value::Number(c) => *c,
                    _ => return Err(anyhow!("drive() second argument must be a number (color)")),
                };
            }
        }

        context.set("drive_amount", Value::Number(amount));
        context.set("drive_amp", Value::Number(amount)); // Keep compatibility
        context.set("drive_color", Value::Number(color));

        Ok(())
    }
}

/// Chorus function: multiple detuned voices for richness
pub struct ChorusFunction;

impl FunctionExecutor for ChorusFunction {
    fn name(&self) -> &str {
        "chorus"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        // Default params
        let mut depth = 0.5; // Amount of pitch modulation
        let mut rate = 1.5; // LFO rate in Hz
        let mut mix = 0.5; // Dry/wet mix

        // Parse params from map { depth: 0.5, rate: 1.5, mix: 0.5 }
        if let Some(Value::Map(params)) = args.first() {
            if let Some(Value::Number(d)) = params.get("depth") {
                depth = *d;
            }
            if let Some(Value::Number(r)) = params.get("rate") {
                rate = *r;
            }
            if let Some(Value::Number(m)) = params.get("mix") {
                mix = *m;
            }
        }

        context.set("chorus_depth", Value::Number(depth));
        context.set("chorus_rate", Value::Number(rate));
        context.set("chorus_mix", Value::Number(mix));

        Ok(())
    }
}

/// Flanger function: sweeping comb filter effect
pub struct FlangerFunction;

impl FunctionExecutor for FlangerFunction {
    fn name(&self) -> &str {
        "flanger"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        // Default params
        let mut depth = 0.7; // Delay modulation depth
        let mut rate = 0.5; // LFO rate in Hz
        let mut feedback = 0.5; // Feedback amount
        let mut mix = 0.5; // Dry/wet mix

        // Parse params from map
        if let Some(Value::Map(params)) = args.first() {
            if let Some(Value::Number(d)) = params.get("depth") {
                depth = *d;
            }
            if let Some(Value::Number(r)) = params.get("rate") {
                rate = *r;
            }
            if let Some(Value::Number(f)) = params.get("feedback") {
                feedback = *f;
            }
            if let Some(Value::Number(m)) = params.get("mix") {
                mix = *m;
            }
        }

        context.set("flanger_depth", Value::Number(depth));
        context.set("flanger_rate", Value::Number(rate));
        context.set("flanger_feedback", Value::Number(feedback));
        context.set("flanger_mix", Value::Number(mix));

        Ok(())
    }
}

/// Phaser function: phase shifting effect
pub struct PhaserFunction;

impl FunctionExecutor for PhaserFunction {
    fn name(&self) -> &str {
        "phaser"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        // Default params
        let mut stages = 4.0; // Number of allpass stages (2-12)
        let mut rate = 0.5; // LFO rate in Hz
        let mut depth = 0.7; // Modulation depth
        let mut feedback = 0.5; // Feedback amount
        let mut mix = 0.5; // Dry/wet mix

        // Parse params from map
        if let Some(Value::Map(params)) = args.first() {
            if let Some(Value::Number(s)) = params.get("stages") {
                stages = *s;
            }
            if let Some(Value::Number(r)) = params.get("rate") {
                rate = *r;
            }
            if let Some(Value::Number(d)) = params.get("depth") {
                depth = *d;
            }
            if let Some(Value::Number(f)) = params.get("feedback") {
                feedback = *f;
            }
            if let Some(Value::Number(m)) = params.get("mix") {
                mix = *m;
            }
        }

        context.set("phaser_stages", Value::Number(stages));
        context.set("phaser_rate", Value::Number(rate));
        context.set("phaser_depth", Value::Number(depth));
        context.set("phaser_feedback", Value::Number(feedback));
        context.set("phaser_mix", Value::Number(mix));

        Ok(())
    }
}

/// Compressor function: dynamic range compression
pub struct CompressorFunction;

impl FunctionExecutor for CompressorFunction {
    fn name(&self) -> &str {
        "compressor"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        // Default params
        let mut threshold = -20.0; // Threshold in dB
        let mut ratio = 4.0; // Compression ratio (1:1 to 20:1)
        let mut attack = 5.0; // Attack time in ms
        let mut release = 50.0; // Release time in ms
        let mut makeup = 0.0; // Makeup gain in dB

        // Parse params from map
        if let Some(Value::Map(params)) = args.first() {
            if let Some(Value::Number(t)) = params.get("threshold") {
                threshold = *t;
            }
            if let Some(Value::Number(r)) = params.get("ratio") {
                ratio = *r;
            }
            if let Some(Value::Number(a)) = params.get("attack") {
                attack = *a;
            }
            if let Some(Value::Number(r)) = params.get("release") {
                release = *r;
            }
            if let Some(Value::Number(m)) = params.get("makeup") {
                makeup = *m;
            }
        }

        context.set("compressor_threshold", Value::Number(threshold));
        context.set("compressor_ratio", Value::Number(ratio));
        context.set("compressor_attack", Value::Number(attack));
        context.set("compressor_release", Value::Number(release));
        context.set("compressor_makeup", Value::Number(makeup));

        Ok(())
    }
}

/// Distortion function: enhanced distortion/saturation
pub struct DistortionFunction;

impl FunctionExecutor for DistortionFunction {
    fn name(&self) -> &str {
        "distortion"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        // Default params
        let mut amount = 0.5; // Distortion amount 0-1
        let mut tone = 0.5; // Tone control (brightness)
        let mut mix = 1.0; // Dry/wet mix

        // Parse params from map
        if let Some(Value::Map(params)) = args.first() {
            if let Some(Value::Number(a)) = params.get("amount") {
                amount = *a;
            }
            if let Some(Value::Number(t)) = params.get("tone") {
                tone = *t;
            }
            if let Some(Value::Number(m)) = params.get("mix") {
                mix = *m;
            }
        }

        context.set("distortion_amount", Value::Number(amount));
        context.set("distortion_tone", Value::Number(tone));
        context.set("distortion_mix", Value::Number(mix));

        Ok(())
    }
}
