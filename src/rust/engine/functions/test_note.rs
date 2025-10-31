use super::*;

#[test]
fn test_parse_note_to_midi() {
    assert_eq!(parse_note_to_midi("C4").unwrap(), 60);
    assert_eq!(parse_note_to_midi("C#4").unwrap(), 61);
    assert_eq!(parse_note_to_midi("D4").unwrap(), 62);
    assert_eq!(parse_note_to_midi("A4").unwrap(), 69);
    assert_eq!(parse_note_to_midi("C5").unwrap(), 72);
    assert_eq!(parse_note_to_midi("Bb4").unwrap(), 70);
}
