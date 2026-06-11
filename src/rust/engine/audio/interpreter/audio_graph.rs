//! Compatibility facade for the canonical audio graph.
//!
//! Phase 2.4 moved the routing model to `engine::audio::graph`. Keep this
//! module as a public re-export so older imports continue to compile.

pub use crate::engine::audio::graph::{
    AudioGraph, AudioRoute, AudioRouteKind, SidechainMode, SidechainParam, canonicalize_node_id,
};
