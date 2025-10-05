// Resolver module - handles statement resolution and variable lookup
pub mod bank;
pub mod call;
pub mod condition;
pub mod driver;
pub mod group;
pub mod let_;
pub mod loop_;
pub mod pattern;
pub mod spawn;
pub mod tempo;
pub mod trigger;
pub mod value;

pub use driver::resolve_statement;
pub use value::resolve_value;
