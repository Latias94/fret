use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size, TextOverflow, TextWrap};
use fret_ui::element::{ElementKind, Length};
use fret_ui::elements;

use super::floating_window_close_glyph_text;

fn test_bounds() -> Rect {
    Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(48.0)))
}

#[test]
fn floating_window_close_glyph_uses_shared_chrome_glyph_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        floating_window_close_glyph_text(cx)
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected floating window close glyph to be text");
    };

    assert_eq!(props.text.as_ref(), "\u{00D7}");
    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Clip);
    assert!(el.inherited_text_style.is_some());
}
