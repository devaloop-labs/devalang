use crate::core::{ debugger::Debugger, lexer::token::Token };

pub fn write_lexer_log_file(output_dir: &str, file_name: &str, tokens: Vec<Token>) {
    let debugger = Debugger::new();
    let mut content = String::new();

    for token in tokens {
        content.push_str(&format!("{:?}\n", token));
    }

    debugger.write_log_file(output_dir, file_name, &content);
}
