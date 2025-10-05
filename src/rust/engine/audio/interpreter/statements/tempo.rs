/// Tempo statement handler
use anyhow::Result;

/// Execute tempo change statement
///
/// Updates the BPM (beats per minute) for the audio interpreter.
/// This affects timing calculations for subsequent notes and patterns.
///
/// # Arguments
/// * `bpm` - New tempo in beats per minute (typically 60-200)
///
/// # Examples
/// ```ignore
/// execute_tempo(120.0)?; // Set tempo to 120 BPM
/// execute_tempo(140.0)?; // Change to 140 BPM
/// ```
pub fn execute_tempo(bpm: f32) -> Result<()> {
    // Validate BPM range
    if bpm <= 0.0 {
        anyhow::bail!("Tempo must be positive, got: {}", bpm);
    }

    if bpm < 20.0 || bpm > 300.0 {
        eprintln!(
            "Warning: Unusual tempo value: {} BPM (typical range: 60-200)",
            bpm
        );
    }

    // Tempo is actually handled at the interpreter level
    // This function validates and acknowledges the tempo change
    Ok(())
}
