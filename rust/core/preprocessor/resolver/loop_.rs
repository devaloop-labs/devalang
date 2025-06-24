use crate::core::{
    parser::parse_without_resolving_with_module,
    preprocessor::resolver::resolve_statement,
    types::{
        module::Module,
        statement::{
            Statement,
            StatementIterator,
            StatementKind,
            StatementResolved,
            StatementResolvedValue,
        },
        variable::VariableValue,
    },
};

pub fn resolve_loop_statement(
    loop_statement: &Statement,
    iterator: StatementIterator,
    module: &Module
) -> StatementResolved {
    let mut resolved_iterator = StatementIterator::Unknown;

    match iterator.clone() {
        StatementIterator::Identifier(id) => {
            if let Some(value) = module.variable_table.variables.get(&id) {
                match value {
                    VariableValue::Array(arr) => {
                        resolved_iterator = StatementIterator::Array(
                            parse_without_resolving_with_module(arr.clone(), module)
                        );
                    }
                    VariableValue::Number(num) => {
                        resolved_iterator = StatementIterator::Number(*num);
                    }
                    _ => {
                        eprintln!("⚠️ Unsupported variable type for loop iterator: {:?}", value);
                        resolved_iterator = StatementIterator::Unknown;
                    }
                }
            } else {
                eprintln!("⚠️ Loop iterator variable '{}' not found", id);
                resolved_iterator = StatementIterator::Unknown;
            }
        }
        StatementIterator::Number(num) => {
            resolved_iterator = StatementIterator::Number(num);
        }
        _ => {
            resolved_iterator = iterator.clone();
        }
    }

    let mut resolved_body: StatementResolvedValue = StatementResolvedValue::Unknown;

    match &loop_statement.value {
        VariableValue::Array(arr) => {
            let raw_statements = parse_without_resolving_with_module(arr.clone(), module);

            let mut resolved_statements = Vec::new();

            for raw_stmt in raw_statements {
                let resolved_stmt = resolve_statement(&raw_stmt, module);
                resolved_statements.push(resolved_stmt);
            }

            resolved_body = StatementResolvedValue::Array(resolved_statements);
        }
        _ => {
            resolved_body = StatementResolvedValue::Unknown;
            eprintln!("⚠️ Unsupported value type for loop body: {:?}", loop_statement.value);
        }
    }

    return StatementResolved {
        kind: StatementKind::Loop { iterator: resolved_iterator },
        value: resolved_body,
        indent: loop_statement.indent,
        line: loop_statement.line,
        column: loop_statement.column,
    };
}
