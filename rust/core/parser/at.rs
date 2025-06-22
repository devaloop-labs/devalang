use rodio::source;

use crate::core::types::{
    parser::Parser,
    statement::{ Statement, StatementKind },
    store::GlobalStore,
    token::TokenKind,
    variable::VariableValue,
};

pub fn parse_at(parser: &mut Parser, global_store: &mut GlobalStore) -> Result<Statement, String> {
    let token = parser.peek().ok_or("Unexpected EOF")?.clone();

    // Vérifie que le token est bien un '@'
    if token.kind != TokenKind::At {
        return Err(format!("Expected '@', found {:?}", token.kind));
    }

    // Consomme le token '@'
    parser.next();

    // On attend un identifiant après '@'
    let identifier_token = parser.peek().ok_or("Expected identifier after '@'")?.clone();
    if identifier_token.kind != TokenKind::Identifier {
        return Err(format!("Expected Identifier, found {:?}", identifier_token.kind));
    }

    if identifier_token.lexeme == "export" {
        // Si l'identifiant est "export", on le consomme et on retourne une déclaration spéciale
        parser.next(); // Consomme "export"

        // Skip LBrace if present
        if let Some(next_token) = parser.peek() {
            if next_token.kind == TokenKind::LBrace {
                parser.next(); // Consomme le LBrace
            }
        }

        // On collecte l'intérieur de la déclaration export
        let exportable_tokens = parser.collect_until(|t| { t.kind == TokenKind::RBrace });

        // NOTE: Insert exportable tokens into the export table
        exportable_tokens.iter().for_each(|t| {
            println!("Exporting token: {:?}", t);
            let variable_value = parse_variable_value(t.lexeme.clone(), parser, global_store);

            parser.export_table.exports.insert(t.lexeme.clone(), variable_value.clone());
        });

        return Ok(Statement {
            kind: StatementKind::Export,
            value: VariableValue::Array(exportable_tokens),
            indent: token.indent,
            line: token.line,
            column: token.column,
        });
    } else if identifier_token.lexeme == "import" {
        // Si l'identifiant est "import", on le consomme et on retourne une déclaration spéciale
        parser.next(); // Consomme "import"

        // Skip LBrace if present
        if let Some(next_token) = parser.peek() {
            if next_token.kind == TokenKind::LBrace {
                parser.next(); // Consomme le LBrace
            }
        }

        // On collecte l'intérieur de la déclaration import
        let importable_tokens = parser.collect_until(|t| { t.kind == TokenKind::RBrace });

        parser.next(); // Consomme le RBrace

        // Skip "from" token
        if let Some(from_token) = parser.peek() {
            if from_token.kind == TokenKind::Identifier && from_token.lexeme == "from" {
                parser.next(); // Consomme "from"
            } else {
                return Err(format!("Expected 'from', found {:?}", from_token.kind));
            }
        } else {
            return Err("Expected 'from' after import declaration".into());
        }

        // Collecte la source après le token "DbQuote"
        let source_token = parser.peek().ok_or("Expected source after 'from'")?.clone();
        if source_token.kind != TokenKind::String {
            return Err(format!("Expected String, found {:?}", source_token.kind));
        }

        // Collecte le contenu de la source jusqu'au DbQuote de fermeture
        let mut source_lexeme = source_token.lexeme.clone();

        // NOTE: Insert importable tokens into the import table
        // parser.import_table.imports.extend(
        //     importable_tokens
        //         .iter()
        //         .map(|t| { (t.lexeme.clone(), parse_variable_kind(t.lexeme.clone(), &mut parser)) })
        // );

        importable_tokens.iter().for_each(|t| {
            let variable_value = parse_variable_value(t.lexeme.clone(), parser, global_store);

            parser.import_table.imports.insert(t.lexeme.clone(), variable_value.clone());
        });

        return Ok(Statement {
            kind: StatementKind::Import {
                names: importable_tokens
                    .iter()
                    .map(|t| t.lexeme.clone())
                    .collect(),
                source: source_lexeme,
            },
            value: VariableValue::Array(importable_tokens),
            indent: token.indent,
            line: token.line,
            column: token.column,
        });
    } else {
        // Si l'identifiant n'est ni "export" ni "import", on le consomme normalement
        parser.next(); // Consomme l'identifiant
    }

    // Consomme l'identifiant
    parser.next();

    println!("Parsing '@' statement with identifier: {:?}", identifier_token);

    // Retourne une déclaration de type At
    Ok(Statement {
        kind: StatementKind::Unknown("At statement".into()),
        value: VariableValue::Text(identifier_token.lexeme.clone()),
        indent: token.indent,
        line: token.line,
        column: token.column,
    })
}

fn parse_variable_value(
    lexeme: String,
    parser: &mut Parser,
    global_store: &mut GlobalStore
) -> VariableValue {
    if lexeme.contains('\"') || lexeme.contains('\'') {
        // If the lexeme contains quotes, treat it as a string
        return VariableValue::Text(lexeme);
    } else if lexeme.parse::<f32>().is_ok() {
        // If the lexeme can be parsed as a number, treat it as a number
        return VariableValue::Number(
            lexeme.parse::<f32>().unwrap_or(0.0) // Placeholder value
        );
    } else if lexeme == "true" || lexeme == "false" {
        // If the lexeme is "true" or "false", treat it as a boolean
        return VariableValue::Boolean(lexeme.parse::<bool>().unwrap_or(false));
    } else if lexeme.starts_with('[') && lexeme.ends_with(']') {
        // If the lexeme starts with '[' and ends with ']', treat it as an array
        return VariableValue::Array(vec![]); // TODO
    } else if lexeme.starts_with('{') && lexeme.ends_with('}') {
        // If the lexeme starts with '{' and ends with '}', treat it as an object
        return VariableValue::Map(vec![].into_iter().collect()); // TODO
    } else if parser.import_table.get_import(&lexeme).is_some() {
        return parser.import_table
            .get_import(&lexeme)
            .map(|value| value.clone())
            .unwrap_or_else(|| {
                // If the lexeme is not found in the variable table or import table, return a default value
                VariableValue::Text(format!("Unknown variable: {}", lexeme))
            });
    } else {
        println!("Modules : {:?}", global_store.modules);

        return VariableValue::Text(format!("Unknown type : {}", lexeme)); // Default case for unknown types
    }
}
