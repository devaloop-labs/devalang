#[cfg(feature = "cli")]
pub mod registry;

#[cfg(not(feature = "cli"))]
pub mod registry {}
