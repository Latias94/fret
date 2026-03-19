use super::*;

#[test]
fn declarative_text_sets_semantics_label() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-text",
        |cx| vec![cx.text("Hello declarative")],
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    // Root is a host widget, so text is in a descendant; ensure at least one Text node carries
    // the label payload.
    assert!(
        snap.nodes
            .iter()
            .any(|n| n.role == fret_core::SemanticsRole::Text
                && n.label.as_deref() == Some("Hello declarative")),
        "expected a Text semantics node with label"
    );
}

#[test]
fn declarative_styled_text_sets_semantics_label() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let text: Arc<str> = Arc::from("Hello styled declarative");
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-styled-text",
        |cx| {
            vec![cx.styled_text_props(crate::element::StyledTextProps::new(
                fret_core::AttributedText::new(
                    Arc::clone(&text),
                    Arc::from([fret_core::TextSpan::new(text.len())]),
                ),
            ))]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|n| n.role == fret_core::SemanticsRole::Text
                && n.label.as_deref() == Some(text.as_ref())),
        "expected a StyledText semantics node with label"
    );
}

#[test]
fn selectable_text_emits_inline_link_spans_in_semantics_snapshot() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let text: Arc<str> = "hello link world".into();
    let span = crate::element::SelectableTextInteractiveSpan {
        range: 6..10,
        tag: Arc::<str>::from("https://example.com"),
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-selectable-text-inline-spans",
        |cx| {
            vec![
                cx.selectable_text_props(crate::element::SelectableTextProps {
                    layout: Default::default(),
                    rich: fret_core::AttributedText::new(
                        Arc::clone(&text),
                        Arc::from([fret_core::TextSpan {
                            len: text.len(),
                            shaping: Default::default(),
                            paint: Default::default(),
                        }]),
                    ),
                    style: None,
                    color: None,
                    wrap: fret_core::TextWrap::Word,
                    overflow: fret_core::TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,
                    ink_overflow: Default::default(),
                    interactive_spans: Arc::from([span]),
                }),
            ]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_core::SemanticsRole::Text && n.value.as_deref() == Some(&text))
        .expect("expected a Text semantics node carrying the selectable value");

    assert_eq!(node.inline_spans.len(), 1);
    assert_eq!(node.inline_spans[0].role, fret_core::SemanticsRole::Link);
    assert_eq!(node.inline_spans[0].range_utf8, (6, 10));
    assert_eq!(
        node.inline_spans[0].tag.as_deref(),
        Some("https://example.com")
    );
}

