use super::*;

#[test]
fn selectable_text_drag_autoscrolls_scroll_container() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let mut services = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-autoscroll",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(scroll_handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0),
                            ..Default::default()
                        },
                        |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();
                            out.push(cx.selectable_text(attributed_plain("hello selectable text")));
                            for _ in 0..50 {
                                out.push(cx.text("filler"));
                            }
                            out
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let selectable_node = ui.children(column_node)[0];

    let scroll_bounds = ui.debug_node_bounds(scroll_node).expect("scroll bounds");
    let selectable_bounds = ui
        .debug_node_bounds(selectable_node)
        .expect("selectable bounds");

    let inside = Point::new(
        Px(selectable_bounds.origin.x.0 + 5.0),
        Px(selectable_bounds.origin.y.0 + 5.0),
    );
    let below = Point::new(
        Px(scroll_bounds.origin.x.0 + 5.0),
        Px(scroll_bounds.origin.y.0 + scroll_bounds.size.height.0 + 10.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: below,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected selectable drag to auto-scroll, got offset={:?}",
        scroll_handle.offset()
    );
}


#[test]
fn selectable_text_drag_autoscrolls_horizontal_scroll_container() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-autoscroll-x",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::X,
                    scroll_handle: Some(scroll_handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    let mut content_layout = crate::element::LayoutStyle::default();
                    content_layout.size.width = Length::Px(Px(600.0));
                    content_layout.size.height = Length::Fill;

                    vec![cx.container(
                        crate::element::ContainerProps {
                            layout: content_layout,
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.selectable_text_props(crate::element::SelectableTextProps {
                                layout: Default::default(),
                                rich: attributed_plain(
                                    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
                                ),
                                style: None,
                                color: None,
                                wrap: fret_core::TextWrap::None,
                                overflow: fret_core::TextOverflow::Clip,
                            })]
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let scroll_node = ui.children(root)[0];
    let selectable_node = ui.children(scroll_node)[0];

    let scroll_bounds = ui.debug_node_bounds(scroll_node).expect("scroll bounds");
    let selectable_bounds = ui
        .debug_node_bounds(selectable_node)
        .expect("selectable bounds");

    let inside = Point::new(
        Px(selectable_bounds.origin.x.0 + 5.0),
        Px(selectable_bounds.origin.y.0 + 5.0),
    );
    let beyond_right = Point::new(
        Px(scroll_bounds.origin.x.0 + scroll_bounds.size.width.0 + 10.0),
        Px(scroll_bounds.origin.y.0 + 5.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: beyond_right,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scroll_handle.offset().x.0 > 0.01,
        "expected selectable drag to auto-scroll horizontally, got offset={:?}",
        scroll_handle.offset()
    );
}


#[test]
fn selectable_text_double_and_triple_click_select() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let rich = attributed_plain("hello world\nsecond line");

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-double-triple-click",
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let selectable_node = ui.children(root)[0];
    let record =
        crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
            .expect("selectable record");
    let element = record.element;

    let pos = Point::new(Px(5.0), Px(5.0));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let (a, b) = crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::SelectableTextState::default,
        |state| (state.selection_anchor, state.caret),
    );
    assert_eq!((a, b), (0, 5), "double click should select first word");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 3,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let (a, b) = crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::SelectableTextState::default,
        |state| (state.selection_anchor, state.caret),
    );
    assert_eq!(
        (a, b),
        (0, 12),
        "triple click should select first line (including trailing newline)"
    );
}


#[test]
fn selectable_text_double_click_respects_window_text_boundary_mode_under_render_transform() {
    fn selection_for_mode(mode: fret_runtime::TextBoundaryMode) -> (usize, usize) {
        let mut app = TestHost::new();
        app.set_global(fret_runtime::PlatformCapabilities::default());
        app.with_global_mut_untracked(
            fret_runtime::WindowTextBoundaryModeService::default,
            |svc, _app| {
                svc.set_base_mode(AppWindowId::default(), mode);
            },
        );

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(60.0)));
        let mut services = FakeTextService::default();

        let rich = attributed_plain("can't");

        let transform = Transform2D::translation(Point::new(Px(40.0), Px(10.0)));
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "selectable-text-double-click-boundary-mode-transform",
            |cx| vec![cx.render_transform(transform, |cx| vec![cx.selectable_text(rich.clone())])],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let transform_node = ui.children(root)[0];
        let selectable_node = ui.children(transform_node)[0];
        let record =
            crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
                .expect("selectable record");
        let element = record.element;

        let selectable_bounds = ui
            .debug_node_bounds(selectable_node)
            .expect("selectable bounds");
        let pos = Point::new(
            Px(selectable_bounds.origin.x.0 + 40.0 + 5.0),
            Px(selectable_bounds.origin.y.0 + 10.0 + 5.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 2,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        crate::elements::with_element_state(
            &mut app,
            window,
            element,
            crate::element::SelectableTextState::default,
            |state| (state.selection_anchor, state.caret),
        )
    }

    assert_eq!(
        selection_for_mode(fret_runtime::TextBoundaryMode::UnicodeWord),
        (0, 5),
        "UnicodeWord should select the whole word"
    );
    assert_eq!(
        selection_for_mode(fret_runtime::TextBoundaryMode::Identifier),
        (0, 3),
        "Identifier should stop at the apostrophe"
    );
}


