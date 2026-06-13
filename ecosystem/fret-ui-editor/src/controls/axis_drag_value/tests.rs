use std::sync::Arc;

use super::AxisDragValue;
use crate::primitives::NumericPresentation;
use fret_app::App;
use fret_core::{AppWindowId, Color, Px, Rect};
use fret_ui::element::{AnyElement, ElementKind, Length};

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

#[test]
fn axis_drag_value_uses_stable_session_shell_for_scrub_and_typing_branches() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let model = app.models_mut().insert(1.25f64);
    let element = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        Rect::default(),
        "axis-drag-value-session-shell",
        |cx| {
            AxisDragValue::new(
                Arc::from("X"),
                Color::from_srgb_hex_rgb(0xf2_59_59),
                model,
                Arc::new(|v| Arc::from(format!("{v:.2}"))),
                Arc::new(|text| text.parse::<f64>().ok()),
            )
            .into_element(cx)
        },
    );

    let ElementKind::Stack(shell) = &element.kind else {
        panic!("axis drag value should mount scrub/typing branches in a stack shell");
    };
    assert_eq!(shell.layout.size.width, Length::Fill);
    assert_eq!(shell.layout.size.height, Length::Auto);
    assert!(matches!(
        shell.layout.size.min_height,
        Some(Length::Px(Px(h))) if h > 0.0
    ));
    assert_eq!(shell.layout.flex.grow, 1.0);
    assert_eq!(shell.layout.flex.basis, Length::Px(Px(0.0)));

    assert_eq!(element.children.len(), 2);
    assert_branch_is_fill(&element.children[0], "scrub branch");
    assert_branch_is_hidden(&element.children[1], "typing branch");
}

fn assert_branch_is_fill(element: &AnyElement, label: &str) {
    let layout = element_layout(element, label);
    assert_eq!(layout.size.width, Length::Fill, "{label} width");
    assert_eq!(layout.size.height, Length::Fill, "{label} height");
    assert_eq!(
        layout.flex.grow, 0.0,
        "{label} should not reuse caller flex grow inside the session shell"
    );
}

fn assert_branch_is_hidden(element: &AnyElement, label: &str) {
    let layout = element_layout(element, label);
    assert_eq!(layout.size.width, Length::Px(Px(0.0)), "{label} width");
    assert_eq!(layout.size.height, Length::Px(Px(0.0)), "{label} height");
    assert_eq!(
        layout.position,
        fret_ui::element::PositionStyle::Absolute,
        "{label} position"
    );
}

fn element_layout<'a>(element: &'a AnyElement, label: &str) -> &'a fret_ui::element::LayoutStyle {
    match &element.kind {
        ElementKind::Pressable(props) => &props.layout,
        ElementKind::Container(props) => &props.layout,
        other => panic!("{label} should expose layout props, got {other:?}"),
    }
}