#[test]
fn declarative_text_input_region_answers_platform_text_input_queries_in_utf16() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let region_id: std::cell::RefCell<Option<crate::GlobalElementId>> = Default::default();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "platform-text-input-text-input-region",
        |cx| {
            let mut props = crate::element::TextInputRegionProps::default();
            props.layout.size.width = crate::element::Length::Fill;
            props.layout.size.height = crate::element::Length::Fill;
            props.a11y_label = Some("Editor".into());
            props.a11y_value = Some("a😀b".into());
            // Select just the emoji (UTF-8 bytes [1,5)).
            props.a11y_text_selection = Some((1, 1 + "😀".len() as u32));
            // Mark the same range as a composition span.
            props.a11y_text_composition = Some((1, 1 + "😀".len() as u32));
            props.ime_cursor_area = Some(Rect::new(
                Point::new(Px(10.0), Px(20.0)),
                Size::new(Px(1.0), Px(12.0)),
            ));

            let region = cx.text_input_region(props, |_cx| Vec::<AnyElement>::new());
            *region_id.borrow_mut() = Some(region.id);
            vec![region]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region_id = region_id.borrow().expect("region element id");
    let region =
        crate::elements::node_for_element(&mut app, window, region_id).expect("region node");
    ui.set_focus(Some(region));

    let selected = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::SelectedTextRange,
    );
    assert_eq!(
        selected,
        fret_runtime::PlatformTextInputQueryResult::Range(Some(fret_runtime::Utf16Range::new(
            1, 3
        )))
    );

    let marked = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::MarkedTextRange,
    );
    assert_eq!(
        marked,
        fret_runtime::PlatformTextInputQueryResult::Range(Some(fret_runtime::Utf16Range::new(
            1, 3
        )))
    );

    let text_for = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::TextForRange {
            range: fret_runtime::Utf16Range::new(1, 3),
        },
    );
    assert_eq!(
        text_for,
        fret_runtime::PlatformTextInputQueryResult::Text(Some("😀".to_string()))
    );

    let text_for_reversed = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::TextForRange {
            range: fret_runtime::Utf16Range::new(3, 1),
        },
    );
    assert_eq!(
        text_for_reversed,
        fret_runtime::PlatformTextInputQueryResult::Text(Some("😀".to_string()))
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("window text input snapshot");
    assert!(snapshot.focus_is_text_input);
    assert!(snapshot.is_composing);
    assert_eq!(snapshot.text_len_utf16, 4, "a(1) 😀(2) b(1)");
    assert_eq!(snapshot.selection_utf16, Some((1, 3)));
    assert_eq!(snapshot.marked_utf16, Some((1, 3)));
    assert_eq!(
        snapshot.ime_cursor_area,
        Some(Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(1.0), Px(12.0)),
        ))
    );
}

#[test]
fn pressable_slider_exposes_stepper_actions_when_numeric_metadata_is_present() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-pressable-slider-actions",
        |cx| {
            let mut props = crate::element::PressableProps::default();
            props.enabled = true;
            props.focusable = true;
            props.layout.size.width = Length::Px(Px(200.0));
            props.layout.size.height = Length::Px(Px(24.0));
            props.a11y = crate::element::PressableA11y {
                role: Some(fret_core::SemanticsRole::Slider),
                label: Some(Arc::from("Volume")),
                ..Default::default()
            };

            let a11y = crate::element::SemanticsDecoration::default()
                .orientation(fret_core::SemanticsOrientation::Horizontal)
                .numeric_value(50.0)
                .numeric_range(0.0, 100.0)
                .numeric_step(1.0)
                .numeric_jump(10.0);

            vec![
                cx.pressable(props, |_cx, _state| Vec::new())
                    .attach_semantics(a11y),
            ]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == fret_core::SemanticsRole::Slider && n.label.as_deref() == Some("Volume")
        })
        .expect("expected a Slider semantics node");

    assert!(node.actions.increment, "expected Increment to be exposed");
    assert!(node.actions.decrement, "expected Decrement to be exposed");
    assert!(
        node.actions.set_value,
        "expected SetValue to be gated on for slider with numeric metadata"
    );
    assert!(
        !node.actions.invoke,
        "expected Click to be suppressed for slider"
    );

    assert_eq!(node.extra.numeric.value, Some(50.0));
    assert_eq!(node.extra.numeric.min, Some(0.0));
    assert_eq!(node.extra.numeric.max, Some(100.0));
    assert_eq!(node.extra.numeric.step, Some(1.0));
    assert_eq!(node.extra.numeric.jump, Some(10.0));
}

