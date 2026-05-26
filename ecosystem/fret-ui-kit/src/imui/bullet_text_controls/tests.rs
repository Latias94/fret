use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size, TextOverflow, TextWrap};
use fret_ui::Theme;
use fret_ui::element::{AnyElement, ElementKind, Length};
use fret_ui::elements;

use super::bullet_text_element;
use crate::imui::BulletTextOptions;

fn first_text<'a>(root: &'a AnyElement, expected: &str) -> Option<&'a AnyElement> {
    match &root.kind {
        ElementKind::Text(props) if props.text.as_ref() == expected => Some(root),
        _ => root
            .children
            .iter()
            .find_map(|child| first_text(child, expected)),
    }
}

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
}

#[test]
fn bullet_text_uses_shared_compact_paragraph_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        bullet_text_element(
            cx,
            Arc::from("Long bullet body that may wrap inside an editor panel"),
            BulletTextOptions::default(),
        )
    });
    let theme = Theme::global(&app);
    let expected_foreground = theme
        .color_by_key("foreground")
        .unwrap_or_else(|| theme.color_token("foreground"));

    let text = first_text(&el, "Long bullet body that may wrap inside an editor panel")
        .expect("expected bullet label text");
    let ElementKind::Text(props) = &text.kind else {
        panic!("expected bullet label to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.flex.grow, 1.0);
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::Word);
    assert_eq!(props.overflow, TextOverflow::Clip);
    assert!(text.inherited_text_style.is_some());
    assert_eq!(text.inherited_foreground, Some(expected_foreground));
}
