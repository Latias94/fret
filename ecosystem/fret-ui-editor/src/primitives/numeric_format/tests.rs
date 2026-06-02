use std::sync::Arc;

use super::{
    NumericPresentation, NumericTextAffixes, affixed_number_format, affixed_number_parse,
    degrees_format, degrees_parse, fixed_decimals_format, plain_number_parse,
    suppress_duplicate_chrome_affixes,
};

#[test]
fn fixed_decimals_format_renders_requested_precision() {
    let fmt = fixed_decimals_format::<f64>(3);
    assert_eq!(fmt(0.625).as_ref(), "0.625");
}

#[test]
fn plain_number_parse_trims_whitespace() {
    let parse = plain_number_parse::<f64>();
    assert_eq!(parse(" 1.250 "), Some(1.25));
}

#[test]
fn affixed_number_format_wraps_base_text() {
    let fmt = affixed_number_format::<f64>(
        fixed_decimals_format(1),
        NumericTextAffixes::new(Some(Arc::from("$")), Some(Arc::from("ms"))),
    );
    assert_eq!(fmt(1.5).as_ref(), "$1.5ms");
}

#[test]
fn affixed_number_parse_accepts_plain_and_suffixed_text() {
    let parse = affixed_number_parse::<f64>(plain_number_parse(), NumericTextAffixes::suffix("°"));
    assert_eq!(parse("45"), Some(45.0));
    assert_eq!(parse("45°"), Some(45.0));
    assert_eq!(parse(" 45 ° "), Some(45.0));
}

#[test]
fn degrees_helpers_share_suffix_semantics() {
    let fmt = degrees_format::<f64>(0);
    let parse = degrees_parse::<f64>();
    assert_eq!(fmt(90.0).as_ref(), "90°");
    assert_eq!(parse("90°"), Some(90.0));
}

#[test]
fn suppress_duplicate_chrome_affixes_hides_existing_prefix_and_suffix() {
    let (prefix, suffix) =
        suppress_duplicate_chrome_affixes("$25%", Some(Arc::from("$")), Some(Arc::from("%")));

    assert!(prefix.is_none());
    assert!(suffix.is_none());
}

#[test]
fn suppress_duplicate_chrome_affixes_keeps_missing_prefix_and_suffix() {
    let (prefix, suffix) =
        suppress_duplicate_chrome_affixes("25", Some(Arc::from("$")), Some(Arc::from("%")));

    assert_eq!(prefix.as_deref(), Some("$"));
    assert_eq!(suffix.as_deref(), Some("%"));
}

#[test]
fn numeric_presentation_keeps_chrome_affixes_outside_format_and_parse() {
    let presentation = NumericPresentation::<f64>::fixed_decimals(2)
        .with_chrome_prefix("$")
        .with_chrome_suffix("ms");

    assert_eq!(presentation.format()(1.25).as_ref(), "1.25");
    assert_eq!(presentation.parse()("1.25"), Some(1.25));
    assert_eq!(
        presentation.chrome_affixes(),
        &NumericTextAffixes::new(Some(Arc::from("$")), Some(Arc::from("ms")))
    );
    assert_eq!(presentation.chrome_prefix().map(AsRef::as_ref), Some("$"));
    assert_eq!(presentation.chrome_suffix().map(AsRef::as_ref), Some("ms"));
}

#[test]
fn numeric_presentation_parts_clone_text_and_chrome_layers_together() {
    let presentation = NumericPresentation::<f64>::fixed_decimals(1)
        .with_chrome_prefix("$")
        .with_chrome_suffix("ms");
    let (format, parse, chrome_affixes) = presentation.parts();

    assert_eq!(format(1.5).as_ref(), "1.5");
    assert_eq!(parse("1.5"), Some(1.5));
    assert_eq!(
        chrome_affixes,
        NumericTextAffixes::new(Some(Arc::from("$")), Some(Arc::from("ms")))
    );
}

#[test]
fn numeric_presentation_degrees_keep_unit_in_text_layer() {
    let presentation = NumericPresentation::<f64>::degrees(0);

    assert_eq!(presentation.format()(90.0).as_ref(), "90°");
    assert_eq!(presentation.parse()("90°"), Some(90.0));
    assert_eq!(
        presentation.chrome_affixes(),
        &NumericTextAffixes::default()
    );
    assert!(presentation.chrome_prefix().is_none());
    assert!(presentation.chrome_suffix().is_none());
}
