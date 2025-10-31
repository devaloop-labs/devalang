use super::*;

#[test]
fn test_linear_curve() {
    assert_eq!(evaluate_curve(CurveType::Linear, 0.0), 0.0);
    assert_eq!(evaluate_curve(CurveType::Linear, 0.5), 0.5);
    assert_eq!(evaluate_curve(CurveType::Linear, 1.0), 1.0);
}

#[test]
fn test_ease_in_curve() {
    let v = evaluate_curve(CurveType::EaseIn, 0.5);
    assert!(v > 0.0 && v < 0.5);
}

#[test]
fn test_parse_curve() {
    let curve = parse_curve("$curve.linear");
    assert!(matches!(curve, Some(CurveType::Linear)));

    let curve = parse_curve("$curve.swing(0.5)");
    assert!(matches!(curve, Some(CurveType::Swing(_))));

    let curve = parse_curve("$ease.bezier(0.25, 0.1, 0.25, 1.0)");
    assert!(matches!(curve, Some(CurveType::Bezier(_, _, _, _))));
}

#[test]
fn test_swing_curve() {
    let v = evaluate_curve(CurveType::Swing(0.5), 0.5);
    assert!(v >= 0.0 && v <= 1.0);
}
