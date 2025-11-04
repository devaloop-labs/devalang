/// Audio Graph - Representation of routing and mixing graph

use crate::language::syntax::ast::Value;
use std::collections::HashMap;

/// Configuration of a node in the audio graph
#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub alias: Option<String>,
    pub effects: Option<Value>, // Effect chain to apply to this node
}

/// A connection/route between two nodes
#[derive(Debug, Clone)]
pub enum Connection {
    /// Simple route: mix source to destination with gain
    Route {
        source: String,
        destination: String,
        gain: f32,
    },
    /// Duck: compress source based on destination envelope
    Duck {
        source: String,
        destination: String,
        effect_params: Value, // Compressor parameters
    },
    /// Sidechain: apply effect to source driven by destination
    Sidechain {
        source: String,
        destination: String,
        effect_params: Value, // Gate or other effect parameters
    },
}

/// The complete audio graph
#[derive(Debug, Clone)]
pub struct AudioGraph {
    /// All nodes (channels/tracks)
    pub nodes: HashMap<String, Node>,
    /// All connections between nodes
    pub connections: Vec<Connection>,
    /// Master node (final mixer) - always "$master"
    pub master_node: String,
}

impl AudioGraph {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        
        // Create default master node
        nodes.insert(
            "$master".to_string(),
            Node {
                name: "$master".to_string(),
                alias: None,
                effects: None,
            },
        );
        
        Self {
            nodes,
            connections: Vec::new(),
            master_node: "$master".to_string(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, name: String, alias: Option<String>, effects: Option<Value>) {
        self.nodes.insert(
            name.clone(),
            Node { name, alias, effects },
        );
    }

    /// Add a connection to the graph
    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
    }

    /// Build the graph from routing configuration
    pub fn from_routing_setup(routing_setup: &crate::engine::audio::interpreter::driver::RoutingSetup) -> Self {
        let mut graph = AudioGraph::new();

        // Add all nodes
        for (name, node_config) in &routing_setup.nodes {
            graph.add_node(
                name.clone(),
                node_config.alias.clone(),
                node_config.effects.clone(),
            );
        }

        // Add routes
        for route in &routing_setup.routes {
            let gain = if let Some(Value::Map(effects_map)) = &route.effects {
                if let Some(Value::Map(volume_map)) = effects_map.get("volume") {
                    if let Some(Value::Number(g)) = volume_map.get("gain") {
                        *g
                    } else {
                        1.0
                    }
                } else if let Some(Value::Number(g)) = effects_map.get("volume") {
                    *g
                } else {
                    1.0
                }
            } else {
                1.0
            };

            graph.add_connection(Connection::Route {
                source: route.source.clone(),
                destination: route.destination.clone(),
                gain,
            });
        }

        // Add ducks
        for duck in &routing_setup.ducks {
            graph.add_connection(Connection::Duck {
                source: duck.source.clone(),
                destination: duck.destination.clone(),
                effect_params: duck.effect.clone(),
            });
        }

        // Add sidechains
        for sidechain in &routing_setup.sidechains {
            graph.add_connection(Connection::Sidechain {
                source: sidechain.source.clone(),
                destination: sidechain.destination.clone(),
                effect_params: sidechain.effect.clone(),
            });
        }

        graph
    }

    /// Get outgoing routes from a node
    pub fn get_outgoing_routes(&self, node_name: &str) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|conn| match conn {
                Connection::Route { source, .. } => source == node_name,
                _ => false,
            })
            .collect()
    }

    /// Get ducks affecting a node
    pub fn get_incoming_ducks(&self, node_name: &str) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|conn| match conn {
                Connection::Duck { source, .. } if source == node_name => true,
                _ => false,
            })
            .collect()
    }

    /// Get sidechains affecting a node
    pub fn get_incoming_sidechains(&self, node_name: &str) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|conn| match conn {
                Connection::Sidechain { source, .. } if source == node_name => true,
                _ => false,
            })
            .collect()
    }

    /// Check if a node exists
    pub fn has_node(&self, node_name: &str) -> bool {
        self.nodes.contains_key(node_name)
    }

    /// Get all node names
    pub fn node_names(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }

    /// Print debug information about the audio graph
    #[allow(dead_code)]
    pub fn debug_print(&self) {
        // This method is kept for debugging purposes but not called in normal operation
    }
}

impl Default for AudioGraph {
    fn default() -> Self {
        Self::new()
    }
}
