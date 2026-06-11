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

    // route <source> to <dest> [with effect(...)]
    // route <source> -> <dest> [@ level]
    if trimmed.starts_with("route ") {
        let rest = trimmed[6..].trim();
        let source;
        let remaining;

        if let Some((source_part, rest)) = rest.split_once(" to ") {
            source = source_part.trim().to_string();
            remaining = rest.trim();
        } else if let Some((source_part, rest)) = rest.split_once("->") {
            source = source_part.trim().to_string();
            remaining = rest.trim();
        } else {
            return Err(anyhow!(
                "route statement requires format: route <source> to <dest> [with effect(...)] or route <source> -> <dest> [@ level]: {}",
                trimmed
            ));
        }

        if let Some((dest_part, effect_part)) = remaining.split_once(" with ") {
            let destination = normalize_routing_target(dest_part.trim().trim_end_matches(':'));
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
        }

        let (destination, effects) = parse_destination_and_gain(remaining)?;
        return Ok(Statement::new(
            StatementKind::RoutingRoute {
                source,
                destination,
                effects,
            },
            Value::Null,
            0,
            line_number,
            1,
        ));
    }

    // duck <source> to <dest> with effect(...)
    // duck <source> -> <dest> with effect(...)
    if trimmed.starts_with("duck ") {
        let rest = trimmed[5..].trim();
        let source;
        let remaining;

        if let Some((source_part, rest)) = rest.split_once(" to ") {
            source = source_part.trim().to_string();
            remaining = rest.trim();
        } else if let Some((source_part, rest)) = rest.split_once("->") {
            source = source_part.trim().to_string();
            remaining = rest.trim();
        } else {
            return Err(anyhow!(
                "duck statement requires format: duck <source> to <dest> with effect(...): {}",
                trimmed
            ));
        }

        if let Some((dest_part, effect_part)) = remaining.split_once(" with ") {
            let destination = normalize_routing_target(dest_part.trim().trim_end_matches(':'));
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
        }

        return Err(anyhow!(
            "duck statement requires 'with' clause: {}",
            trimmed
        ));
    }

    // sidechain <source> to <dest> with effect(...)
    // sidechain <source> -> <dest> with effect(...)
    if trimmed.starts_with("sidechain ") {
        let rest = trimmed[10..].trim();
        let source;
        let remaining;

        if let Some((source_part, rest)) = rest.split_once(" to ") {
            source = source_part.trim().to_string();
            remaining = rest.trim();
        } else if let Some((source_part, rest)) = rest.split_once("->") {
            source = source_part.trim().to_string();
            remaining = rest.trim();
        } else {
            return Err(anyhow!(
                "sidechain statement requires format: sidechain <source> to <dest> with effect(...): {}",
                trimmed
            ));
        }

        if let Some((dest_part, effect_part)) = remaining.split_once(" with ") {
            let destination = normalize_routing_target(dest_part.trim().trim_end_matches(':'));
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
        }

        return Err(anyhow!(
            "sidechain statement requires 'with' clause: {}",
            trimmed
        ));
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

fn normalize_routing_target(target: &str) -> String {
    if target == "master" || target == "$master" {
        "master".to_string()
    } else {
        target.to_string()
    }
}

fn parse_destination_and_gain(dest_expr: &str) -> Result<(String, Option<Value>)> {
    let mut parts = dest_expr.trim().split('@');
    let destination =
        normalize_routing_target(parts.next().unwrap_or("").trim().trim_end_matches(':'));
    let effects = if let Some(gain_str) = parts.next() {
        let gain_str = gain_str.trim();
        if gain_str.is_empty() {
            None
        } else {
            let gain = gain_str
                .parse::<f32>()
                .map_err(|_| anyhow!("Invalid mix level: {}", gain_str))?;
            let mut volume_map = std::collections::HashMap::new();
            volume_map.insert("gain".to_string(), Value::Number(gain));
            let mut map = std::collections::HashMap::new();
            map.insert("volume".to_string(), Value::Map(volume_map));
            Some(Value::Map(map))
        }
    } else {
        None
    };
    Ok((destination, effects))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_routing_fx_eq_chain() {
        let statement =
            parse_routing_statement("fx master -> eq({ low: 1.2, mid: 0.8, high: 1.1 })", 1)
                .unwrap();

        match statement.kind {
            StatementKind::RoutingFx { target, effects } => {
                assert_eq!(target, "master");
                match effects {
                    Value::Map(map) => {
                        assert!(matches!(map.get("eq"), Some(Value::Map(_))));
                    }
                    other => panic!("expected effects map, got {:?}", other),
                }
            }
            other => panic!("expected routing fx, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_routing_arrow_with_gain() {
        let statement = parse_routing_statement("route drums -> master @ 0.8", 1).unwrap();

        match statement.kind {
            StatementKind::RoutingRoute {
                source,
                destination,
                effects,
            } => {
                assert_eq!(source, "drums");
                assert_eq!(destination, "master");
                match effects {
                    Some(Value::Map(map)) => match map.get("volume") {
                        Some(Value::Map(volume_map)) => match volume_map.get("gain") {
                            Some(Value::Number(gain)) => {
                                assert!((gain - 0.8).abs() < f32::EPSILON)
                            }
                            other => panic!("expected gain number, got {:?}", other),
                        },
                        other => panic!("expected volume map, got {:?}", other),
                    },
                    other => panic!("expected route effects map, got {:?}", other),
                }
            }
            other => panic!("expected routing route, got {:?}", other),
        }
    }
}
