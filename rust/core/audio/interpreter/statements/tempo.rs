use crate::core::parser::statement::{Statement, StatementKind};
use devalang_types::Value;
use devalang_utils::logger::{LogLevel, Logger};

pub fn interprete_tempo_statement(stmt: &Statement) -> Option<(f32, f32)> {
    if let StatementKind::Tempo = &stmt.kind {
        match &stmt.value {
            Value::Number(bpm) => {
                let bpm = { *bpm };
                let duration = 60.0 / bpm;

                return Some((bpm, duration));
            }

            Value::String(bpm_str) => {
                if let Ok(bpm) = bpm_str.parse::<f32>() {
                    let duration = 60.0 / bpm;
                    return Some((bpm, duration));
                }
            }

            Value::Identifier(bpm_ident) => {
                if let Ok(bpm) = bpm_ident.parse::<f32>() {
                    let duration = 60.0 / bpm;
                    return Some((bpm, duration));
                }
            }

            _ => {
                let logger = Logger::new();
                logger.log_message(
                    LogLevel::Warning,
                    format!("Invalid tempo value: {:?}", stmt.value).as_str(),
                );
            }
        }
    }

    None
}
