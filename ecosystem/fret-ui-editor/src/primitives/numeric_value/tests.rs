use super::{NumericValueConstraints, constrain_numeric_value};

#[test]
fn numeric_constraints_swap_inverted_bounds() {
    let constraints = NumericValueConstraints {
        min: Some(10.0),
        max: Some(2.0),
        clamp: true,
        step: None,
    }
    .normalized();

    assert_eq!(constraints.min, Some(2.0));
    assert_eq!(constraints.max, Some(10.0));
}

#[test]
fn numeric_constraints_quantize_from_min_origin_then_clamp() {
    let constraints = NumericValueConstraints {
        min: Some(0.0),
        max: Some(1.0),
        clamp: true,
        step: Some(0.125),
    };

    assert!((constraints.apply_f64(0.61) - 0.625).abs() < 1e-9);
    assert!((constraints.apply_f64(1.24) - 1.0).abs() < 1e-9);
}

#[test]
fn numeric_constraints_quantize_without_range_uses_zero_origin() {
    let constraints = NumericValueConstraints {
        min: None,
        max: None,
        clamp: false,
        step: Some(0.5),
    };

    assert!((constraints.apply_f64(1.24) - 1.0).abs() < 1e-9);
    assert_eq!(constrain_numeric_value(constraints, 3_i32), 3);
}
