/// Sleep/Wait statement handler
use anyhow::Result;

/// Execute sleep/wait statement
///
/// Introduces a pause or delay in the audio timeline.
/// This advances the cursor time without generating audio events.
///
/// # Arguments
/// * `duration_ms` - Duration to wait in milliseconds
///
/// # Examples
/// ```ignore
/// execute_sleep(1000.0)?; // Wait 1 second
/// execute_sleep(500.0)?;  // Wait 0.5 seconds
/// ```
pub fn execute_sleep(duration_ms: f32) -> Result<()> {
    // Validate duration
    if duration_ms < 0.0 {
        anyhow::bail!("Sleep duration cannot be negative: {}", duration_ms);
    }

    if duration_ms > 60000.0 {
        eprintln!(
            "Warning: Very long sleep duration: {} ms (>1 minute)",
            duration_ms
        );
    }

    // Sleep is handled at interpreter level by advancing cursor_time
    // This function validates the duration value
    Ok(())
}