#[test]
fn selectable_text_double_click_respects_window_text_boundary_mode_under_scroll_offset() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());
    app.with_global_mut_untracked(
        fret_runtime::WindowTextBoundaryModeService::default,
        |svc, _app| {
            svc.set_base_mode(
                AppWindowId::default(),
                fret_runtime::TextBoundaryMode::Identifier,
            );
        },
    );

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(140.0), Px(50.0)));
    let mut services = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-double-click-boundary-mode-scroll",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(scroll_handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0),
                            ..Default::default()
                        },
                        |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();
                            for _ in 0..40 {
                                let mut row_layout = crate::element::LayoutStyle::default();
                                row_layout.size.height = Length::Px(Px(18.0));
                                out.push(cx.container(
                                    crate::element::ContainerProps {
                                        layout: row_layout,
                                        ..Default::default()
                                    },
                                    |cx| vec![cx.text("filler")],
                                ));
                            }
                            out.push(cx.selectable_text(attributed_plain("can't")));
                            out
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Force the selectable text into view via an imperative scroll offset.
    //
    // Note: scroll is applied via a render transform, so `debug_node_bounds` reports the layout
    // bounds in content space. We must subtract the scroll offset to get a screen-space click
    // position.
    scroll_handle.set_offset(Point::new(Px(0.0), Px(100_000.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let selectable_node = *ui
        .children(column_node)
        .last()
        .expect("expected selectable text as last child");

    let selectable_bounds = ui
        .debug_node_bounds(selectable_node)
        .expect("selectable bounds");
    let scroll_offset = scroll_handle.offset();
    let pos = Point::new(
        Px(selectable_bounds.origin.x.0 + 5.0),
        Px(selectable_bounds.origin.y.0 - scroll_offset.y.0 + 5.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let record =
        crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
            .expect("selectable record");
    let (a, b) = crate::elements::with_element_state(
        &mut app,
        window,
        record.element,
        crate::element::SelectableTextState::default,
        |state| (state.selection_anchor, state.caret),
    );

    assert_eq!(
        (a, b),
        (0, 3),
        "Identifier mode should stop at the apostrophe"
    );
}


#[test]
fn selectable_text_ctrl_arrow_word_navigation_respects_window_text_boundary_mode() {
    fn caret_positions_for_mode(mode: fret_runtime::TextBoundaryMode) -> (usize, usize) {
        let mut app = TestHost::new();
        app.set_global(fret_runtime::PlatformCapabilities::default());
        app.with_global_mut_untracked(
            fret_runtime::WindowTextBoundaryModeService::default,
            |svc, _app| {
                svc.set_base_mode(AppWindowId::default(), mode);
            },
        );

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(60.0)));
        let mut services = FakeTextService::default();

        let rich = attributed_plain("can't");
        let text_len = rich.text.len();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "selectable-text-ctrl-arrow-boundary-mode",
            |cx| vec![cx.selectable_text(rich.clone())],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let selectable_node = ui.children(root)[0];
        ui.set_focus(Some(selectable_node));

        let record =
            crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
                .expect("selectable record");
        let element = record.element;

        crate::elements::with_element_state(
            &mut app,
            window,
            element,
            crate::element::SelectableTextState::default,
            |state| {
                state.selection_anchor = 0;
                state.caret = 0;
            },
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowRight,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                repeat: false,
            },
        );

        let caret_right = crate::elements::with_element_state(
            &mut app,
            window,
            element,
            crate::element::SelectableTextState::default,
            |state| state.caret,
        );

        crate::elements::with_element_state(
            &mut app,
            window,
            element,
            crate::element::SelectableTextState::default,
            |state| {
                state.selection_anchor = text_len;
                state.caret = text_len;
            },
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowLeft,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                repeat: false,
            },
        );

        let caret_left = crate::elements::with_element_state(
            &mut app,
            window,
            element,
            crate::element::SelectableTextState::default,
            |state| state.caret,
        );

        (caret_right, caret_left)
    }

    assert_eq!(
        caret_positions_for_mode(fret_runtime::TextBoundaryMode::UnicodeWord),
        (5, 0),
        "UnicodeWord should treat \"can't\" as a single word"
    );
    assert_eq!(
        caret_positions_for_mode(fret_runtime::TextBoundaryMode::Identifier),
        (3, 4),
        "Identifier should split \"can't\" around the apostrophe"
    );
}


