use std::sync::Arc;

use super::editor_input_value_text;
use crate::primitives::EditorDensity;
use fret_app::App;
use fret_core::{AppWindowId, Color, Point, Px, Rect, Size, TextOverflow, TextWrap};
use fret_ui::element::{ElementKind, Length};
use fret_ui::elements;

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
}

#[test]
fn editor_input_value_text_is_single_line_and_shrinkable() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let color = Color::from_srgb_hex_rgb(0xDD_EE_FF);

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        editor_input_value_text(
            cx,
            EditorDensity::default(),
            Px(12.0),
            Arc::from("123456789.123456789"),
            color,
            Length::Fill,
        )
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected editor_input_value_text(...) to build a Text element");
    };

    assert_eq!(props.color, Some(color));
    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.size.height, Length::Fill);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.layout.flex.grow, 1.0);
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Ellipsis);
}
