use std::sync::Arc;

use super::{NumericInput, editor_numeric_input_text_style};
use crate::primitives::EditorDensity;
use crate::primitives::NumericPresentation;
use fret_app::App;
use fret_core::{Px, TextStyle};

#[test]
fn numeric_input_text_style_uses_density_row_height_for_edit_line_box() {
    let style = editor_numeric_input_text_style(
        TextStyle {
            size: Px(12.0),
            line_height: Some(Px(16.0)),
            ..Default::default()
        },
        EditorDensity {
            row_height: Px(24.0),
            ..Default::default()
        },
    );

    assert_eq!(style.line_height, Some(Px(24.0)));
}

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
