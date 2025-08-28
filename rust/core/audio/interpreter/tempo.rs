use crate::core::{
    parser::statement::{Statement, StatementKind},
    shared::value::Value,
};

pub fn interprete_tempo_statement(stmt: &Statement) -> Option<(f32, f32)> {
    if let StatementKind::Tempo = &stmt.kind {
        if let Value::Number(bpm) = &stmt.value {
            let bpm = *bpm as f32;
            let duration = 60.0 / bpm;

            return Some((bpm, duration));
        } else {
            eprintln!("❌ Invalid tempo value: {:?}", stmt.value);
        }
    }

    None
}
