use super::*;

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
