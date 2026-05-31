use std::sync::Arc;

use super::{slider_typing_parse, slider_typing_validate};
use crate::controls::numeric_input::{NumericParseFn, NumericValidateFn};

#[test]
fn slider_typing_parse_quantizes_and_clamps_parsed_values() {
    let parse: NumericParseFn<f64> = Arc::new(|s| s.trim().parse::<f64>().ok());
    let parse = slider_typing_parse(parse, 0.0, 10.0, true, Some(0.5));

    assert_eq!(parse("4.76"), Some(5.0));
    assert_eq!(parse("12.0"), Some(10.0));
    assert_eq!(parse("not-a-number"), None);
}

#[test]
fn slider_typing_validate_adds_range_check_only_when_unclamped() {
    assert!(slider_typing_validate::<f64>(None, 0.0, 10.0, true).is_none());

    let validate = slider_typing_validate::<f64>(None, 0.0, 10.0, false).unwrap();
    assert_eq!(validate(12.0).as_deref(), Some("Out of range"));
    assert_eq!(validate(5.0), None);
}

#[test]
fn slider_typing_validate_delegates_custom_validator_inside_range() {
    let custom: NumericValidateFn<f64> = Arc::new(|v| {
        if (v - 5.0).abs() < f64::EPSILON {
            Some(Arc::from("No five"))
        } else {
            None
        }
    });
    let validate = slider_typing_validate(Some(custom), 0.0, 10.0, false).unwrap();

    assert_eq!(validate(5.0).as_deref(), Some("No five"));
    assert_eq!(validate(12.0).as_deref(), Some("Out of range"));
}
