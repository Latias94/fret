use super::*;

#[test]
fn tree_row_label_uses_shared_list_row_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        let spec = DisclosureSpec::tree_node(
            Arc::from("Very long tree node"),
            TreeNodeOptions {
                leaf: true,
                ..Default::default()
            },
        );
        header_row(
            cx,
            &spec,
            spec.label.clone(),
            false,
            PressableState::default(),
        )
    });
    let expected_palette = resolve_disclosure_palette(
        Theme::global(&app),
        &DisclosureSpec::tree_node(
            Arc::from("Very long tree node"),
            TreeNodeOptions {
                leaf: true,
                ..Default::default()
            },
        ),
        PressableState::default(),
    );

    let text = first_text(&el, "Very long tree node").expect("expected tree row label text");
    let ElementKind::Text(props) = &text.kind else {
        panic!("expected tree row label to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Ellipsis);
    assert!(text.inherited_text_style.is_some());
    assert_eq!(text.inherited_foreground, Some(expected_palette.foreground));
}

#[test]
fn disclosure_indicator_uses_shared_chrome_glyph_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        let spec = DisclosureSpec::tree_node(
            Arc::from("Expandable tree node"),
            TreeNodeOptions::default(),
        );
        header_row(
            cx,
            &spec,
            spec.label.clone(),
            false,
            PressableState::default(),
        )
    });
    let expected_palette = resolve_disclosure_palette(
        Theme::global(&app),
        &DisclosureSpec::tree_node(
            Arc::from("Expandable tree node"),
            TreeNodeOptions::default(),
        ),
        PressableState::default(),
    );

    let text = first_text(&el, ">").expect("expected disclosure indicator text");
    let ElementKind::Text(props) = &text.kind else {
        panic!("expected disclosure indicator to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Clip);
    assert!(text.inherited_text_style.is_some());
    assert_eq!(text.inherited_foreground, Some(expected_palette.foreground));
}
