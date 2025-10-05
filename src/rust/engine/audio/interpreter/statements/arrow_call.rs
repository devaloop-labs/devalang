use crate::engine::functions::{FunctionContext, FunctionRegistry};
/// Arrow call statement handler
use anyhow::{Result, anyhow};

pub fn execute_arrow_call(
    registry: &FunctionRegistry,
    target: &str,
    method: &str,
    args: &[crate::language::syntax::ast::Value],
    chain: Option<&[crate::language::syntax::ast::Value]>,
    cursor_time: f32,
    tempo: f32,
) -> Result<FunctionContext> {
    // Create initial context
    let mut context = FunctionContext {
        target: target.to_string(),
        state: std::collections::HashMap::new(),
        start_time: cursor_time,
        duration: 0.0,
        tempo,
    };

    // Execute first method
    registry.execute(method, &mut context, args)?;

    // Execute chained methods if any
    if let Some(chain_calls) = chain {
        for call_value in chain_calls {
            if let crate::language::syntax::ast::Value::Map(call_map) = call_value {
                // Extract method and args from map
                let method_name = call_map
                    .get("method")
                    .and_then(|v| {
                        if let crate::language::syntax::ast::Value::String(s) = v {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| anyhow!("Chain call missing method name"))?;

                let method_args = call_map
                    .get("args")
                    .and_then(|v| {
                        if let crate::language::syntax::ast::Value::Array(a) = v {
                            Some(a.as_slice())
                        } else {
                            None
                        }
                    })
                    .unwrap_or(&[]);

                // Execute chained method
                registry.execute(method_name, &mut context, method_args)?;
            }
        }
    }

    Ok(context)
}