#[test]
fn pressable_spin_button_exposes_stepper_actions_when_numeric_metadata_is_present() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-pressable-spin-button-actions",
        |cx| {
            let mut props = crate::element::PressableProps::default();
            props.enabled = true;
            props.focusable = true;
            props.layout.size.width = Length::Px(Px(200.0));
            props.layout.size.height = Length::Px(Px(24.0));
            props.a11y = crate::element::PressableA11y {
                role: Some(fret_core::SemanticsRole::SpinButton),
                label: Some(Arc::from("Font size")),
                ..Default::default()
            };

            let a11y = crate::element::SemanticsDecoration::default()
                .numeric_value(12.0)
                .numeric_range(1.0, 72.0)
                .numeric_step(1.0)
                .numeric_jump(10.0);

            vec![
                cx.pressable(props, |_cx, _state| Vec::new())
                    .attach_semantics(a11y),
            ]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == fret_core::SemanticsRole::SpinButton
                && n.label.as_deref() == Some("Font size")
        })
        .expect("expected a SpinButton semantics node");

    assert!(node.actions.increment, "expected Increment to be exposed");
    assert!(node.actions.decrement, "expected Decrement to be exposed");
    assert!(
        node.actions.set_value,
        "expected SetValue to be gated on for spin button with numeric metadata"
    );
    assert!(
        !node.actions.invoke,
        "expected Click to be suppressed for spin button"
    );

    assert_eq!(node.extra.numeric.value, Some(12.0));
    assert_eq!(node.extra.numeric.min, Some(1.0));
    assert_eq!(node.extra.numeric.max, Some(72.0));
    assert_eq!(node.extra.numeric.step, Some(1.0));
    assert_eq!(node.extra.numeric.jump, Some(10.0));
}

#[test]
fn declarative_scrollbar_emits_role_and_scroll_metadata() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
    );
    let mut services = FakeTextService::default();

    let handle = crate::scroll::ScrollHandle::default();
    handle.set_viewport_size(fret_core::Size::new(Px(100.0), Px(100.0)));
    handle.set_content_size(fret_core::Size::new(Px(100.0), Px(220.0)));
    handle.set_offset(fret_core::Point::new(Px(0.0), Px(40.0)));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-scrollbar",
        |cx| {
            vec![cx.scrollbar(crate::element::ScrollbarProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Px(Px(12.0)),
                        height: Length::Px(Px(120.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                axis: crate::element::ScrollbarAxis::Vertical,
                scroll_target: None,
                scroll_handle: handle.clone(),
                style: crate::element::ScrollbarStyle::default(),
            })]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_core::SemanticsRole::ScrollBar)
        .expect("expected a ScrollBar semantics node");

    assert_eq!(
        node.extra.orientation,
        Some(fret_core::SemanticsOrientation::Vertical)
    );
    assert!(node.actions.scroll_by, "expected scroll-by action surface");
    assert_eq!(node.extra.scroll.y, Some(40.0));
    assert_eq!(node.extra.scroll.y_min, Some(0.0));
    assert_eq!(node.extra.scroll.y_max, Some(120.0));
}

