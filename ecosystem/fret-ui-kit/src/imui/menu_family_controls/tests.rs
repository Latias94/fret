use super::*;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size, TextOverflow, TextWrap};
use fret_ui::element::{ElementKind, PressableState};
use fret_ui::elements;

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
}

#[test]
fn menu_trigger_visual_uses_button_label_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        visual::menu_trigger_visual(
            cx,
            Arc::from("Very long menu label"),
            false,
            true,
            PressableState::default(),
        )
    });

    let text = el
        .children
        .first()
        .expect("expected menu trigger text child");
    let ElementKind::Text(props) = &text.kind else {
        panic!("expected menu trigger label to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Ellipsis);
    assert!(text.inherited_text_style.is_some());
    assert!(text.inherited_foreground.is_some());
}
