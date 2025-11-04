use crate::language::syntax::ast::{Statement, StatementKind, Value};
use crate::language::syntax::parser::driver::effects::parse_chained_effects;
use anyhow::{Result, anyhow};

pub fn parse_routing_command(line_number: usize) -> Result<Statement> {
    // The "routing" keyword itself indicates a block-based routing declaration
    // The body will be filled in during indentation parsing in the main driver
    Ok(Statement::new(
        StatementKind::Routing { body: Vec::new() },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse routing block statements (node, fx, route, duck, sidechain)
pub fn parse_routing_statement<'a>(line: &str, line_number: usize) -> Result<Statement> {
    let trimmed = line.trim();

    // node <name> [= <alias>]
    if trimmed.starts_with("node ") {
        let rest = trimmed[5..].trim();
        if let Some((name, alias)) = rest.split_once('=') {
            let name = name.trim().trim_end_matches(':').to_string();
            let alias = alias.trim().trim_end_matches(':').to_string();
            return Ok(Statement::new(
                StatementKind::RoutingNode {
                    name,
                    alias: Some(alias),
                },
                Value::Null,
                0,
                line_number,
                1,
            ));
        } else {
            let name = rest.trim_end_matches(':').to_string();
            return Ok(Statement::new(
                StatementKind::RoutingNode { name, alias: None },
                Value::Null,
                0,
                line_number,
                1,
            ));
        }
    }

    // fx <target> -> effect1 -> effect2 ...
    if trimmed.starts_with("fx ") {
        let rest = trimmed[3..].trim();
        if let Some((target, effects_str)) = rest.split_once("->") {
            let target = target.trim().to_string();
            let effects_chain = format!("->{}", effects_str);
            let effects = parse_chained_effects(&effects_chain)?;
            return Ok(Statement::new(
                StatementKind::RoutingFx { target, effects },
                Value::Null,
                0,
                line_number,
                1,
            ));
        } else {
            return Err(anyhow!(
                "fx statement requires effects after '->': {}",
                trimmed
            ));
        }
    }

    // route <source> to <dest> with effect(...)
    if trimmed.starts_with("route ") {
        let rest = trimmed[6..].trim();
        if let Some((source_part, rest)) = rest.split_once(" to ") {
            let source = source_part.trim().to_string();
            if let Some((dest_part, effect_part)) = rest.split_once(" with ") {
                let destination = dest_part.trim().to_string();
                let effect_str = effect_part.trim().trim_end_matches(':');
                let effects = parse_single_routing_effect(effect_str)?;
                return Ok(Statement::new(
                    StatementKind::RoutingRoute {
                        source,
                        destination,
                        effects: Some(effects),
                    },
                    Value::Null,
                    0,
                    line_number,
                    1,
                ));
            } else {
                let destination = rest.trim().trim_end_matches(':').to_string();
                return Ok(Statement::new(
                    StatementKind::RoutingRoute {
                        source,
                        destination,
                        effects: None,
                    },
                    Value::Null,
                    0,
                    line_number,
                    1,
                ));
            }
        } else {
            return Err(anyhow!(
                "route statement requires format: route <source> to <dest> [with effect(...)]: {}",
                trimmed
            ));
        }
    }

    // duck <source> to <dest> with effect(...)
    if trimmed.starts_with("duck ") {
        let rest = trimmed[5..].trim();
        if let Some((source_part, rest)) = rest.split_once(" to ") {
            let source = source_part.trim().to_string();
            if let Some((dest_part, effect_part)) = rest.split_once(" with ") {
                let destination = dest_part.trim().to_string();
                let effect_str = effect_part.trim().trim_end_matches(':');
                let effect = parse_single_routing_effect(effect_str)?;
                return Ok(Statement::new(
                    StatementKind::RoutingDuck {
                        source,
                        destination,
                        effect,
                    },
                    Value::Null,
                    0,
                    line_number,
                    1,
                ));
            } else {
                return Err(anyhow!(
                    "duck statement requires 'with' clause: {}",
                    trimmed
                ));
            }
        } else {
            return Err(anyhow!(
                "duck statement requires format: duck <source> to <dest> with effect(...): {}",
                trimmed
            ));
        }
    }

    // sidechain <source> to <dest> with effect(...)
    if trimmed.starts_with("sidechain ") {
        let rest = trimmed[10..].trim();
        if let Some((source_part, rest)) = rest.split_once(" to ") {
            let source = source_part.trim().to_string();
            if let Some((dest_part, effect_part)) = rest.split_once(" with ") {
                let destination = dest_part.trim().to_string();
                let effect_str = effect_part.trim().trim_end_matches(':');
                let effect = parse_single_routing_effect(effect_str)?;
                return Ok(Statement::new(
                    StatementKind::RoutingSidechain {
                        source,
                        destination,
                        effect,
                    },
                    Value::Null,
                    0,
                    line_number,
                    1,
                ));
            } else {
                return Err(anyhow!(
                    "sidechain statement requires 'with' clause: {}",
                    trimmed
                ));
            }
        } else {
            return Err(anyhow!(
                "sidechain statement requires format: sidechain <source> to <dest> with effect(...): {}",
                trimmed
            ));
        }
    }

    Err(anyhow!("Unknown routing statement: {}", trimmed))
}

/// Parse a single routing effect like "effect({ param: value, ... })"
fn parse_single_routing_effect(effect_str: &str) -> Result<Value> {
    let effect_str = effect_str.trim();
    if effect_str.is_empty() {
        return Err(anyhow!("Empty effect definition"));
    }

    // Handle function-like syntax: effect_name(param1, param2, ...)
    if let Some((name, params_str)) = effect_str.split_once('(') {
        let name = name.trim().to_string();
        let params_str = params_str.trim_end_matches(')');

        // Handle parameter map
        if params_str.contains('{') && params_str.contains('}') {
            let map_str = params_str.trim_matches(|c| c == '{' || c == '}');
            let mut params_map = std::collections::HashMap::new();

            for pair in map_str.split(',') {
                if let Some((key, value)) = pair.split_once(':') {
                    let key = key.trim().to_string();
                    let value = value.trim();

                    let value = if value == "true" {
                        Value::Boolean(true)
                    } else if value == "false" {
                        Value::Boolean(false)
                    } else if let Ok(num) = value.parse::<f32>() {
                        Value::Number(num)
                    } else {
                        Value::String(value.trim_matches('"').to_string())
                    };

                    params_map.insert(key, value);
                }
            }

            let mut result = std::collections::HashMap::new();
            result.insert(name, Value::Map(params_map));
            return Ok(Value::Map(result));
        } else if let Ok(num) = params_str.parse::<f32>() {
            let mut result = std::collections::HashMap::new();
            result.insert(name, Value::Number(num));
            return Ok(Value::Map(result));
        } else {
            let mut result = std::collections::HashMap::new();
            result.insert(
                name,
                Value::String(params_str.trim_matches('"').to_string()),
            );
            return Ok(Value::Map(result));
        }
    } else {
        let mut result = std::collections::HashMap::new();
        result.insert(effect_str.to_string(), Value::Null);
        return Ok(Value::Map(result));
    }
}
