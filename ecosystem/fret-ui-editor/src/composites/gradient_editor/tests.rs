use super::stops_group::gradient_editor_empty_state_text;
use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size, TextOverflow, TextWrap};
use fret_ui::element::{ElementKind, Length};
use fret_ui::elements;

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    )
}

#[test]
fn gradient_editor_empty_state_text_is_single_line_and_shrinkable() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        gradient_editor_empty_state_text(cx, "No stops")
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected gradient editor empty state to be text");
    };

    assert_eq!(props.text.as_ref(), "No stops");
    assert!(props.style.is_some());
    assert!(props.color.is_some());
    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Ellipsis);
}
