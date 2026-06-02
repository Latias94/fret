use std::sync::Arc;

use super::NumericInput;
use crate::primitives::NumericPresentation;
use fret_app::App;

#[test]
fn numeric_input_from_presentation_adopts_format_parse_and_chrome_affixes() {
    let mut app = App::new();
    let model = app.models_mut().insert(1.25f64);
    let presentation = NumericPresentation::<f64>::fixed_decimals(2)
        .with_chrome_prefix("$")
        .with_chrome_suffix("ms");

    let input = NumericInput::from_presentation(model, presentation);

    assert_eq!((input.format)(1.25).as_ref(), "1.25");
    assert_eq!((input.parse)("1.25"), Some(1.25));
    assert_eq!(input.options.prefix, Some(Arc::from("$")));
    assert_eq!(input.options.suffix, Some(Arc::from("ms")));
}