#[test]
fn declarative_text_input_region_utf16_queries_are_deterministic_for_mixed_scripts_and_surrogates()
{
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let zwj_cluster = "👩\u{200D}💻";
    let value = format!("a😀ב{zwj_cluster}✌\u{FE0F}e\u{0301}中");
    let zwj_start = value.find(zwj_cluster).expect("ZWJ cluster start");
    let laptop_start = zwj_start + "👩".len() + "\u{200D}".len();

    // Force platform-facing selection/composition to exercise clamping:
    // - anchor points inside the UTF-8 bytes of the first surrogate-pair scalar ("👩")
    // - focus points inside the UTF-8 bytes of the second surrogate-pair scalar ("💻")
    let anchor_utf8 = u32::try_from(zwj_start.saturating_add(1)).unwrap();
    let focus_utf8 = u32::try_from(laptop_start.saturating_add(1)).unwrap();

    fn utf16_len(s: &str) -> u32 {
        u32::try_from(s.encode_utf16().count()).unwrap()
    }

    let woman_u16_start = utf16_len(&value[..zwj_start]);
    let laptop_u16_start = utf16_len(&value[..laptop_start]);
    let laptop_u16_end = utf16_len(&value[..laptop_start + "💻".len()]);

    let region_id: std::cell::RefCell<Option<crate::GlobalElementId>> = Default::default();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "platform-text-input-mixed-scripts-surrogates",
        |cx| {
            let mut props = crate::element::TextInputRegionProps::default();
            props.layout.size.width = crate::element::Length::Fill;
            props.layout.size.height = crate::element::Length::Fill;
            props.a11y_label = Some("Editor".into());
            props.a11y_value = Some(value.clone().into());
            props.a11y_text_selection = Some((anchor_utf8, focus_utf8));
            props.a11y_text_composition = Some((anchor_utf8, focus_utf8));
            props.ime_cursor_area = Some(Rect::new(
                Point::new(Px(10.0), Px(20.0)),
                Size::new(Px(1.0), Px(12.0)),
            ));

            let region = cx.text_input_region(props, |_cx| Vec::<AnyElement>::new());
            *region_id.borrow_mut() = Some(region.id);
            vec![region]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region_id = region_id.borrow().expect("region element id");
    let region =
        crate::elements::node_for_element(&mut app, window, region_id).expect("region node");
    ui.set_focus(Some(region));

    let selected = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::SelectedTextRange,
    );
    assert_eq!(
        selected,
        fret_runtime::PlatformTextInputQueryResult::Range(Some(fret_runtime::Utf16Range::new(
            woman_u16_start,
            laptop_u16_start
        )))
    );

    let marked = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::MarkedTextRange,
    );
    assert_eq!(
        marked,
        fret_runtime::PlatformTextInputQueryResult::Range(Some(fret_runtime::Utf16Range::new(
            woman_u16_start,
            laptop_u16_end
        )))
    );

    // UTF-16 ranges that split inside surrogate pairs must clamp deterministically.
    let emoji_u16_start = utf16_len("a");
    for (label, inside_u16, expect) in [
        ("emoji", emoji_u16_start + 1, "😀"),
        ("woman", woman_u16_start + 1, "👩"),
        ("laptop", laptop_u16_start + 1, "💻"),
    ] {
        let out = ui.platform_text_input_query(
            &mut app,
            &mut services,
            1.0,
            &fret_runtime::PlatformTextInputQuery::TextForRange {
                range: fret_runtime::Utf16Range::new(inside_u16, inside_u16),
            },
        );
        assert_eq!(
            out,
            fret_runtime::PlatformTextInputQueryResult::Text(Some(expect.to_string())),
            "{label} range should clamp to the full scalar"
        );
    }

    let cluster_text = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::TextForRange {
            range: fret_runtime::Utf16Range::new(woman_u16_start, laptop_u16_end),
        },
    );
    assert_eq!(
        cluster_text,
        fret_runtime::PlatformTextInputQueryResult::Text(Some(zwj_cluster.to_string()))
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("window text input snapshot");
    assert!(snapshot.focus_is_text_input);
    assert!(snapshot.is_composing);
    assert_eq!(snapshot.text_len_utf16, utf16_len(value.as_str()));
    assert_eq!(
        snapshot.selection_utf16,
        Some((woman_u16_start, laptop_u16_start))
    );
    assert_eq!(
        snapshot.marked_utf16,
        Some((woman_u16_start, laptop_u16_end))
    );
}

