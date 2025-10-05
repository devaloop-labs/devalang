//! Debug logging and error tracking for WASM

use serde::{Deserialize, Serialize};
use std::cell::RefCell;

/// Structured error with location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    #[serde(rename = "type")]
    pub error_type: String,
}

impl ParseError {
    pub fn new(message: String, line: usize, column: usize, error_type: String) -> Self {
        Self {
            message,
            line,
            column,
            error_type,
        }
    }
}

thread_local! {
    /// Log of sample loading events
    pub static SAMPLE_LOAD_LOG: RefCell<Vec<String>> = RefCell::new(Vec::new());

    /// Log of playback debug messages
    pub static PLAYBACK_DEBUG_LOG: RefCell<Vec<String>> = RefCell::new(Vec::new());

    /// Flag to enable/disable error debugging
    pub static WASM_DEBUG_ERROR_FLAG: RefCell<bool> = RefCell::new(false);

    /// Storage for last errors (legacy string format)
    pub static LAST_ERRORS: RefCell<Vec<String>> = RefCell::new(Vec::new());

    /// Storage for structured parse errors
    pub static PARSE_ERRORS: RefCell<Vec<ParseError>> = RefCell::new(Vec::new());
}

/// Log a sample loading event
pub fn log_sample_load(message: String) {
    SAMPLE_LOAD_LOG.with(|log| {
        log.borrow_mut().push(message);
    });
}

/// Log a playback debug message
pub fn log_playback_debug(message: String) {
    PLAYBACK_DEBUG_LOG.with(|log| {
        log.borrow_mut().push(message);
    });
}

/// Get and optionally clear sample load log
pub fn get_sample_load_log(clear: bool) -> Vec<String> {
    SAMPLE_LOAD_LOG.with(|log| {
        let result = log.borrow().clone();
        if clear {
            log.borrow_mut().clear();
        }
        result
    })
}

/// Get and optionally clear playback debug log
pub fn get_playback_debug_log(clear: bool) -> Vec<String> {
    PLAYBACK_DEBUG_LOG.with(|log| {
        let result = log.borrow().clone();
        if clear {
            log.borrow_mut().clear();
        }
        result
    })
}

/// Set debug error flag
pub fn set_debug_errors(enable: bool) {
    WASM_DEBUG_ERROR_FLAG.with(|flag| {
        *flag.borrow_mut() = enable;
    });
}

/// Check if debug errors are enabled
pub fn is_debug_errors_enabled() -> bool {
    WASM_DEBUG_ERROR_FLAG.with(|flag| *flag.borrow())
}

/// Store an error
pub fn push_error(error: String) {
    LAST_ERRORS.with(|errors| {
        errors.borrow_mut().push(error);
    });
}

/// Store a structured parse error
pub fn push_parse_error(error: ParseError) {
    PARSE_ERRORS.with(|errors| {
        errors.borrow_mut().push(error);
    });
}

/// Store a parse error from components
pub fn push_parse_error_from_parts(
    message: String,
    line: usize,
    column: usize,
    error_type: String,
) {
    push_parse_error(ParseError::new(message, line, column, error_type));
}

/// Get and optionally clear last errors
pub fn get_errors(clear: bool) -> Vec<String> {
    LAST_ERRORS.with(|errors| {
        let result = errors.borrow().clone();
        if clear {
            errors.borrow_mut().clear();
        }
        result
    })
}

/// Get and optionally clear parse errors
pub fn get_parse_errors(clear: bool) -> Vec<ParseError> {
    PARSE_ERRORS.with(|errors| {
        let result = errors.borrow().clone();
        if clear {
            errors.borrow_mut().clear();
        }
        result
    })
}

/// Clear all debug logs
pub fn clear_all_logs() {
    SAMPLE_LOAD_LOG.with(|log| log.borrow_mut().clear());
    PLAYBACK_DEBUG_LOG.with(|log| log.borrow_mut().clear());
    LAST_ERRORS.with(|errors| errors.borrow_mut().clear());
    PARSE_ERRORS.with(|errors| errors.borrow_mut().clear());
}

/// Get debug state summary
#[derive(serde::Serialize)]
pub struct DebugState {
    pub sample_load_count: usize,
    pub playback_debug_count: usize,
    pub error_count: usize,
    pub parse_error_count: usize,
    pub debug_errors_enabled: bool,
}

pub fn get_debug_state() -> DebugState {
    DebugState {
        sample_load_count: SAMPLE_LOAD_LOG.with(|log| log.borrow().len()),
        playback_debug_count: PLAYBACK_DEBUG_LOG.with(|log| log.borrow().len()),
        error_count: LAST_ERRORS.with(|errors| errors.borrow().len()),
        parse_error_count: PARSE_ERRORS.with(|errors| errors.borrow().len()),
        debug_errors_enabled: is_debug_errors_enabled(),
    }
}
