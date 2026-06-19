use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_ui::element::ElementKind;
use fret_ui::elements::with_element_cx;

use super::{NumericInput, NumericInputErrorDisplay};
use crate::primitives::NumericPresentation;

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

#[test]
fn numeric_input_defaults_to_trailing_icon_error_display() {
    let model = fret_app::App::new().models_mut().insert(1.25f64);
    let input = NumericInput::new(
        model,
        Arc::new(|v| Arc::from(format!("{v:.2}"))),
        Arc::new(|text| text.parse::<f64>().ok()),
    );

    assert!(matches!(
        input.options.error_display,
        NumericInputErrorDisplay::TrailingIcon
    ));
}

#[test]
fn numeric_input_without_inline_error_returns_joined_field_root() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let model = app.models_mut().insert(1.25f64);

    let element = with_element_cx(
        &mut app,
        window,
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(220.0), Px(60.0))),
        "numeric-input-direct-field",
        |cx| {
            NumericInput::new(
                model,
                Arc::new(|v| Arc::from(format!("{v:.2}"))),
                Arc::new(|text| text.parse::<f64>().ok()),
            )
            .into_element(cx)
        },
    );

    assert!(
        matches!(element.kind, ElementKind::HoverRegion(_)),
        "numeric input should return the joined field root directly when inline error is absent"
    );
    assert_eq!(element.children.len(), 1);
}
