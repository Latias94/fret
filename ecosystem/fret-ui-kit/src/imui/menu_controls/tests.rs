use super::element::menu_item_element_with_pressable_hook_inner;
use super::visual::{menu_item_indicator_text, menu_item_label_text, menu_item_shortcut_text};
use super::*;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size, TextOverflow, TextWrap};
use fret_ui::element::{ElementKind, Length};
use fret_ui::elements;

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
}

#[test]
fn menu_item_label_text_uses_shared_list_row_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        menu_item_label_text(cx, Arc::from("Open very long recent project path"))
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected menu item label to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Ellipsis);
    assert!(el.inherited_text_style.is_some());
}

#[test]
fn menu_item_shortcut_text_uses_shared_control_readout_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        menu_item_shortcut_text(cx, Arc::from("Ctrl+Alt+Shift+P"))
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected menu item shortcut to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Ellipsis);
    assert!(el.inherited_text_style.is_some());
    assert!(el.inherited_foreground.is_some());
}

#[test]
fn menu_item_indicator_text_uses_shared_chrome_glyph_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        menu_item_indicator_text(cx, Arc::from("\u{203A}"))
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected menu item indicator to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Clip);
    assert!(el.inherited_text_style.is_some());
}

#[test]
fn menu_item_root_pressable_owns_visible_row_children() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        let mut response = ResponseExt::default();
        let options = MenuItemOptions {
            test_id: Some(Arc::from("menu.item")),
            ..Default::default()
        };

        menu_item_element_with_pressable_hook_inner(
            cx,
            Arc::from("Open"),
            options,
            SemanticsRole::MenuItem,
            None,
            None,
            noop_menu_item_pressable_hook::<App>,
            &mut response,
        )
    });

    let ElementKind::Pressable(props) = &el.kind else {
        panic!("expected menu item root to be the pressable row");
    };

    assert_eq!(props.a11y.test_id.as_deref(), Some("menu.item"));
    assert_ne!(
        props.layout.position,
        fret_ui::element::PositionStyle::Absolute
    );
    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.size.height, Length::Auto);
    assert_eq!(el.children.len(), 1);
    assert!(matches!(el.children[0].kind, ElementKind::Container(_)));
}
