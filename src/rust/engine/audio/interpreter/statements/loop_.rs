use crate::engine::audio::interpreter::driver::AudioInterpreter;
use crate::language::syntax::ast::{Statement, Value};
/// Loop and For statement execution
use anyhow::Result;

impl AudioInterpreter {
    /// Execute a Loop statement (repeat N times)
    pub fn execute_loop(&mut self, count: &Value, body: &[Statement]) -> Result<()> {
        // Extract loop count
        let loop_count = match count {
            Value::Number(n) => (*n) as usize,
            Value::Identifier(ident) => {
                // Try to get variable value
                if let Some(Value::Number(n)) = self.variables.get(ident) {
                    *n as usize
                } else {
                    anyhow::bail!("❌ Loop iterator '{}' must be a number", ident);
                }
            }
            _ => {
                anyhow::bail!("❌ Loop iterator must be a number, found: {:?}", count);
            }
        };

        // Execute body N times
        for _iteration in 0..loop_count {
            self.collect_events(body)?;
        }

        Ok(())
    }

    /// Execute a For statement (foreach item in array/range)
    pub fn execute_for(
        &mut self,
        variable: &str,
        iterable: &Value,
        body: &[Statement],
    ) -> Result<()> {
        // Extract items to iterate over
        let items = match iterable {
            Value::Array(arr) => arr.clone(),
            Value::Identifier(ident) => {
                // Try to get variable value
                if let Some(Value::Array(arr)) = self.variables.get(ident) {
                    arr.clone()
                } else {
                    anyhow::bail!("❌ For iterable '{}' must be an array", ident);
                }
            }
            Value::Range { start, end } => {
                // Generate range [start..end]
                let start_val = match start.as_ref() {
                    Value::Number(n) => *n as i32,
                    _ => anyhow::bail!("❌ Range start must be a number"),
                };
                let end_val = match end.as_ref() {
                    Value::Number(n) => *n as i32,
                    _ => anyhow::bail!("❌ Range end must be a number"),
                };

                // Create array from range
                (start_val..=end_val)
                    .map(|i| Value::Number(i as f32))
                    .collect()
            }
            _ => {
                anyhow::bail!(
                    "❌ For iterable must be an array or range, found: {:?}",
                    iterable
                );
            }
        };

        // Execute body for each item
        for (_idx, item) in items.iter().enumerate() {
            // Set loop variable
            let old_value = self.variables.insert(variable.to_string(), item.clone());

            // Execute body
            self.collect_events(body)?;

            // Restore old value or remove
            match old_value {
                Some(val) => self.variables.insert(variable.to_string(), val),
                None => self.variables.remove(variable),
            };
        }

        Ok(())
    }
}
