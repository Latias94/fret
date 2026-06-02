use std::sync::Arc;

use super::AxisDragValue;
use crate::primitives::NumericPresentation;
use fret_app::App;
use fret_core::Color;

#[test]
fn axis_drag_value_from_presentation_adopts_format_parse_and_chrome_affixes() {
    let mut app = App::new();
    let model = app.models_mut().insert(1.25f64);
    let presentation = NumericPresentation::<f64>::fixed_decimals(2)
        .with_chrome_prefix("$")
        .with_chrome_suffix("ms");

    let drag_value = AxisDragValue::from_presentation(
        Arc::from("X"),
        Color::from_srgb_hex_rgb(0xf2_59_59),
        model,
        presentation,
    );

    assert_eq!((drag_value.format)(1.25).as_ref(), "1.25");
    assert_eq!((drag_value.parse)("1.25"), Some(1.25));
    assert_eq!(drag_value.options.prefix, Some(Arc::from("$")));
    assert_eq!(drag_value.options.suffix, Some(Arc::from("ms")));
}
