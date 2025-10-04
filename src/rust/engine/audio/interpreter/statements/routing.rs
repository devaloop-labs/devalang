/// Audio routing statement handler
use anyhow::Result;
use std::collections::HashMap;

/// Audio routing configuration
#[derive(Debug, Clone)]
pub struct AudioRoute {
    pub source: String,
    pub destination: String,
    pub channels: Vec<usize>,
    pub mix_level: f32,
}

impl Default for AudioRoute {
    fn default() -> Self {
        Self {
            source: String::new(),
            destination: "master".to_string(),
            channels: vec![0, 1], // Stereo by default
            mix_level: 1.0,
        }
    }
}

/// Execute audio routing statement
///
/// Routes audio signals from source to destination with optional processing.
/// Supports multi-channel routing, level control, and send/return configurations.
///
/// # Arguments
/// * `route` - Routing configuration
/// * `routing_table` - Global routing table to update
///
/// # Examples
/// ```ignore
/// // Route synth to reverb
/// route synth1 -> reverb
///
/// // Route with level control
/// route drums -> master @ 0.8
///
/// // Multi-channel routing
/// route synth -> [reverb, delay] @ [0.5, 0.3]
/// ```
pub fn execute_routing(
    route: &AudioRoute,
    routing_table: &mut HashMap<String, Vec<AudioRoute>>,
) -> Result<()> {
    // Validate routing configuration
    if route.source.is_empty() {
        anyhow::bail!("Routing source cannot be empty");
    }

    if route.destination.is_empty() {
        anyhow::bail!("Routing destination cannot be empty");
    }

    if route.mix_level < 0.0 || route.mix_level > 2.0 {
        anyhow::bail!(
            "Mix level must be between 0.0 and 2.0, got: {}",
            route.mix_level
        );
    }

    if route.channels.is_empty() {
        anyhow::bail!("Routing must specify at least one channel");
    }

    // Add route to routing table
    routing_table
        .entry(route.source.clone())
        .or_insert_with(Vec::new)
        .push(route.clone());

    // Log routing setup
    #[cfg(feature = "cli")]
    {
        eprintln!(
            "Route: {} -> {} ({} ch @ {:.2})",
            route.source,
            route.destination,
            route.channels.len(),
            route.mix_level
        );
    }

    Ok(())
}

/// Create a default routing from source to master
pub fn create_default_route(source: String) -> AudioRoute {
    AudioRoute {
        source,
        destination: "master".to_string(),
        channels: vec![0, 1],
        mix_level: 1.0,
    }
}

/// Parse routing expression from statement
pub fn parse_routing_expr(expr: &str) -> Result<AudioRoute> {
    // Simple parser for routing expressions
    // Format: "source -> destination @ level"

    let parts: Vec<&str> = expr.split("->").collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid routing expression: {}", expr);
    }

    let source = parts[0].trim().to_string();
    let dest_and_level: Vec<&str> = parts[1].split('@').collect();

    let destination = dest_and_level[0].trim().to_string();
    let mix_level = if dest_and_level.len() > 1 {
        dest_and_level[1]
            .trim()
            .parse::<f32>()
            .map_err(|_| anyhow::anyhow!("Invalid mix level: {}", dest_and_level[1]))?
    } else {
        1.0
    };

    Ok(AudioRoute {
        source,
        destination,
        channels: vec![0, 1],
        mix_level,
    })
}
