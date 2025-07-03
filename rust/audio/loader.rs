use crate::core::{ shared::{ duration::Duration, value::Value }, store::variable::VariableTable };

pub fn load_trigger(
    trigger: &Value,
    duration: &Duration,
    base_duration: f32,
    variable_table: VariableTable
) -> (String, f32) {
    let mut trigger_path = String::new();
    let mut duration_as_secs = 0.0;

    match trigger {
        Value::String(src) => {
            trigger_path = src.to_string();
        }
        _ => {
            eprintln!("❌ Invalid trigger type. Expected a text variable.");
        }
    }

    match duration {
        Duration::Identifier(duration_identifier) => {
            if duration_identifier == "auto" {
                duration_as_secs = base_duration;
            } else if let Some(Value::Number(num)) = variable_table.get(duration_identifier) {
                duration_as_secs = *num;
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

        _ => {
            eprintln!("❌ Invalid duration type. Expected an identifier.");
        }
    }

    (trigger_path, duration_as_secs)
}
