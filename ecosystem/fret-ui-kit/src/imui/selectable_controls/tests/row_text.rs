use super::*;

#[test]
fn selectable_row_label_uses_shared_list_row_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        selectable_row_element(
            cx,
            Arc::from("Long selectable row label"),
            true,
            false,
            false,
            PressableState::default(),
        )
    });
    let expected_palette =
        resolve_selectable_palette(Theme::global(&app), true, false, false, false);

    let text = first_text(&el).expect("expected selectable row text");
    let ElementKind::Text(props) = &text.kind else {
        panic!("expected selectable row label to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Ellipsis);
    assert!(text.inherited_text_style.is_some());
    assert_eq!(text.inherited_foreground, Some(expected_palette.fg));
}
