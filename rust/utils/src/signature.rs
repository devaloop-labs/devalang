#[cfg(feature = "cli")]
pub fn get_signature(version: &str) -> String {
    let signature = format!(
        r#"
   /|_/|                    
  / ^ ^(_o                  🦊 Devalang
 /    __.'                  
 /     \                    A programming language for music and sound.
/  _   \_                   Part of the Devaloop project.
(_) (_) '._                 
  '.__     '. .-''-'.       https://devalang.com
     ( '.   ('.____.''
     _) )'_, )              v{}
    (__/ (__/
"#,
        version
    );

    signature
}

#[cfg(not(feature = "cli"))]
pub fn get_signature(version: &str) -> String {
    let signature = format!(
        r#"
   /|_/|                    
  / ^ ^(_o                  🦊 Devalang
 /    __.'                  
 /     \                    A programming language for music and sound.
/  _   \_                   Part of the Devaloop project.
(_) (_) '._                 
  '.__     '. .-''-'.       https://devalang.com
     ( '.   ('.____.''
     _) )'_, )              v{}
    (__/ (__/
"#,
        version
    );

    signature
}
