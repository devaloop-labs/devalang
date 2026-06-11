// Audio Graph (v0.2.0)
// Explicit representation of audio routing and node connections

use crate::engine::audio::nodes::AudioNode;
use crate::language::syntax::ast::Value;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Sidechain/ducking mode
#[derive(Debug, Clone)]
pub struct SidechainParam {
    pub mode: SidechainMode,
    pub amount: f32,
}

/// Types of sidechain ducking
#[derive(Debug, Clone, PartialEq)]
pub enum SidechainMode {
    /// Amplitude-based ducking (simple volume reduction)
    Amplitude,
    /// Compressor-based ducking (more musical)
    Compressor,
}

/// Type of connection represented by a route.
#[derive(Debug, Clone, PartialEq)]
pub enum AudioRouteKind {
    /// Standard mix route.
    Mix,
    /// Duck destination based on source activity.
    Duck,
    /// Sidechain source based on destination activity.
    Sidechain,
}

/// Audio route: explicit connection between nodes
#[derive(Debug, Clone)]
pub struct AudioRoute {
    /// Source node ID
    pub from: String,
    /// Destination node ID
    pub to: String,
    /// Volume scaling (0.0 to 1.0+)
    pub volume: f32,
    /// Pan (-1.0 to 1.0: left to right)
    pub pan: f32,
    /// Route behavior
    pub kind: AudioRouteKind,
    /// Optional routing/effect parameters from the DSL
    pub effect_params: Option<Value>,
    /// Optional sidechain/ducking
    pub sidechain: Option<SidechainParam>,
}

impl AudioRoute {
    /// Create new route from source to destination
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            volume: 1.0,
            pan: 0.0,
            kind: AudioRouteKind::Mix,
            effect_params: None,
            sidechain: None,
        }
    }

    /// Create a ducking route.
    pub fn duck(from: impl Into<String>, to: impl Into<String>, params: Value) -> Self {
        Self {
            kind: AudioRouteKind::Duck,
            effect_params: Some(params),
            ..Self::new(from, to)
        }
    }

    /// Create a sidechain route.
    pub fn sidechain(from: impl Into<String>, to: impl Into<String>, params: Value) -> Self {
        Self {
            kind: AudioRouteKind::Sidechain,
            effect_params: Some(params),
            ..Self::new(from, to)
        }
    }

    /// Set route volume
    pub fn with_volume(mut self, vol: f32) -> Self {
        self.volume = vol;
        self
    }

    /// Set route pan
    pub fn with_pan(mut self, pan: f32) -> Self {
        self.pan = pan.clamp(-1.0, 1.0);
        self
    }

    /// Add sidechain ducking
    pub fn with_sidechain(mut self, mode: SidechainMode, amount: f32) -> Self {
        self.sidechain = Some(SidechainParam { mode, amount });
        self
    }
}

/// Audio Graph: collection of nodes and routes
#[derive(Debug, Clone)]
pub struct AudioGraph {
    /// All nodes by ID
    pub nodes: HashMap<String, AudioNode>,
    /// All routes (connections)
    pub routes: Vec<AudioRoute>,
    /// Master output node ID (always present)
    pub master: String,
}

pub fn canonicalize_node_id(name: &str) -> String {
    if name == "$master" || name == "master" {
        "master".to_string()
    } else {
        name.to_string()
    }
}

impl AudioGraph {
    /// Create new empty graph with master output
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        let master_node = AudioNode::new_master();
        nodes.insert("master".to_string(), master_node);

