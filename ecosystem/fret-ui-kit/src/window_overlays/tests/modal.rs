use super::*;

#[test]
fn window_resize_closes_modal_overlays_that_opt_in() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();

    let open = app.models_mut().insert(true);
    fret_runtime::apply_window_metrics_event(&mut app, window, &Event::WindowFocusChanged(true));

    let bounds_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );
    let bounds_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(120.0)),
    );

    let _trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds_a,
        open.clone(),
    );

    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0x1),
            root_name: "modal".into(),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: true,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds_a);
    assert_eq!(app.models().get_copied(&open), Some(true));

    begin_frame(&mut app, window);
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0x1),
            root_name: "modal".into(),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: true,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds_b);
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn window_focus_lost_closes_modal_overlays_that_opt_in() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();

    let open = app.models_mut().insert(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let _trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    fret_runtime::apply_window_metrics_event(&mut app, window, &Event::WindowFocusChanged(true));
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0x1),
            root_name: "modal".into(),
            trigger: None,
            close_on_window_focus_lost: true,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert_eq!(app.models().get_copied(&open), Some(true));

    begin_frame(&mut app, window);
    fret_runtime::apply_window_metrics_event(&mut app, window, &Event::WindowFocusChanged(false));
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0x1),
            root_name: "modal".into(),
            trigger: None,
            close_on_window_focus_lost: true,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn modal_blocks_underlay_click_and_closes_on_escape() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base layer contains a pressable that increments underlay_clicks.
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&underlay_clicked);
                    vec![]
                },
            )]
        },
    );
    ui.set_root(base);

    // Install modal layer.
    begin_frame(&mut app, window);
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(80.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| vec![],
            )]
        });
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0xabc),
            root_name: modal_root_name(GlobalElementId(0xabc)),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Clicking underlay area should not reach base (modal barrier blocks underlay input).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));

    // Escape should close via DismissibleLayer.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn modal_dismiss_handler_can_prevent_default_close() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);
    let dismiss_called = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_cx| Vec::new(),
    );
    ui.set_root(base);

    begin_frame(&mut app, window);
    let dismiss_called_for_handler = dismiss_called.clone();
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0x111),
            root_name: modal_root_name(GlobalElementId(0x111)),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: Some(Arc::new(move |host, _cx, req| {
                let _ = host
                    .models_mut()
                    .update(&dismiss_called_for_handler, |v| *v = true);
                req.prevent_default();
            })),
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(app.models().get_copied(&dismiss_called), Some(true));
    assert_eq!(app.models().get_copied(&open), Some(true));
}

#[test]
fn modal_can_remain_present_while_still_blocking_underlay_during_close_animation() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);
    let overlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base layer contains a full-size pressable we expect NOT to receive the click.
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&underlay_clicked);
                    vec![]
                },
            )]
        },
    );
    ui.set_root(base);

    // Install a modal layer that is still `present` but `open=false` (closing animation).
    begin_frame(&mut app, window);
    let modal_id = GlobalElementId(0xbeef);
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: false,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&overlay_clicked);
                    vec![]
                },
            )]
        });
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open,
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays.modals.get(&(window, modal_id)).map(|p| p.layer)
        })
        .expect("modal layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("modal debug layer info");

    assert!(info.visible);
    assert!(info.blocks_underlay_input);
    assert!(info.hit_testable);
    assert_eq!(
        info.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None
    );
    assert!(!info.wants_pointer_down_outside_events);
    assert!(!info.wants_pointer_move_events);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));
    assert_eq!(app.models().get_copied(&overlay_clicked), Some(true));
}

#[test]
fn modal_restores_focus_to_trigger_while_closing_but_still_present() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable element mappings for the trigger.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    ui.set_focus(Some(trigger_node));

    let modal_id = GlobalElementId(0xabc);
    let mut modal_focusable: Option<GlobalElementId> = None;
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            vec![cx.pressable_with_id(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(80.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st, id| {
                    modal_focusable = Some(id);
                    Vec::new()
                },
            )]
        });
    let modal_focusable = modal_focusable.expect("modal focusable element id");

    // Second frame: install a modal overlay and ensure focus is inside the modal layer.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(modal_focusable),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children.clone(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let modal_focus_node =
        fret_ui::elements::node_for_element(&mut app, window, modal_focusable).expect("modal node");
    assert_eq!(ui.focus(), Some(modal_focus_node));

    // Third frame: close (`open=false`) but keep `present=true` to simulate an exit transition.
    let _ = app.models_mut().update(&open, |v| *v = false);

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(modal_focusable),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    assert_eq!(ui.focus(), Some(trigger_node));
}