#[test]
fn selectable_text_pointer_down_requests_focus() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let rich = attributed_plain("hello world");
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-pointer-down-focus",
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let selectable_node = ui.children(root)[0];
    let selectable_bounds = ui
        .debug_node_bounds(selectable_node)
        .expect("selectable bounds");
    let pos = Point::new(
        Px(selectable_bounds.origin.x.0 + 5.0),
        Px(selectable_bounds.origin.y.0 + 5.0),
    );

    assert_eq!(ui.focus(), None, "expected no focus before click");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.focus(),
        Some(selectable_node),
        "expected selectable text to request focus on pointer down"
    );
}


#[test]
fn selectable_text_double_click_sets_primary_selection_when_enabled() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::TextInteractionSettings {
        linux_primary_selection: true,
    });
    let mut caps = fret_runtime::PlatformCapabilities::default();
    caps.clipboard.primary_text = true;
    app.set_global(caps);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let rich = attributed_plain("hello world");
    let root_name = "selectable-text-primary-selection-double-click";
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let selectable_node = ui.children(root)[0];
    let selectable_bounds = ui
        .debug_node_bounds(selectable_node)
        .expect("selectable bounds");
    let pos = Point::new(
        Px(selectable_bounds.origin.x.0 + 5.0),
        Px(selectable_bounds.origin.y.0 + 5.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        app.take_effects().iter().any(|e| {
            matches!(
                e,
                fret_runtime::Effect::PrimarySelectionSetText { text }
                if text == "hello"
            )
        }),
        "expected selectable text selection to set primary selection when enabled"
    );
}


