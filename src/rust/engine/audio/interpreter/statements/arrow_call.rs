use crate::engine::functions::FunctionContext;
/// Arrow call statement handler
use anyhow::{Result, anyhow};

pub fn execute_arrow_call(
    interpreter: &mut crate::engine::audio::interpreter::driver::AudioInterpreter,
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

    // Record the invocation method (so downstream can distinguish note vs synth chained calls)
    context.set(
        "method",
        crate::language::syntax::ast::Value::String(method.to_string()),
    );

    // Resolve argument identifiers to runtime values before executing functions
    let resolved_args: Vec<crate::language::syntax::ast::Value> = args
        .iter()
        .map(|v| interpreter.resolve_value(v))
        .collect::<Result<Vec<_>>>()?;

    // Execute first method
    if interpreter.function_registry.has(method) {
        interpreter
            .function_registry
            .execute(method, &mut context, &resolved_args)?;
    } else {
        // Not a registered function: treat as a routing/target name
        context.set(
            "route_to",
            crate::language::syntax::ast::Value::String(method.to_string()),
        );
    }

    // Execute chained methods if any
    if let Some(chain_calls) = chain {
        // Collect unknown chained calls as effect definitions so they can be attached to the
        // resulting FunctionContext for later extraction into per-event effects.
        let mut unknown_effects: Vec<crate::language::syntax::ast::Value> = Vec::new();
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

                // Resolve chained method args using interpreter
                let resolved_method_args: Vec<crate::language::syntax::ast::Value> = method_args
                    .iter()
                    .map(|v| interpreter.resolve_value(v))
                    .collect::<Result<Vec<_>>>()?;

                // Execute chained method if known; otherwise collect as effect map
                if interpreter.function_registry.has(method_name) {
                    interpreter.function_registry.execute(
                        method_name,
                        &mut context,
                        &resolved_method_args,
                    )?;
                } else {
                    // Convert chained call_map (method+args) into an effect-style map
                    // Expected effect map shape: { "type": "lfo", ...params... }
                    let mut effect_map: std::collections::HashMap<
                        String,
                        crate::language::syntax::ast::Value,
                    > = std::collections::HashMap::new();
                    effect_map.insert(
                        "type".to_string(),
                        crate::language::syntax::ast::Value::String(method_name.to_string()),
                    );

                    // If first arg is a Map, merge it as params
                    if let Some(first_arg) = resolved_method_args.get(0) {
                        if let crate::language::syntax::ast::Value::Map(m) = first_arg {
                            for (k, v) in m.iter() {
                                effect_map.insert(k.clone(), v.clone());
                            }
                        } else {
                            // otherwise, store as single 'value' param
                            effect_map.insert("value".to_string(), first_arg.clone());
                        }
                    }

                    unknown_effects.push(crate::language::syntax::ast::Value::Map(effect_map));
                }
            }
        }

        if !unknown_effects.is_empty() {
            context.set(
                "effects",
                crate::language::syntax::ast::Value::Array(unknown_effects),
            );
        }
    }

    Ok(context)
}
