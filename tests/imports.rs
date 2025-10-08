use std::fs;

use devalang_wasm::language::preprocessor::loader::load_module_exports;
use devalang_wasm::language::syntax::parser::driver::SimpleParser;

#[test]
fn test_imports_load_groups_patterns_vars() {
    // Prepare temporary files in target dir
    let tmp_dir = std::env::temp_dir().join("devalang_test_imports");
    let _ = fs::remove_dir_all(&tmp_dir);
    let _ = fs::create_dir_all(&tmp_dir);

    let module_path = tmp_dir.join("mod.deva");
    let content = r#"
let testVar = 42

pattern myPat with target "pattern" = "C4 E4 G4"

group myGroup:
    print "in group"

@export { testVar, myPat, myGroup }
"#;
    fs::write(&module_path, content).expect("write module");

    // Parse and load
    let exports = load_module_exports(&module_path).expect("load exports");

    assert!(exports.variables.contains_key("testVar"));
    assert!(exports.patterns.contains_key("myPat"));
    assert!(exports.groups.contains_key("myGroup"));
}
