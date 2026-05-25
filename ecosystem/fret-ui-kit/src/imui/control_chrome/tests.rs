use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect};
use fret_ui::element::{ElementKind, Length};
use fret_ui::elements;

use super::{control_text, fill_text};

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(320.0), Px(160.0)),
    )
}

#[test]
fn imui_control_text_uses_shared_button_label_role() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let foreground = fret_core::Color::from_srgb_hex_rgb(0x11_22_33);

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        control_text(cx, Arc::from("Apply selected changes"), foreground)
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected control_text(...) to build a Text element");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, fret_core::TextWrap::None);
    assert_eq!(props.overflow, fret_core::TextOverflow::Ellipsis);
    assert!(el.inherited_text_style.is_some());
    assert_eq!(el.inherited_foreground, Some(foreground));
}

#[test]
fn imui_fill_text_is_single_line_and_shrinkable() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let foreground = fret_core::Color::from_srgb_hex_rgb(0x44_55_66);

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        fill_text(
            cx,
            Arc::from("Long boolean/combo label that should not wrap in compact chrome"),
            foreground,
        )
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected fill_text(...) to build a Text element");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.flex.grow, 1.0);
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, fret_core::TextWrap::None);
    assert_eq!(props.overflow, fret_core::TextOverflow::Ellipsis);
    assert!(el.inherited_text_style.is_some());
    assert_eq!(el.inherited_foreground, Some(foreground));
}
