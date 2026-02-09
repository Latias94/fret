use super::*;

#[test]
fn text_area_select_all_is_blocked_when_empty() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let model = app.models_mut().insert(String::new());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-area-select-all-empty",
        |cx| vec![cx.text_area(crate::element::TextAreaProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let area_node = ui.children(root)[0];
    ui.set_focus(Some(area_node));

    let select_all = CommandId::from("text.select_all");
    let edit_select_all = CommandId::from("edit.select_all");
    let clear = CommandId::from("text.clear");
    let edit_copy = CommandId::from("edit.copy");
    let edit_cut = CommandId::from("edit.cut");
    let unknown = CommandId::from("text.unknown");

    assert!(
        !ui.is_command_available(&mut app, &select_all),
        "expected text.select_all to be unavailable for empty text area"
    );
    assert!(
        !ui.is_command_available(&mut app, &edit_select_all),
        "expected edit.select_all to be unavailable for empty text area"
    );
    assert!(
        !ui.is_command_available(&mut app, &clear),
        "expected text.clear to be unavailable for empty text area"
    );
    assert!(
        !ui.is_command_available(&mut app, &edit_copy),
        "expected edit.copy to be unavailable without a selection"
    );
    assert!(
        !ui.is_command_available(&mut app, &edit_cut),
        "expected edit.cut to be unavailable without a selection"
    );
    assert!(
        !ui.is_command_available(&mut app, &unknown),
        "expected unknown text.* commands to be NotHandled for availability"
    );
}


#[test]
fn text_area_double_click_respects_window_text_boundary_mode_under_render_transform() {
    fn selection_for_mode(mode: fret_runtime::TextBoundaryMode) -> Option<(u32, u32)> {
        let mut app = TestHost::new();
        app.set_global(fret_runtime::PlatformCapabilities::default());
        app.with_global_mut_untracked(
            fret_runtime::WindowTextBoundaryModeService::default,
            |svc, _app| {
                svc.set_base_mode(AppWindowId::default(), mode);
            },
        );

        let model = app.models_mut().insert("can't".to_string());

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(220.0), Px(120.0)),
        );
        let mut services = FakeTextService::default();

        let transform = Transform2D::translation(Point::new(Px(30.0), Px(10.0)));
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "text-area-double-click-boundary-mode-transform",
            |cx| {
                vec![cx.render_transform(transform, |cx| {
                    let mut props = crate::element::TextAreaProps::new(model.clone());
                    props.layout.size.width = Length::Px(Px(160.0));
                    props.layout.size.height = Length::Px(Px(80.0));
                    vec![cx.text_area(props)]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let transform_node = ui.children(root)[0];
        let area_node = ui.children(transform_node)[0];
        let area_bounds = ui.debug_node_visual_bounds(area_node).expect("area bounds");
        let pos = Point::new(
            Px(area_bounds.origin.x.0 + 5.0),
            Px(area_bounds.origin.y.0 + 5.0),
        );
        assert_eq!(
            ui.debug_hit_test(pos).hit,
            Some(area_node),
            "expected the translated hit-test position to target the text area"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert_eq!(ui.focus(), Some(area_node));
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 2,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert_eq!(ui.focus(), Some(area_node));

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let snapshot = app
            .global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window))
            .cloned()
            .expect("expected a window text input snapshot");
        assert!(snapshot.focus_is_text_input);
        snapshot.selection_utf16
    }

    assert_eq!(
        selection_for_mode(fret_runtime::TextBoundaryMode::UnicodeWord),
        Some((0, 5)),
        "UnicodeWord should select the whole word"
    );
    assert_eq!(
        selection_for_mode(fret_runtime::TextBoundaryMode::Identifier),
        Some((0, 3)),
        "Identifier should stop at the apostrophe"
    );
}


#[test]
fn text_area_triple_click_selects_logical_line_including_newline_under_render_transform() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let model = app.models_mut().insert("abc\ndef".to_string());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(220.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let transform = Transform2D::translation(Point::new(Px(30.0), Px(10.0)));
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-area-triple-click-select-line-transform",
        |cx| {
            vec![cx.render_transform(transform, |cx| {
                let mut props = crate::element::TextAreaProps::new(model.clone());
                props.layout.size.width = Length::Px(Px(160.0));
                props.layout.size.height = Length::Px(Px(80.0));
                vec![cx.text_area(props)]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let transform_node = ui.children(root)[0];
    let area_node = ui.children(transform_node)[0];
    let area_bounds = ui.debug_node_visual_bounds(area_node).expect("area bounds");
    let pos = Point::new(
        Px(area_bounds.origin.x.0 + 5.0),
        Px(area_bounds.origin.y.0 + 5.0),
    );
    assert_eq!(
        ui.debug_hit_test(pos).hit,
        Some(area_node),
        "expected the translated hit-test position to target the text area"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(area_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 3,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(area_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot");
    assert!(snapshot.focus_is_text_input);
    assert_eq!(
        snapshot.selection_utf16,
        Some((0, 4)),
        "triple click should select first line (including trailing newline)"
    );
}


#[test]
fn text_area_double_click_cancels_ime_preedit() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let text = "hello world".to_string();
    let base_len_utf16: u32 = text.encode_utf16().count().try_into().unwrap();
    let model = app.models_mut().insert(text);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(220.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-area-double-click-cancels-ime-preedit",
        |cx| {
            let mut props = crate::element::TextAreaProps::new(model.clone());
            props.layout.size.width = Length::Px(Px(200.0));
            props.layout.size.height = Length::Px(Px(80.0));
            vec![cx.text_area(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let area_node = ui.children(root)[0];
    let area_bounds = ui.debug_node_visual_bounds(area_node).expect("area bounds");
    let pos = Point::new(
        Px(area_bounds.origin.x.0 + 5.0),
        Px(area_bounds.origin.y.0 + 5.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(area_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Ime(fret_core::ImeEvent::Preedit {
            text: "X".to_string(),
            cursor: Some((0, 1)),
        }),
    );
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after preedit");
    assert!(snapshot.focus_is_text_input);
    assert!(snapshot.is_composing);
    assert!(snapshot.marked_utf16.is_some());
    assert_eq!(
        snapshot.text_len_utf16,
        base_len_utf16 + 1,
        "expected composed text length to include the preedit"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after double click");
    assert!(snapshot.focus_is_text_input);
    assert!(!snapshot.is_composing);
    assert_eq!(snapshot.marked_utf16, None);
    assert_eq!(snapshot.text_len_utf16, base_len_utf16);
}


#[test]
fn text_area_triple_click_cancels_ime_preedit() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let text = "hello world".to_string();
    let base_len_utf16: u32 = text.encode_utf16().count().try_into().unwrap();
    let model = app.models_mut().insert(text);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(220.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-area-triple-click-cancels-ime-preedit",
        |cx| {
            let mut props = crate::element::TextAreaProps::new(model.clone());
            props.layout.size.width = Length::Px(Px(200.0));
            props.layout.size.height = Length::Px(Px(80.0));
            vec![cx.text_area(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let area_node = ui.children(root)[0];
    let area_bounds = ui.debug_node_visual_bounds(area_node).expect("area bounds");
    let pos = Point::new(
        Px(area_bounds.origin.x.0 + 5.0),
        Px(area_bounds.origin.y.0 + 5.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(area_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Ime(fret_core::ImeEvent::Preedit {
            text: "X".to_string(),
            cursor: Some((0, 1)),
        }),
    );
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after preedit");
    assert!(snapshot.focus_is_text_input);
    assert!(snapshot.is_composing);
    assert!(snapshot.marked_utf16.is_some());
    assert_eq!(
        snapshot.text_len_utf16,
        base_len_utf16 + 1,
        "expected composed text length to include the preedit"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 3,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after triple click");
    assert!(snapshot.focus_is_text_input);
    assert!(!snapshot.is_composing);
    assert_eq!(snapshot.marked_utf16, None);
    assert_eq!(snapshot.text_len_utf16, base_len_utf16);
    assert_eq!(snapshot.selection_utf16, Some((0, base_len_utf16)));
}


#[test]
fn text_area_double_click_respects_window_text_boundary_mode_under_scroll_offset() {
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

    let model = app.models_mut().insert("can't".to_string());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(220.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-area-double-click-boundary-mode-scroll",
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

                            let mut props = crate::element::TextAreaProps::new(model.clone());
                            props.layout.size.width = Length::Px(Px(160.0));
                            props.layout.size.height = Length::Px(Px(80.0));
                            out.push(cx.text_area(props));

                            out
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let area_node = *ui
        .children(column_node)
        .last()
        .expect("expected area as last child");
    let area_bounds = ui.debug_node_bounds(area_node).expect("area bounds");

    scroll_handle.set_offset(Point::new(Px(0.0), area_bounds.origin.y));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let area_bounds = ui
        .debug_node_visual_bounds(area_node)
        .expect("area bounds after scroll");
    let pos = Point::new(
        Px(area_bounds.origin.x.0 + 5.0),
        Px(area_bounds.origin.y.0 + 5.0),
    );
    assert_eq!(
        ui.debug_hit_test(pos).hit,
        Some(area_node),
        "expected the scrolled hit-test position to target the text area"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(area_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(area_node));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot");
    assert!(snapshot.focus_is_text_input);
    let selection_utf16 = snapshot.selection_utf16;

    assert_eq!(
        selection_utf16,
        Some((0, 3)),
        "Identifier mode should stop at the apostrophe"
    );
}


#[test]
fn text_area_word_navigation_respects_window_text_boundary_mode() {
    fn caret_positions_for_mode(mode: fret_runtime::TextBoundaryMode) -> (u32, u32) {
        let mut app = TestHost::new();
        app.set_global(fret_runtime::PlatformCapabilities::default());
        app.with_global_mut_untracked(
            fret_runtime::WindowTextBoundaryModeService::default,
            |svc, _app| {
                svc.set_base_mode(AppWindowId::default(), mode);
            },
        );

        let model = app.models_mut().insert("can't".to_string());

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(220.0), Px(120.0)),
        );
        let mut services = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "text-area-word-nav-boundary-mode",
            |cx| {
                let mut props = crate::element::TextAreaProps::new(model.clone());
                props.layout.size.width = Length::Px(Px(200.0));
                props.layout.size.height = Length::Px(Px(80.0));
                vec![cx.text_area(props)]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let area_node = ui.children(root)[0];
        ui.set_focus(Some(area_node));

        // Ensure the underlying text blob exists so line-based commands (`move_home` / `move_end`)
        // can use text geometry queries.
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let move_home = CommandId::from("text.move_home");
        assert!(
            ui.dispatch_command(&mut app, &mut services, &move_home),
            "expected text.move_home to be handled by text area"
        );
        let move_word_right = CommandId::from("text.move_word_right");
        assert!(
            ui.dispatch_command(&mut app, &mut services, &move_word_right),
            "expected text.move_word_right to be handled by text area"
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        let snapshot = app
            .global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window))
            .cloned()
            .expect("expected a window text input snapshot");
        assert!(snapshot.focus_is_text_input);
        let (anchor_u16, focus_u16) = snapshot.selection_utf16.expect("selection");
        assert_eq!(
            anchor_u16, focus_u16,
            "expected a collapsed selection after move"
        );
        let caret_right = focus_u16;

        let select_all = CommandId::from("text.select_all");
        assert!(
            ui.dispatch_command(&mut app, &mut services, &select_all),
            "expected text.select_all to be handled by text area"
        );
        let move_right = CommandId::from("text.move_right");
        assert!(
            ui.dispatch_command(&mut app, &mut services, &move_right),
            "expected text.move_right to be handled by text area"
        );
        let move_word_left = CommandId::from("text.move_word_left");
        assert!(
            ui.dispatch_command(&mut app, &mut services, &move_word_left),
            "expected text.move_word_left to be handled by text area"
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        let snapshot = app
            .global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window))
            .cloned()
            .expect("expected a window text input snapshot");
        assert!(snapshot.focus_is_text_input);
        let (anchor_u16, focus_u16) = snapshot.selection_utf16.expect("selection");
        assert_eq!(
            anchor_u16, focus_u16,
            "expected a collapsed selection after move"
        );
        let caret_left = focus_u16;

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
fn text_area_delete_word_respects_window_text_boundary_mode() {
    fn text_after_commands(
        mode: fret_runtime::TextBoundaryMode,
        commands: &[&'static str],
    ) -> String {
        let mut app = TestHost::new();
        app.set_global(fret_runtime::PlatformCapabilities::default());
        app.with_global_mut_untracked(
            fret_runtime::WindowTextBoundaryModeService::default,
            |svc, _app| {
                svc.set_base_mode(AppWindowId::default(), mode);
            },
        );

        let model = app.models_mut().insert("can't".to_string());

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(220.0), Px(120.0)),
        );
        let mut services = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "text-area-delete-word-boundary-mode",
            |cx| {
                let mut props = crate::element::TextAreaProps::new(model.clone());
                props.layout.size.width = Length::Px(Px(200.0));
                props.layout.size.height = Length::Px(Px(80.0));
                vec![cx.text_area(props)]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let area_node = ui.children(root)[0];
        ui.set_focus(Some(area_node));

        for command in commands {
            let command_id = CommandId::from(*command);
            assert!(
                ui.dispatch_command(&mut app, &mut services, &command_id),
                "expected {command_id:?} to be handled by text area"
            );
        }

        app.models().get_cloned(&model).unwrap_or_default()
    }

    assert_eq!(
        text_after_commands(
            fret_runtime::TextBoundaryMode::UnicodeWord,
            &[
                "text.select_all",
                "text.move_right",
                "text.delete_word_backward"
            ],
        ),
        "",
        "UnicodeWord should delete the whole word on delete_word_backward"
    );
    assert_eq!(
        text_after_commands(
            fret_runtime::TextBoundaryMode::Identifier,
            &[
                "text.select_all",
                "text.move_right",
                "text.delete_word_backward"
            ],
        ),
        "can'",
        "Identifier should delete only the last identifier segment on delete_word_backward"
    );
    assert_eq!(
        text_after_commands(
            fret_runtime::TextBoundaryMode::UnicodeWord,
            &[
                "text.select_all",
                "text.move_word_left",
                "text.move_word_left",
                "text.move_word_left",
                "text.delete_word_forward",
            ],
        ),
        "",
        "UnicodeWord should delete the whole word on delete_word_forward"
    );
    assert_eq!(
        text_after_commands(
            fret_runtime::TextBoundaryMode::Identifier,
            &[
                "text.select_all",
                "text.move_word_left",
                "text.move_word_left",
                "text.move_word_left",
                "text.delete_word_forward",
            ],
        ),
        "'t",
        "Identifier should delete only the first identifier segment on delete_word_forward"
    );
}


#[test]
fn text_area_triple_click_selects_logical_line_including_newline_under_scroll_offset() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let model = app.models_mut().insert("abc\ndef".to_string());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(220.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-area-triple-click-select-line-scroll",
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

                            let mut props = crate::element::TextAreaProps::new(model.clone());
                            props.layout.size.width = Length::Px(Px(160.0));
                            props.layout.size.height = Length::Px(Px(80.0));
                            out.push(cx.text_area(props));

                            out
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let area_node = *ui
        .children(column_node)
        .last()
        .expect("expected area as last child");
    let area_bounds = ui.debug_node_bounds(area_node).expect("area bounds");

    scroll_handle.set_offset(Point::new(Px(0.0), area_bounds.origin.y));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let area_bounds = ui
        .debug_node_visual_bounds(area_node)
        .expect("area bounds after scroll");
    let pos = Point::new(
        Px(area_bounds.origin.x.0 + 5.0),
        Px(area_bounds.origin.y.0 + 5.0),
    );
    assert_eq!(
        ui.debug_hit_test(pos).hit,
        Some(area_node),
        "expected the scrolled hit-test position to target the text area"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(area_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 3,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(area_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot");
    assert!(snapshot.focus_is_text_input);
    assert_eq!(
        snapshot.selection_utf16,
        Some((0, 4)),
        "triple click should select first line (including trailing newline)"
    );
}