#[test]
fn declarative_text_input_region_platform_query_hook_can_answer_bounds_and_index() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let region_id: std::cell::RefCell<Option<crate::GlobalElementId>> = Default::default();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "platform-text-input-query-hook",
        |cx| {
            let mut props = crate::element::TextInputRegionProps::default();
            props.layout.size.width = crate::element::Length::Fill;
            props.layout.size.height = crate::element::Length::Fill;
            props.a11y_label = Some("Editor".into());
            props.a11y_value = Some("abc".into());
            props.a11y_text_selection = Some((0, 0));

            let region = cx.text_input_region(props, |cx| {
                cx.text_input_region_on_platform_text_input_query(std::sync::Arc::new(
                    move |_host, _action_cx, _services, _bounds, _scale_factor, _props, query| {
                        match query {
                            fret_runtime::PlatformTextInputQuery::BoundsForRange { .. } => {
                                Some(fret_runtime::PlatformTextInputQueryResult::Bounds(Some(
                                    Rect::new(
                                        Point::new(Px(10.0), Px(20.0)),
                                        Size::new(Px(1.0), Px(12.0)),
                                    ),
                                )))
                            }
                            fret_runtime::PlatformTextInputQuery::CharacterIndexForPoint {
                                point,
                            } => {
                                let idx = if point.x.0 < 0.0 { 0 } else { 1 };
                                Some(fret_runtime::PlatformTextInputQueryResult::Index(Some(idx)))
                            }
                            _ => None,
                        }
                    },
                ));
                Vec::<AnyElement>::new()
            });
            *region_id.borrow_mut() = Some(region.id);
            vec![region]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region_id = region_id.borrow().expect("region element id");
    let region =
        crate::elements::node_for_element(&mut app, window, region_id).expect("region node");
    ui.set_focus(Some(region));

    let bounds_for = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::BoundsForRange {
            range: fret_runtime::Utf16Range::new(0, 0),
        },
    );
    assert_eq!(
        bounds_for,
        fret_runtime::PlatformTextInputQueryResult::Bounds(Some(Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(1.0), Px(12.0)),
        )))
    );

    let idx = ui.platform_text_input_query(
        &mut app,
        &mut services,
        1.0,
        &fret_runtime::PlatformTextInputQuery::CharacterIndexForPoint {
            point: Point::new(Px(5.0), Px(5.0)),
        },
    );
    assert_eq!(
        idx,
        fret_runtime::PlatformTextInputQueryResult::Index(Some(1))
    );
}

#[test]
fn declarative_text_input_region_platform_replace_hook_can_handle_replace_text() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let region_id: std::cell::RefCell<Option<crate::GlobalElementId>> = Default::default();
    let called = std::rc::Rc::new(std::cell::RefCell::new(Vec::<(
        fret_runtime::Utf16Range,
        String,
    )>::new()));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "platform-text-input-replace-hook",
        |cx| {
            let mut props = crate::element::TextInputRegionProps::default();
            props.layout.size.width = crate::element::Length::Fill;
            props.layout.size.height = crate::element::Length::Fill;
            props.a11y_label = Some("Editor".into());
            props.a11y_value = Some("abc".into());
            props.a11y_text_selection = Some((0, 0));

            let called = called.clone();
            let region = cx.text_input_region(props, |cx| {
                cx.text_input_region_on_platform_text_input_replace_text_in_range_utf16(
                    std::sync::Arc::new(
                        move |_host,
                              _action_cx,
                              _services,
                              _bounds,
                              _scale_factor,
                              _props,
                              range,
                              text| {
                            called.borrow_mut().push((range, text.to_string()));
                            true
                        },
                    ),
                );
                Vec::<AnyElement>::new()
            });
            *region_id.borrow_mut() = Some(region.id);
            vec![region]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region_id = region_id.borrow().expect("region element id");
    let region =
        crate::elements::node_for_element(&mut app, window, region_id).expect("region node");
    ui.set_focus(Some(region));

    assert!(ui.platform_text_input_replace_text_in_range_utf16(
        &mut app,
        &mut services,
        1.0,
        fret_runtime::Utf16Range::new(0, 0),
        "X"
    ));
    assert_eq!(
        called.borrow().as_slice(),
        &[(fret_runtime::Utf16Range::new(0, 0), "X".to_string())]
    );
}

