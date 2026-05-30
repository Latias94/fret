use super::*;

#[test]
fn tree_node_hover_palette_prefers_accent_chrome_over_popover_fill() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("list.active.background".to_string(), "#224466".to_string());
        cfg.colors
            .insert("accent".to_string(), "#335577".to_string());
        cfg.colors
            .insert("accent-foreground".to_string(), "#fefefe".to_string());
        cfg.colors
            .insert("foreground".to_string(), "#f5f6f7".to_string());
        cfg.colors.insert("card".to_string(), "#101418".to_string());
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    let spec = DisclosureSpec::tree_node(Arc::from("Scene"), TreeNodeOptions::default());

    let hovered = resolve_disclosure_palette(
        theme,
        &spec,
        PressableState {
            hovered: true,
            ..Default::default()
        },
    );
    assert_eq!(
        hovered.background,
        Some(Color::from_srgb_hex_rgb(0x33_55_77))
    );
    assert_eq!(hovered.foreground, Color::from_srgb_hex_rgb(0xfe_fe_fe));

    let idle = resolve_disclosure_palette(theme, &spec, PressableState::default());
    assert_eq!(idle.background, None);
    assert_eq!(idle.foreground, Color::from_srgb_hex_rgb(0xf5_f6_f7));
}

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