#[test]
fn selectable_text_arrow_up_down_uses_preferred_x_across_lines() {
    #[derive(Default)]
    struct LineTextService {
        text: String,
    }

    impl LineTextService {
        fn line_range(&self, line: usize) -> Option<(usize, usize)> {
            if line == 0 && self.text.is_empty() {
                return Some((0, 0));
            }

            let mut start = 0usize;
            let mut line_idx = 0usize;
            for (i, ch) in self.text.char_indices() {
                if ch != '\n' {
                    continue;
                }
                if line_idx == line {
                    return Some((start, i));
                }
                start = i + 1;
                line_idx += 1;
            }

            if line_idx == line {
                return Some((start, self.text.len()));
            }
            None
        }

        fn line_count(&self) -> usize {
            if self.text.is_empty() {
                return 1;
            }
            self.text.chars().filter(|c| *c == '\n').count() + 1
        }

        fn index_to_line_col(&self, index: usize) -> (usize, usize) {
            let index = index.min(self.text.len());
            let mut line = 0usize;
            let mut col = 0usize;
            for (i, ch) in self.text.char_indices() {
                if i >= index {
                    break;
                }
                if ch == '\n' {
                    line += 1;
                    col = 0;
                } else {
                    col += 1;
                }
            }
            (line, col)
        }
    }

    impl fret_core::TextService for LineTextService {
        fn prepare(
            &mut self,
            input: &fret_core::TextInput,
            _constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            self.text = input.text().to_string();

            let line_h = Px(10.0);
            let lines = self.line_count().max(1) as f32;
            (
                fret_core::TextBlobId::default(),
                fret_core::TextMetrics {
                    size: Size::new(Px(200.0), Px(line_h.0 * lines)),
                    baseline: Px(8.0),
                },
            )
        }

        fn caret_rect(
            &mut self,
            _blob: fret_core::TextBlobId,
            index: usize,
            _affinity: fret_core::CaretAffinity,
        ) -> Rect {
            let line_h = Px(10.0);
            let char_w = Px(10.0);

            let (line, col) = self.index_to_line_col(index);
            Rect::new(
                Point::new(Px(char_w.0 * (col as f32)), Px(line_h.0 * (line as f32))),
                Size::new(Px(1.0), line_h),
            )
        }

        fn hit_test_point(
            &mut self,
            _blob: fret_core::TextBlobId,
            point: Point,
        ) -> fret_core::HitTestResult {
            let line_h = 10.0_f32;
            let char_w = 10.0_f32;

            let mut line = (point.y.0 / line_h).floor() as i32;
            line = line.clamp(0, self.line_count().saturating_sub(1) as i32);
            let line = line as usize;

            let (start, end) = self.line_range(line).unwrap_or((0, 0));
            let len = end.saturating_sub(start);

            let mut col = (point.x.0 / char_w).round() as i32;
            col = col.clamp(0, len as i32);
            let col = col as usize;

            fret_core::HitTestResult {
                index: (start + col).min(self.text.len()),
                affinity: fret_core::CaretAffinity::Downstream,
            }
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for LineTextService {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for LineTextService {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = LineTextService::default();

    let text = "0123456789\nabc\n0123456789";

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-preferred-x",
        |cx| {
            vec![
                cx.selectable_text_props(crate::element::SelectableTextProps {
                    layout: Default::default(),
                    rich: attributed_plain(text),
                    style: None,
                    color: None,
                    wrap: fret_core::TextWrap::None,
                    overflow: fret_core::TextOverflow::Clip,
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let selectable_node = ui.children(root)[0];
    let record =
        crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
            .expect("selectable record");
    let element = record.element;

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(80.0), Px(5.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let (caret, preferred_x) = crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::SelectableTextState::default,
        |state| (state.caret, state.preferred_x),
    );
    assert_eq!(
        caret, 14,
        "expected down to clamp into the short middle line"
    );
    assert_eq!(
        preferred_x,
        Some(Px(80.0)),
        "expected preferred_x to preserve the original column"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let caret = crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::SelectableTextState::default,
        |state| state.caret,
    );
    assert_eq!(
        caret, 23,
        "expected preferred_x to restore the original column on the next long line"
    );
}


#[test]
fn selectable_text_sets_active_text_selection() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let rich = attributed_plain("hello world");
    let root_name = "selectable-text-active-text-selection";
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let selectable_node = ui.children(root)[0];
    let record =
        crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
            .expect("selectable record");
    let element = record.element;

    let pos = Point::new(Px(5.0), Px(5.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let active =
        crate::elements::with_window_state(&mut app, window, |st| st.active_text_selection());
    assert_eq!(
        active,
        Some(crate::elements::ActiveTextSelection {
            root: crate::elements::global_root(window, root_name),
            element,
        }),
        "expected active text selection to be tracked while selection is non-empty"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::SetTextSelection {
            anchor: 0,
            focus: 0,
        },
    );

    let active =
        crate::elements::with_window_state(&mut app, window, |st| st.active_text_selection());
    assert_eq!(
        active, None,
        "expected active text selection to clear when selection is collapsed"
    );
}


#[test]
fn selectable_text_copy_availability_requires_selection() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let rich = attributed_plain("hello world");
    let root_name = "selectable-text-copy-availability";
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let selectable_node = ui.children(root)[0];
    ui.set_focus(Some(selectable_node));

    let copy = CommandId::from("text.copy");
    let select_all = CommandId::from("text.select_all");

    assert!(
        ui.is_command_available(&mut app, &select_all),
        "expected text.select_all to be available for focused selectable text"
    );
    assert!(
        !ui.is_command_available(&mut app, &copy),
        "expected text.copy to be unavailable without a selection"
    );

    assert!(
        ui.dispatch_command(&mut app, &mut services, &select_all),
        "expected text.select_all to be handled by selectable text"
    );

    assert!(
        ui.is_command_available(&mut app, &copy),
        "expected text.copy to be available when a selection exists"
    );
}


#[test]
fn selectable_text_copy_availability_respects_clipboard_capabilities() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities {
        clipboard: fret_runtime::capabilities::ClipboardCapabilities {
            text: false,
            files: false,
            primary_text: false,
        },
        ..fret_runtime::PlatformCapabilities::default()
    });
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let rich = attributed_plain("hello world");
    let root_name = "selectable-text-copy-availability-clipboard-caps";
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let selectable_node = ui.children(root)[0];
    ui.set_focus(Some(selectable_node));

    let copy = CommandId::from("text.copy");
    let select_all = CommandId::from("text.select_all");

    assert!(
        ui.dispatch_command(&mut app, &mut services, &select_all),
        "expected text.select_all to be handled by selectable text"
    );
    assert!(
        !ui.is_command_available(&mut app, &copy),
        "expected text.copy to be unavailable when clipboard text is unsupported"
    );
    assert!(
        !ui.dispatch_command(&mut app, &mut services, &copy),
        "expected text.copy to not be handled when clipboard text is unsupported"
    );
}

