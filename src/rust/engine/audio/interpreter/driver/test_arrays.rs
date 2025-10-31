use super::*;

#[test]
fn test_array_index_returns_value_from_map_elements() {
    let mut interp = AudioInterpreter::new(44100);

    // Build array where each element is a Map { index: <n>, value: <note> }
    let mut arr: Vec<Value> = Vec::new();
    let notes = vec!["C4", "D4", "E4", "F4", "G4"];
    for (i, &n) in notes.iter().enumerate() {
        let mut m = std::collections::HashMap::new();
        m.insert("index".to_string(), Value::Number(i as f32));
        m.insert("value".to_string(), Value::String(n.to_string()));
        arr.push(Value::Map(m));
    }

    interp
        .variables
        .insert("notesTest".to_string(), Value::Array(arr));

    // Access the 3rd index (0-based) -> should return "F4"
    let resolved = interp
        .resolve_value(&Value::Identifier("notesTest[3]".to_string()))
        .unwrap();
    match resolved {
        Value::String(s) => assert_eq!(s, "F4"),
        other => panic!("expected String(F4), got {:?}", other),
    }
}

#[test]
fn test_array_index_map_element_property_access() {
    let mut interp = AudioInterpreter::new(44100);

    let mut elem0 = std::collections::HashMap::new();
    elem0.insert("index".to_string(), Value::Number(0.0));
    elem0.insert("value".to_string(), Value::String("C4".to_string()));

    let mut elem1 = std::collections::HashMap::new();
    elem1.insert("index".to_string(), Value::Number(1.0));
    elem1.insert("value".to_string(), Value::String("D4".to_string()));

    let mut synth_map = std::collections::HashMap::new();
    synth_map.insert("volume".to_string(), Value::Number(0.75));
    let mut elem2 = std::collections::HashMap::new();
    elem2.insert("index".to_string(), Value::Number(2.0));
    elem2.insert("value".to_string(), Value::Map(synth_map));

    let arr = Value::Array(vec![
        Value::Map(elem0),
        Value::Map(elem1),
        Value::Map(elem2),
    ]);

    let mut obj = std::collections::HashMap::new();
    obj.insert("myArray".to_string(), arr);

    interp
        .variables
        .insert("myObject".to_string(), Value::Map(obj));

    // Access property myObject.myArray[2].volume
    let resolved = interp
        .resolve_value(&Value::Identifier("myObject.myArray[2].volume".to_string()))
        .unwrap();
    match resolved {
        Value::Number(n) => assert!((n - 0.75).abs() < 0.0001),
        other => panic!("expected Number(0.75), got {:?}", other),
    }
}

#[test]
fn test_array_old_form_identifiers_still_works() {
    let mut interp = AudioInterpreter::new(44100);

    let arr = Value::Array(vec![
        Value::Identifier("C4".to_string()),
        Value::Identifier("D4".to_string()),
        Value::Identifier("E4".to_string()),
    ]);

    interp.variables.insert("plainNotes".to_string(), arr);

    // Access plainNotes[1] -> should resolve to string "D4"
    let resolved = interp
        .resolve_value(&Value::Identifier("plainNotes[1]".to_string()))
        .unwrap();
    match resolved {
        Value::String(s) => assert_eq!(s, "D4"),
        other => panic!("expected String(D4), got {:?}", other),
    }
}