#[test]
fn declarative_text_input_region_platform_replace_hook_can_handle_replace_and_mark() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let region_id: std::cell::RefCell<Option<crate::GlobalElementId>> = Default::default();
    let called = std::rc::Rc::new(std::cell::RefCell::new(Vec::<(
        fret_runtime::Utf16Range,
        String,
        Option<fret_runtime::Utf16Range>,
        Option<fret_runtime::Utf16Range>,
    )>::new()));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "platform-text-input-replace-and-mark-hook",
        |cx| {
            let mut props = crate::element::TextInputRegionProps::default();
            props.layout.size.width = crate::element::Length::Fill;
            props.layout.size.height = crate::element::Length::Fill;
            props.a11y_label = Some("Editor".into());
            props.a11y_value = Some("abc".into());
            props.a11y_text_selection = Some((0, 0));

            let called = called.clone();
            let region = cx.text_input_region(props, |cx| {
                cx.text_input_region_on_platform_text_input_replace_and_mark_text_in_range_utf16(
                    std::sync::Arc::new(
                        move |_host,
                              _action_cx,
                              _services,
                              _bounds,
                              _scale_factor,
                              _props,
                              range,
                              text,
                              marked,
                              selected| {
                            called
                                .borrow_mut()
                                .push((range, text.to_string(), marked, selected));
                            true
                        },
                    ),
                );
                Vec::<AnyElement>::new()
            });
            *region_id.borrow_mut() = Some(region.id);
            vec![region]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region_id = region_id.borrow().expect("region element id");
    let region =
        crate::elements::node_for_element(&mut app, window, region_id).expect("region node");
    ui.set_focus(Some(region));

    assert!(ui.platform_text_input_replace_and_mark_text_in_range_utf16(
        &mut app,
        &mut services,
        1.0,
        fret_runtime::Utf16Range::new(0, 0),
        "X",
        Some(fret_runtime::Utf16Range::new(0, 1)),
        None
    ));
    assert_eq!(
        called.borrow().as_slice(),
        &[(
            fret_runtime::Utf16Range::new(0, 0),
            "X".to_string(),
            Some(fret_runtime::Utf16Range::new(0, 1)),
            None
        )]
    );
}

#[test]
fn declarative_attach_semantics_overrides_role_label_and_sets_test_id() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-attach-semantics",
        |cx| {
            vec![
                cx.text("Hello declarative").attach_semantics(
                    crate::element::SemanticsDecoration::default()
                        .test_id("hello")
                        .role(fret_core::SemanticsRole::Button)
                        .label("Stamped label"),
                ),
            ]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes.iter().any(|n| {
            n.test_id.as_deref() == Some("hello")
                && n.role == fret_core::SemanticsRole::Button
                && n.label.as_deref() == Some("Stamped label")
        }),
        "expected a semantics node with attached test_id/role/label overrides"
    );
}

#[test]
fn declarative_attach_semantics_can_override_state_and_relations() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-attach-semantics-v2",
        |cx| {
            let label = cx.text("Label").test_id("label");
            let labelled_by = label.id.0;

            vec![
                label,
                cx.text("Target").attach_semantics(
                    crate::element::SemanticsDecoration::default()
                        .test_id("target")
                        .role(fret_core::SemanticsRole::Checkbox)
                        .disabled(true)
                        .selected(true)
                        .expanded(true)
                        .checked(Some(true))
                        .labelled_by_element(labelled_by),
                ),
            ]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let label_node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("label"))
        .expect("label semantics node");
    let target_node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("target"))
        .expect("target semantics node");

    assert_eq!(target_node.role, fret_core::SemanticsRole::Checkbox);
    assert!(target_node.flags.disabled);
    assert!(target_node.flags.selected);
    assert!(target_node.flags.expanded);
    assert_eq!(target_node.flags.checked, Some(true));
    assert!(
        target_node.labelled_by.contains(&label_node.id),
        "expected target to be labelled_by the label node"
    );
}

#[test]
fn declarative_text_input_sets_semantics_label() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert("hello".to_string());
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-text-input-label",
        |cx| {
            let mut props = crate::element::TextInputProps::new(model);
            props.a11y_label = Some("Search".into());
            vec![cx.text_input(props)]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|n| n.role == fret_core::SemanticsRole::TextField
                && n.label.as_deref() == Some("Search")),
        "expected a TextField semantics node with label"
    );
}

