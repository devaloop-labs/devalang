// Parent `services` module controls `cli` gating; avoid duplicating crate-level cfg here.
pub mod outputs;
pub mod pipeline;

pub use pipeline::{BuildArtifacts, BuildRequest, ProjectBuilder};