#[test]
fn modal_close_auto_focus_handler_can_prevent_default_restore() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let modal_id = GlobalElementId(0xabc);
    let mut modal_focusable: Option<GlobalElementId> = None;
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            vec![cx.pressable_with_id(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(80.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st, id| {
                    modal_focusable = Some(id);
                    Vec::new()
                },
            )]
        });
    let modal_focusable = modal_focusable.expect("modal focusable element id");

    let on_close_auto_focus: fret_ui::action::OnCloseAutoFocus =
        Arc::new(|_host, _cx, req| req.prevent_default());

    // Second frame: mount modal and focus inside.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(modal_focusable),
            on_open_auto_focus: None,
            on_close_auto_focus: Some(on_close_auto_focus.clone()),
            on_dismiss_request: None,
            children: modal_children.clone(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let modal_focus_node =
        fret_ui::elements::node_for_element(&mut app, window, modal_focusable).expect("modal node");
    assert_eq!(ui.focus(), Some(modal_focus_node));

    // Third frame: close while still present; prevent restoring focus to the trigger.
    let _ = app.models_mut().update(&open, |v| *v = false);

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(modal_focusable),
            on_open_auto_focus: None,
            on_close_auto_focus: Some(on_close_auto_focus),
            on_dismiss_request: None,
            children: modal_children,
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(modal_focus_node));
}

#[test]
fn modal_initial_focus_is_only_applied_on_opening_edge() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable element mappings for the trigger.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let modal_id = GlobalElementId(0xabc);
    let mut a: Option<GlobalElementId> = None;
    let mut b: Option<GlobalElementId> = None;

    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            let props = PressableProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(80.0));
                    layout.size.height = Length::Px(Px(32.0));
                    layout
                },
                enabled: true,
                focusable: true,
                ..Default::default()
            };

            vec![
                cx.pressable_with_id(props.clone(), |_cx, _st, id| {
                    a = Some(id);
                    Vec::new()
                }),
                cx.pressable_with_id(props, |_cx, _st, id| {
                    b = Some(id);
                    Vec::new()
                }),
            ]
        });
    let a = a.expect("focusable a element id");
    let b = b.expect("focusable b element id");

    // Second frame: mount a modal overlay with two focusable elements, using `initial_focus=A`.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(a),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children.clone(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let node_a = fret_ui::elements::node_for_element(&mut app, window, a).expect("node a");
    let node_b = fret_ui::elements::node_for_element(&mut app, window, b).expect("node b");
    assert_eq!(ui.focus(), Some(node_a));

    // Simulate user moving focus within the modal.
    ui.set_focus(Some(node_b));
    assert_eq!(ui.focus(), Some(node_b));

    // Third frame: keep the modal open and re-request it. Initial focus should not be re-applied.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(a),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children,
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(node_b));
}

#[test]
fn modal_reasserts_focus_when_focus_leaves_modal_layer_while_open() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable element mappings for the trigger.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    ui.set_focus(Some(trigger_node));

    let modal_id = GlobalElementId(0xabc);
    let mut modal_focusable: Option<GlobalElementId> = None;
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            vec![cx.pressable_with_id(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(80.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st, id| {
                    modal_focusable = Some(id);
                    Vec::new()
                },
            )]
        });
    let modal_focusable = modal_focusable.expect("modal focusable element id");

    // Second frame: install a modal overlay. Focus should move inside the modal layer.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(modal_focusable),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children.clone(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let modal_focus_node =
        fret_ui::elements::node_for_element(&mut app, window, modal_focusable).expect("modal node");
    assert_eq!(ui.focus(), Some(modal_focus_node));

    // Simulate a bug where focus is programmatically moved back to the underlay while the modal is
    // still open. The overlay policy should reassert focus containment on the next frame.
    ui.set_focus(Some(trigger_node));

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(modal_focusable),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children,
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(modal_focus_node));
}