#[test]
fn declarative_text_input_respects_a11y_role_override_and_expanded() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert("hello".to_string());
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-text-input-role-override",
        |cx| {
            let mut props = crate::element::TextInputProps::new(model);
            props.a11y_label = Some("Combobox".into());
            props.a11y_role = Some(fret_core::SemanticsRole::ComboBox);
            props.expanded = Some(true);
            vec![cx.text_input(props)]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes.iter().any(|n| {
            n.role == fret_core::SemanticsRole::ComboBox
                && n.flags.expanded
                && n.label.as_deref() == Some("Combobox")
                && n.value.as_deref() == Some("hello")
        }),
        "expected a ComboBox semantics node with expanded=true and correct label/value"
    );
}

#[test]
fn declarative_text_input_region_publishes_text_field_semantics_and_ranges_when_focused() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let region_id: std::cell::RefCell<Option<crate::GlobalElementId>> = Default::default();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-text-input-region",
        |cx| {
            let mut props = crate::element::TextInputRegionProps::default();
            props.layout.size.width = crate::element::Length::Fill;
            props.layout.size.height = crate::element::Length::Fill;
            props.a11y_label = Some("Editor".into());
            props.a11y_value = Some("hello".into());
            props.a11y_text_selection = Some((2, 2));
            props.a11y_text_composition = Some((1, 3));

            let region = cx.text_input_region(props, |_cx| Vec::<AnyElement>::new());
            *region_id.borrow_mut() = Some(region.id);
            vec![region]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region_id = region_id.borrow().expect("region element id");
    let region =
        crate::elements::node_for_element(&mut app, window, region_id).expect("region node");
    ui.set_focus(Some(region));

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    snap.validate().expect("semantics snapshot should validate");

    let node = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == fret_core::SemanticsRole::TextField && n.label.as_deref() == Some("Editor")
        })
        .expect("expected a TextField semantics node for the text input region");

    assert_eq!(node.value.as_deref(), Some("hello"));
    assert_eq!(node.text_selection, Some((2, 2)));
    assert_eq!(node.text_composition, Some((1, 3)));

    // When not focused, the ranges are cleared (label/value remain).
    ui.set_focus(None);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == fret_core::SemanticsRole::TextField && n.label.as_deref() == Some("Editor")
        })
        .expect("expected a TextField semantics node for the text input region");
    assert_eq!(node.text_selection, None);
    assert_eq!(node.text_composition, None);
}

#[test]
fn declarative_text_area_updates_model_on_text_input() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-area-text-input",
        |cx| {
            let mut props = crate::element::TextAreaProps::new(model.clone());
            props.min_height = Px(80.0);
            vec![cx.text_area(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let focusable = ui
        .first_focusable_descendant_including_declarative(&mut app, window, root)
        .expect("focusable text area");
    ui.set_focus(Some(focusable));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("hello\nworld".to_string()),
    );
    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some("hello\nworld")
    );
}