        Self {
            nodes,
            routes: Vec::new(),
            master: "master".to_string(),
        }
    }

    /// Add node to graph
    pub fn add_node(&mut self, node: AudioNode) -> Result<()> {
        if canonicalize_node_id(&node.id) == self.master {
            return Err(anyhow!("Cannot add node with reserved ID 'master'"));
        }
        if self.nodes.contains_key(&node.id) {
            return Err(anyhow!("Node with ID '{}' already exists", node.id));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    /// Add route to graph (validates source and destination exist)
    pub fn add_route(&mut self, route: AudioRoute) -> Result<()> {
        if !self.nodes.contains_key(&route.from) {
            return Err(anyhow!("Source node '{}' does not exist", route.from));
        }
        if !self.nodes.contains_key(&route.to) {
            return Err(anyhow!("Destination node '{}' does not exist", route.to));
        }
        self.routes.push(route);
        Ok(())
    }

    /// Add or update a bus node. This is used for routing declarations, where
    /// repeated declarations may add effects to an existing bus.
    pub fn upsert_bus(
        &mut self,
        id: impl Into<String>,
        alias: Option<String>,
        effects: Option<Value>,
    ) -> Result<()> {
        let id = canonicalize_node_id(&id.into());
        if id == self.master {
            if let Some(effects) = effects {
                if let Some(master) = self.nodes.get_mut(&self.master) {
                    master.effects = normalize_effect_value(effects)
                        .into_iter()
                        .map(effect_to_node_effect)
                        .collect();
                }
            }
            return Ok(());
        }

        if !self.nodes.contains_key(&id) {
            let mut node = AudioNode::new_bus(&id);
            node.name = alias.clone();
            self.nodes.insert(id.clone(), node);
        }

        if let Some(node) = self.nodes.get_mut(&id) {
            if alias.is_some() {
                node.name = alias;
            }
            if let Some(effects) = effects {
                node.effects = normalize_effect_value(effects)
                    .into_iter()
                    .map(effect_to_node_effect)
                    .collect();
            }
        }

        Ok(())
    }

    /// Build the canonical graph from the interpreter routing setup.
    pub fn from_routing_setup(
        routing_setup: &crate::engine::audio::interpreter::driver::RoutingSetup,
    ) -> Self {
        let mut graph = AudioGraph::new();

        for (name, node_config) in &routing_setup.nodes {
            let _ = graph.upsert_bus(name, node_config.alias.clone(), node_config.effects.clone());
        }

        for route in &routing_setup.routes {
            let source = canonicalize_node_id(&route.source);
            let destination = canonicalize_node_id(&route.destination);
            let _ = graph.upsert_bus(source.clone(), None, None);
            let _ = graph.upsert_bus(destination.clone(), None, None);

            let gain = route
                .effects
                .as_ref()
                .and_then(extract_route_gain)
                .unwrap_or(1.0);

            let _ = graph.add_route(AudioRoute::new(source, destination).with_volume(gain));
        }

        for duck in &routing_setup.ducks {
            let source = canonicalize_node_id(&duck.source);
            let destination = canonicalize_node_id(&duck.destination);
            let _ = graph.upsert_bus(source.clone(), None, None);
            let _ = graph.upsert_bus(destination.clone(), None, None);
            let _ = graph.add_route(AudioRoute::duck(source, destination, duck.effect.clone()));
        }

        for sidechain in &routing_setup.sidechains {
            let source = canonicalize_node_id(&sidechain.source);
            let destination = canonicalize_node_id(&sidechain.destination);
            let _ = graph.upsert_bus(source.clone(), None, None);
            let _ = graph.upsert_bus(destination.clone(), None, None);
            let _ = graph.add_route(AudioRoute::sidechain(
                source,
                destination,
                sidechain.effect.clone(),
            ));
        }

        graph.optimized()
    }

    /// Get node by ID
    pub fn get_node(&self, id: &str) -> Option<&AudioNode> {
        self.nodes.get(id)
    }

    /// Get mutable node by ID
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut AudioNode> {
        self.nodes.get_mut(id)
    }

    /// Get all routes from a node
    pub fn get_routes_from(&self, node_id: &str) -> Vec<&AudioRoute> {
        self.routes.iter().filter(|r| r.from == node_id).collect()
    }

    /// Get all routes to a node
    pub fn get_routes_to(&self, node_id: &str) -> Vec<&AudioRoute> {
        self.routes.iter().filter(|r| r.to == node_id).collect()
    }

    /// Count total nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Count total routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Validate graph consistency
    pub fn validate(&self) -> Result<()> {
        // Check master exists
        if !self.nodes.contains_key(&self.master) {
            return Err(anyhow!("Master node '{}' does not exist", self.master));
        }

        // Check all routes reference valid nodes
        for route in &self.routes {
            if !self.nodes.contains_key(&route.from) {
                return Err(anyhow!("Route source '{}' does not exist", route.from));
            }
            if !self.nodes.contains_key(&route.to) {
                return Err(anyhow!("Route destination '{}' does not exist", route.to));
            }
        }

        Ok(())
    }

    /// Get all node names.
    pub fn node_names(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }

    /// Check if a node exists.
    pub fn has_node(&self, node_id: &str) -> bool {
        self.nodes.contains_key(&canonicalize_node_id(node_id))
    }

    /// Return an optimized copy with duplicate mix routes merged and node
    /// effects normalized.
    pub fn optimized(&self) -> Self {
        let mut optimized = self.clone();
        optimized.merge_duplicate_mix_routes();
        optimized.normalize_node_effects();
        optimized
    }

    fn merge_duplicate_mix_routes(&mut self) {
        let mut mix_routes: HashMap<(String, String), AudioRoute> = HashMap::new();
        let mut routes = Vec::new();

        for route in self.routes.drain(..) {
            if route.kind == AudioRouteKind::Mix {
                let key = (route.from.clone(), route.to.clone());
                mix_routes
                    .entry(key)
                    .and_modify(|existing| existing.volume += route.volume)
                    .or_insert(route);
            } else {
                routes.push(route);
            }
        }

        routes.extend(mix_routes.into_values().filter(|route| route.volume != 0.0));
        self.routes = routes;
    }

    fn normalize_node_effects(&mut self) {
        for node in self.nodes.values_mut() {
            if node.effects.is_empty() {
                continue;
            }

            let effects = Value::Array(
                node.effects
                    .iter()
                    .map(|(_, params)| params.clone())
                    .collect(),
            );
            node.effects = normalize_effect_value(effects)
                .into_iter()
                .map(effect_to_node_effect)
                .collect();
        }
    }
}

impl Default for AudioGraph {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_route_gain(effects: &Value) -> Option<f32> {
    if let Value::Map(effects_map) = effects {
        if let Some(Value::Map(volume_map)) = effects_map.get("volume") {
            if let Some(Value::Number(gain)) = volume_map.get("gain") {
                return Some(*gain);
            }
        }
        if let Some(Value::Number(gain)) = effects_map.get("volume") {
            return Some(*gain);
        }
    }
    None
}

fn effect_to_node_effect(effect: Value) -> (String, Value) {
    let name = effect_name(&effect).unwrap_or_else(|| "effect".to_string());
    (name, effect)
}

fn normalize_effect_value(effects: Value) -> Vec<Value> {
    let flattened = flatten_effects(effects);
    let mut normalized = Vec::new();

    for effect in flattened {
        if let Some(effect_key) = effect_name(&effect) {
            if let Some(last) = normalized.last_mut() {
                if let Some(last_key) = effect_name(last) {
                    if last_key == effect_key {
                        merge_effect_items(last, &effect);
                        continue;
                    }
                }
            }
        }

        normalized.push(effect);
    }

    normalized
}

fn flatten_effects(effects: Value) -> Vec<Value> {
    match effects {
        Value::Array(arr) => arr.into_iter().flat_map(flatten_effects).collect(),
        other => vec![other],
    }
}

fn effect_name(value: &Value) -> Option<String> {
    match value {
        Value::String(s) | Value::Identifier(s) => Some(s.clone()),
        Value::Map(map) => {
            if let Some(effect_type) = map.get("type").or_else(|| map.get("effect")) {
                match effect_type {
                    Value::String(s) | Value::Identifier(s) => Some(s.clone()),
                    _ => None,
                }
            } else if map.len() == 1 {
                map.keys().next().cloned()
            } else {
                None
            }
        }
        _ => None,
    }
}

fn merge_effect_items(target: &mut Value, source: &Value) {
    if let Value::Map(target_map) = target {
        if let Value::Map(source_map) = source {
            for (key, value) in source_map {
                target_map.insert(key.clone(), value.clone());
            }
            return;
        }
    }

    if matches!(target, Value::String(_) | Value::Identifier(_)) {
        if let Value::Map(_) = source {
            *target = source.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph() {
        let graph = AudioGraph::new();
        assert_eq!(graph.node_count(), 1); // Master
        assert!(graph.nodes.contains_key("master"));
    }

    #[test]
    fn test_add_node() {
        let mut graph = AudioGraph::new();
        let node = AudioNode::new_synth("synth_1");
        graph.add_node(node).unwrap();
        assert_eq!(graph.node_count(), 2); // Master + synth_1
    }

    #[test]
    fn test_add_duplicate_node() {
        let mut graph = AudioGraph::new();
        let node = AudioNode::new_synth("syn");
        graph.add_node(node.clone()).unwrap();
        let result = graph.add_node(node);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_route() {
        let mut graph = AudioGraph::new();
        let node = AudioNode::new_synth("syn");
        graph.add_node(node).unwrap();

        let route = AudioRoute::new("syn", "master");
        graph.add_route(route).unwrap();
        assert_eq!(graph.route_count(), 1);
    }

    #[test]
    fn test_route_from_nonexistent_node() {
        let mut graph = AudioGraph::new();
        let route = AudioRoute::new("nonexistent", "master");
        assert!(graph.add_route(route).is_err());
    }

    #[test]
    fn test_route_to_nonexistent_node() {
        let mut graph = AudioGraph::new();
        let route = AudioRoute::new("master", "nonexistent");
        assert!(graph.add_route(route).is_err());
    }

    #[test]
    fn test_get_routes_from() {
        let mut graph = AudioGraph::new();
        let syn1 = AudioNode::new_synth("syn1");
        let syn2 = AudioNode::new_synth("syn2");
        graph.add_node(syn1).unwrap();
        graph.add_node(syn2).unwrap();

        graph.add_route(AudioRoute::new("syn1", "master")).unwrap();
        graph.add_route(AudioRoute::new("syn1", "syn2")).unwrap();
        graph.add_route(AudioRoute::new("syn2", "master")).unwrap();

        let from_syn1 = graph.get_routes_from("syn1");
        assert_eq!(from_syn1.len(), 2);
    }

    #[test]
    fn test_route_with_volume_and_pan() {
        let route = AudioRoute::new("syn", "master")
            .with_volume(0.5)
            .with_pan(-0.3);
        assert_eq!(route.volume, 0.5);
        assert_eq!(route.pan, -0.3);
    }

    #[test]
    fn test_canonicalize_master_aliases() {
        assert_eq!(canonicalize_node_id("$master"), "master");
        assert_eq!(canonicalize_node_id("master"), "master");
        assert_eq!(canonicalize_node_id("drums"), "drums");
    }

    #[test]
    fn test_from_routing_setup_creates_bus_routes_and_effects() {
        let mut setup = crate::engine::audio::interpreter::driver::RoutingSetup::default();
        setup.nodes.insert(
            "drums".to_string(),
            crate::engine::audio::interpreter::driver::RoutingNodeConfig {
                name: "drums".to_string(),
                alias: Some("kickPattern".to_string()),
                effects: Some(Value::Array(vec![
                    Value::String("reverb".to_string()),
                    Value::String("reverb".to_string()),
                ])),
            },
        );
        setup
            .routes
            .push(crate::engine::audio::interpreter::driver::RouteConfig {
                source: "drums".to_string(),
                destination: "$master".to_string(),
                effects: Some(Value::Map({
                    let mut volume = HashMap::new();
                    volume.insert("gain".to_string(), Value::Number(0.75));
                    let mut map = HashMap::new();
                    map.insert("volume".to_string(), Value::Map(volume));
                    map
                })),
            });
        setup
            .ducks
            .push(crate::engine::audio::interpreter::driver::DuckConfig {
                source: "kick".to_string(),
                destination: "drums".to_string(),
                effect: Value::String("compressor".to_string()),
            });
        setup
            .sidechains
            .push(crate::engine::audio::interpreter::driver::SidechainConfig {
                source: "drums".to_string(),
                destination: "pad".to_string(),
                effect: Value::String("gate".to_string()),
            });

        let graph = AudioGraph::from_routing_setup(&setup);

        assert!(graph.has_node("master"));
        assert!(graph.has_node("drums"));
        assert!(graph.has_node("kick"));
        assert!(graph.has_node("pad"));
        assert_eq!(graph.nodes["drums"].name, Some("kickPattern".to_string()));
        assert_eq!(graph.nodes["drums"].effects.len(), 1);
        assert!(graph.routes.iter().any(|route| {
            route.kind == AudioRouteKind::Mix
                && route.from == "drums"
                && route.to == "master"
                && (route.volume - 0.75).abs() < f32::EPSILON
        }));
        assert!(
            graph
                .routes
                .iter()
                .any(|route| route.kind == AudioRouteKind::Duck)
        );
        assert!(
            graph
                .routes
                .iter()
                .any(|route| route.kind == AudioRouteKind::Sidechain)
        );
    }
}
