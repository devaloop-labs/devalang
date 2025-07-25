use crate::core::{
    parser::statement::StatementKind,
    shared::{ duration::Duration, value::Value },
    store::variable::VariableTable,
};

pub fn load_trigger(
    trigger: &Value,
    duration: &Duration,
    effects: &Option<Value>,
    base_duration: f32,
    variable_table: VariableTable
) -> (String, f32) {
    let mut trigger_path = String::new();
    let mut duration_as_secs = 0.0;

    match trigger {
        Value::String(src) => {
            trigger_path = src.to_string();
        }
        Value::Identifier(src) => {
            trigger_path = src.to_string();
        }

        Value::Map(map) => {
            if let Some(Value::String(src)) = map.get("entity") {
                trigger_path = format!("devalang://bank/{}", src.to_string());
            } else if let Some(Value::Identifier(src)) = map.get("entity") {
                trigger_path = format!("devalang://bank/{}", src.to_string());
            } else {
                eprintln!(
                    "❌ Trigger map must contain an 'entity' key with a string or identifier value."
                );
            }
        }
        Value::Sample(src) => {
            trigger_path = src.to_string();
        }
        Value::Statement(stmt) => {
            if let StatementKind::Trigger { entity, duration, effects } = &stmt.kind {
                trigger_path = entity.clone();
            } else {
                eprintln!("❌ Trigger statement must be of type 'Trigger', found: {:?}", stmt.kind);
            }
        }
        _ => {
            eprintln!(
                "❌ Invalid trigger type. Expected a string or identifier variable, found: {:?}",
                trigger
            );
        }
    }

    match duration {
        Duration::Identifier(duration_identifier) => {
            if duration_identifier == "auto" {
                duration_as_secs = base_duration;
            } else if let Some(Value::Number(num)) = variable_table.get(duration_identifier) {
                duration_as_secs = *num;
            } else if let Some(Value::String(num_str)) = variable_table.get(duration_identifier) {
                duration_as_secs = num_str.parse::<f32>().unwrap_or(base_duration);
            } else if
                let Some(Value::Identifier(num_str)) = variable_table.get(duration_identifier)
            {
                duration_as_secs = num_str.parse::<f32>().unwrap_or(base_duration);
            } else {
                eprintln!("❌ Invalid duration identifier: {}", duration_identifier);
            }
        }

        Duration::Number(num) => {
            duration_as_secs = *num;
        }

        Duration::Auto => {
            duration_as_secs = base_duration;
        }

        Duration::Beat(beat_str) => {
            let parts: Vec<&str> = beat_str.split('/').collect();

            if parts.len() == 2 {
                let numerator: f32 = parts[0].parse().unwrap_or(1.0);
                let denominator: f32 = parts[1].parse().unwrap_or(1.0);
                duration_as_secs = (numerator / denominator) * base_duration;
            } else {
                eprintln!("❌ Invalid beat duration format: {}", beat_str);
            }
        }

        _ => {
            eprintln!("❌ Invalid duration type. Expected an identifier.");
        }
    }

    (trigger_path, duration_as_secs)
}