#[test]
fn declarative_semantics_can_be_focusable_and_receive_key_hooks() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let invoked = app.models_mut().insert(0u32);
    let mut semantics_id: Option<crate::GlobalElementId> = None;

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-focusable-semantics",
        |cx| {
            let invoked = invoked.clone();
            let semantics = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::List,
                    focusable: true,
                    ..Default::default()
                },
                |cx, id| {
                    semantics_id = Some(id);
                    let invoked = invoked.clone();
                    cx.key_on_key_down_for(
                        id,
                        Arc::new(move |host, _cx, down| {
                            if down.repeat || down.key != fret_core::KeyCode::ArrowDown {
                                return false;
                            }
                            let _ = host.models_mut().update(&invoked, |v: &mut u32| *v += 1);
                            true
                        }),
                    );
                    vec![cx.container(
                        crate::element::ContainerProps {
                            layout: {
                                let mut layout = crate::element::LayoutStyle::default();
                                layout.size.width = crate::element::Length::Fill;
                                layout.size.height = crate::element::Length::Fill;
                                layout
                            },
                            ..Default::default()
                        },
                        |cx| vec![cx.text("List container")],
                    )]
                },
            );
            vec![semantics]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let semantics_id = semantics_id.expect("semantics element id");
    let semantics_node =
        crate::elements::node_for_element(&mut app, window, semantics_id).expect("semantics node");
    ui.set_focus(Some(semantics_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(app.models().get_copied(&invoked).unwrap_or_default(), 1);
}

#[test]
fn declarative_pressable_focusable_controls_focus_traversal() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let mut first_id: Option<crate::GlobalElementId> = None;
    let mut second_id: Option<crate::GlobalElementId> = None;

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-pressable-focusable",
        |cx| {
            let mut props = crate::element::PressableProps::default();
            props.layout.size.width = Length::Px(Px(80.0));
            props.layout.size.height = Length::Px(Px(32.0));
            props.focusable = false;

            let first = cx.pressable_with_id(props, |cx, _st, id| {
                first_id = Some(id);
                vec![cx.text("first")]
            });

            let mut props2 = crate::element::PressableProps::default();
            props2.layout.size.width = Length::Px(Px(80.0));
            props2.layout.size.height = Length::Px(Px(32.0));
            props2.focusable = true;

            let second = cx.pressable_with_id(props2, |cx, _st, id| {
                second_id = Some(id);
                vec![cx.text("second")]
            });

            vec![cx.row(crate::element::RowProps::default(), move |_cx| {
                vec![first, second]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let first_id = first_id.expect("first element id");
    let second_id = second_id.expect("second element id");

    let first_node =
        crate::elements::node_for_element(&mut app, window, first_id).expect("first node");
    let second_node =
        crate::elements::node_for_element(&mut app, window, second_id).expect("second node");

    let focusable = ui
        .first_focusable_descendant_including_declarative(&mut app, window, root)
        .expect("focusable pressable");
    assert_eq!(focusable, second_node);
    assert_ne!(focusable, first_node);
}

#[test]
fn declarative_semantics_focusable_controls_focus_traversal() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let mut first_id: Option<crate::GlobalElementId> = None;
    let mut second_id: Option<crate::GlobalElementId> = None;

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-semantics-focusable",
        |cx| {
            let mut props = crate::element::SemanticsProps::default();
            props.role = fret_core::SemanticsRole::List;
            props.layout.size.width = Length::Px(Px(80.0));
            props.layout.size.height = Length::Px(Px(32.0));
            props.focusable = false;

            let first = cx.semantics_with_id(props, |cx, id| {
                first_id = Some(id);
                vec![cx.text("first")]
            });

            let mut props2 = crate::element::SemanticsProps::default();
            props2.role = fret_core::SemanticsRole::List;
            props2.layout.size.width = Length::Px(Px(80.0));
            props2.layout.size.height = Length::Px(Px(32.0));
            props2.focusable = true;

            let second = cx.semantics_with_id(props2, |cx, id| {
                second_id = Some(id);
                vec![cx.text("second")]
            });

            vec![cx.row(crate::element::RowProps::default(), move |_cx| {
                vec![first, second]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let first_id = first_id.expect("first element id");
    let second_id = second_id.expect("second element id");

    let first_node =
        crate::elements::node_for_element(&mut app, window, first_id).expect("first node");
    let second_node =
        crate::elements::node_for_element(&mut app, window, second_id).expect("second node");

    let focusable = ui
        .first_focusable_descendant_including_declarative(&mut app, window, root)
        .expect("focusable semantics");
    assert_eq!(focusable, second_node);
    assert_ne!(focusable, first_node);
}
