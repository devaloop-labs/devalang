// Audio Node Types (v0.2.0)
// Defines the core building blocks of the audio graph

use crate::language::syntax::ast::Value;
use std::collections::HashMap;

/// Type of audio node in the graph
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    /// Synthesizer source (generates audio)
    Synth,
    /// Sample playback source (loads and plays samples)
    Sample,
    /// Mix bus (collects routed audio before another destination)
    Bus,
    /// Group/container node (collects multiple nodes)
    Group,
    /// Master output node (final audio destination)
    Output,
}

impl NodeType {
    pub fn is_source(&self) -> bool {
        matches!(self, NodeType::Synth | NodeType::Sample)
    }

    pub fn is_container(&self) -> bool {
        matches!(self, NodeType::Bus | NodeType::Group | NodeType::Output)
    }
}

/// Parameters for a node effect
#[derive(Debug, Clone)]
pub struct NodeParam {
    pub name: String,
    pub default: f32,
    pub min: f32,
    pub max: f32,
    pub description: Option<String>,
}

/// Audio Node: source of audio or container of other nodes
#[derive(Debug, Clone)]
pub struct AudioNode {
    /// Unique identifier
    pub id: String,
    /// Human-readable name (optional)
    pub name: Option<String>,
    /// Type of node
    pub node_type: NodeType,
    /// Effect chain: list of (effect_name, parameters)
    pub effects: Vec<(String, Value)>,
    /// Node parameters (e.g., volume, pan)
    pub parameters: HashMap<String, f32>,
}

impl AudioNode {
    /// Create new synth node
    pub fn new_synth(id: impl Into<String>) -> Self {
        let id = id.into();
        let mut params = HashMap::new();
        params.insert("volume".to_string(), 1.0);
        params.insert("pan".to_string(), 0.0);

        Self {
            id,
            name: None,
            node_type: NodeType::Synth,
            effects: Vec::new(),
            parameters: params,
        }
    }

    /// Create new sample node
    pub fn new_sample(id: impl Into<String>) -> Self {
        let id = id.into();
        let mut params = HashMap::new();
        params.insert("volume".to_string(), 1.0);
        params.insert("pan".to_string(), 0.0);

        Self {
            id,
            name: None,
            node_type: NodeType::Sample,
            effects: Vec::new(),
            parameters: params,
        }
    }

    /// Create new group node
    pub fn new_group(id: impl Into<String>) -> Self {
        let id = id.into();
        let mut params = HashMap::new();
        params.insert("volume".to_string(), 1.0);
        params.insert("pan".to_string(), 0.0);

        Self {
            id,
            name: None,
            node_type: NodeType::Group,
            effects: Vec::new(),
            parameters: params,
        }
    }

    /// Create new bus node
    pub fn new_bus(id: impl Into<String>) -> Self {
        let id = id.into();
        let mut params = HashMap::new();
        params.insert("volume".to_string(), 1.0);
        params.insert("pan".to_string(), 0.0);

        Self {
            id,
            name: None,
            node_type: NodeType::Bus,
            effects: Vec::new(),
            parameters: params,
        }
    }

    /// Create master output node
    pub fn new_master() -> Self {
        let mut params = HashMap::new();
        params.insert("volume".to_string(), 1.0);

        Self {
            id: "master".to_string(),
            name: Some("Master Output".to_string()),
            node_type: NodeType::Output,
            effects: Vec::new(),
            parameters: params,
        }
    }

    /// Add effect to effect chain
    pub fn add_effect(&mut self, name: impl Into<String>, params: Value) {
        self.effects.push((name.into(), params));
    }

    /// Set node parameter
    pub fn set_parameter(&mut self, name: impl Into<String>, value: f32) {
        self.parameters.insert(name.into(), value);
    }

    /// Get node parameter
    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        self.parameters.get(name).copied()
    }

    /// Volume in dB (converted from linear)
    pub fn volume(&self) -> f32 {
        self.parameters.get("volume").copied().unwrap_or(1.0)
    }

    /// Pan (-1.0 to 1.0)
    pub fn pan(&self) -> f32 {
        self.parameters.get("pan").copied().unwrap_or(0.0)
    }

    /// Set volume (linear scale: 0.0 to 2.0+)
    pub fn set_volume(&mut self, vol: f32) {
        self.set_parameter("volume", vol);
    }

    /// Set pan (-1.0 to 1.0: left to right)
    pub fn set_pan(&mut self, pan: f32) {
        self.set_parameter("pan", pan.clamp(-1.0, 1.0));
    }

    /// Check if node is a source (generates audio)
    pub fn is_source(&self) -> bool {
        self.node_type.is_source()
    }

    /// Check if node is a container
    pub fn is_container(&self) -> bool {
        self.node_type.is_container()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_synth_node() {
        let node = AudioNode::new_synth("synth_1");
        assert_eq!(node.id, "synth_1");
        assert_eq!(node.node_type, NodeType::Synth);
        assert_eq!(node.volume(), 1.0);
        assert_eq!(node.pan(), 0.0);
        assert!(node.is_source());
    }

    #[test]
    fn test_new_group_node() {
        let node = AudioNode::new_group("drums");
        assert_eq!(node.id, "drums");
        assert_eq!(node.node_type, NodeType::Group);
        assert!(node.is_container());
    }

    #[test]
    fn test_set_volume() {
        let mut node = AudioNode::new_synth("syn");
        node.set_volume(0.5);
        assert_eq!(node.volume(), 0.5);
    }

    #[test]
    fn test_add_effect() {
        let mut node = AudioNode::new_synth("syn");
        node.add_effect("reverb", Value::Number(0.8));
        assert_eq!(node.effects.len(), 1);
        assert_eq!(node.effects[0].0, "reverb");
    }
}
