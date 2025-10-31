use super::*;

#[test]
fn test_parse_chord_notation() {
    let cmaj7 = parse_chord_notation("Cmaj7").unwrap();
    assert_eq!(cmaj7, vec!["C4", "E4", "G4", "B4"]);

    let dmin = parse_chord_notation("Dmin").unwrap();
    assert_eq!(dmin, vec!["D4", "F4", "A4"]);
}

#[test]
fn test_generate_chord() {
    let c_major = generate_chord("C4", "major").unwrap();
    assert_eq!(c_major, vec!["C4", "E4", "G4"]);

    let d_minor = generate_chord("D4", "minor").unwrap();
    assert_eq!(d_minor, vec!["D4", "F4", "A4"]);
}
