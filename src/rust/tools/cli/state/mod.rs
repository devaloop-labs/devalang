#![cfg(feature = "cli")]

use std::sync::Arc;

use crate::tools::logger::Logger;

#[derive(Clone)]
pub struct CliContext {
    logger: Arc<Logger>,
}

impl CliContext {
    pub fn new() -> Self {
        Self {
            logger: Arc::new(Logger::new()),
        }
    }

    pub fn logger(&self) -> Arc<Logger> {
        Arc::clone(&self.logger)
    }
}
