use std::sync::Arc;

use super::{compose_affixed_value_text, default_slider_format, default_slider_parse};
use crate::primitives::numeric_format::suppress_duplicate_chrome_affixes;

#[test]
fn default_slider_format_uses_integer_or_three_decimal_text() {
    let format = default_slider_format::<f64>();

    assert_eq!(format(4.0).as_ref(), "4");
    assert_eq!(format(4.5).as_ref(), "4.500");
    assert_eq!(format(4.1256).as_ref(), "4.126");
}

#[test]
fn default_slider_parse_accepts_trimmed_f64_text() {
    let parse = default_slider_parse::<f64>();

    assert_eq!(parse(" 1.25 "), Some(1.25));
    assert_eq!(parse("not-a-number"), None);
}

#[test]
fn compose_affixed_value_text_keeps_plain_value_when_no_affix() {
    let value = Arc::<str>::from("12.0");
    assert_eq!(compose_affixed_value_text(&value, None, None), value);
}

#[test]
fn compose_affixed_value_text_joins_prefix_and_suffix_without_extra_spacing() {
    let value = Arc::<str>::from("12.0");
    let prefix = Arc::<str>::from("$");
    let suffix = Arc::<str>::from("px");
    assert_eq!(
        compose_affixed_value_text(&value, Some(&prefix), Some(&suffix)).as_ref(),
        "$12.0px"
    );
}

#[test]
fn compose_affixed_value_text_can_skip_duplicate_suffix_chrome() {
    let value = Arc::<str>::from("25%");
    let (_prefix, suffix) =
        suppress_duplicate_chrome_affixes(value.as_ref(), None, Some(Arc::from("%")));

    assert_eq!(
        compose_affixed_value_text(&value, None, suffix.as_ref()).as_ref(),
        "25%"
    );
}
