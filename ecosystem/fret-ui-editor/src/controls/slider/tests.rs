use std::sync::Arc;

use super::{Slider, compose_affixed_value_text};
use crate::primitives::NumericPresentation;
use crate::primitives::numeric_format::suppress_duplicate_chrome_affixes;
use fret_app::App;

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

#[test]
fn slider_from_presentation_adopts_format_parse_and_chrome_affixes() {
    let mut app = App::new();
    let model = app.models_mut().insert(0.25f64);
    let presentation = NumericPresentation::<f64>::fixed_decimals(1)
        .with_chrome_prefix("$")
        .with_chrome_suffix("ms");

    let slider = Slider::from_presentation(model, 0.0, 1.0, presentation);

    assert_eq!((slider.format)(0.25).as_ref(), "0.2");
    assert_eq!((slider.parse)("0.2"), Some(0.2));
    assert_eq!(slider.options.prefix, Some(Arc::from("$")));
    assert_eq!(slider.options.suffix, Some(Arc::from("ms")));
}
