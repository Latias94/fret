use std::sync::Arc;

use super::Slider;
use crate::primitives::NumericPresentation;
use fret_app::App;

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
