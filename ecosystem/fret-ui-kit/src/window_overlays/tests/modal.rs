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

    fn build_modal_children<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
        focusable_out: Option<&mut Option<GlobalElementId>>,
    ) -> Vec<fret_ui::element::AnyElement> {
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
            move |_cx, _st, id| {
                if let Some(out) = focusable_out {
                    *out = Some(id);
                }
                Vec::new()
            },
        )]
    }

    let modal_id = GlobalElementId(0xabc);
    let mut modal_focusable: Option<GlobalElementId> = None;
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            build_modal_children(cx, Some(&mut modal_focusable))
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

    let modal_children_for_open =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            build_modal_children(cx, None)
        });
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
            children: modal_children_for_open,
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

    fn build_modal_children<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
        focusable_out: Option<&mut Option<GlobalElementId>>,
    ) -> Vec<fret_ui::element::AnyElement> {
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
            move |_cx, _st, id| {
                if let Some(out) = focusable_out {
                    *out = Some(id);
                }
                Vec::new()
            },
        )]
    }

    let modal_id = GlobalElementId(0xabc);
    let mut modal_focusable: Option<GlobalElementId> = None;
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            build_modal_children(cx, Some(&mut modal_focusable))
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
    let modal_children_for_open =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            build_modal_children(cx, None)
        });
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
            children: modal_children_for_open,
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

    fn build_modal_children<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
        a_out: Option<&mut Option<GlobalElementId>>,
        b_out: Option<&mut Option<GlobalElementId>>,
    ) -> Vec<fret_ui::element::AnyElement> {
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
            cx.pressable_with_id(props.clone(), move |_cx, _st, id| {
                if let Some(out) = a_out {
                    *out = Some(id);
                }
                Vec::new()
            }),
            cx.pressable_with_id(props, move |_cx, _st, id| {
                if let Some(out) = b_out {
                    *out = Some(id);
                }
                Vec::new()
            }),
        ]
    }

    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            build_modal_children(cx, Some(&mut a), Some(&mut b))
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

    let modal_children_for_open =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            build_modal_children(cx, None, None)
        });
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
            children: modal_children_for_open,
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
fn reopening_attached_modal_republishes_input_context_after_focus_handoff() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let command = fret_runtime::CommandId::from("test.modal_reopen_available");
    app.commands_mut().register(
        command.clone(),
        fret_runtime::CommandMeta::new("Modal Reopen Available")
            .with_scope(fret_runtime::CommandScope::Widget),
    );
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let mut trigger_id: Option<GlobalElementId> = None;

    let value = app.models_mut().insert(String::new());

    fn build_modal_children<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
        focusable_out: Option<&mut Option<GlobalElementId>>,
        value: Model<String>,
        command: fret_runtime::CommandId,
    ) -> Vec<fret_ui::element::AnyElement> {
        let input = cx
            .text_input(fret_ui::element::TextInputProps::new(value.clone()))
            .key_context("modal.reopen");
        if let Some(out) = focusable_out {
            *out = Some(input.id);
        }
        let shell = cx.container(ContainerProps::default(), |_cx| vec![input]);
        cx.command_on_command_availability_for(
            shell.id,
            Arc::new(move |_host, _acx, requested| {
                if requested == command {
                    return fret_ui::CommandAvailability::Available;
                }
                fret_ui::CommandAvailability::NotHandled
            }),
        );
        vec![shell]
    }

    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "modal-reopen-keyctx-base",
        |cx| {
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
                    trigger_id = Some(id);
                    Vec::new()
                },
            )]
        },
    );
    ui.set_root(base);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger = trigger_id.expect("trigger element id");
    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    ui.set_focus(Some(trigger_node));
    ui.publish_window_runtime_snapshots(&mut app);

    let initial_input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("initial input context snapshot");
    assert!(
        !initial_input_ctx.focus_is_text_input,
        "underlay trigger focus should publish a non-text-input snapshot before the modal reopens"
    );

    let modal_id = GlobalElementId(0xfeed);
    let mut modal_focusable: Option<GlobalElementId> = None;
    let modal_children_closed =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-reopen-keyctx", |cx| {
            build_modal_children(
                cx,
                Some(&mut modal_focusable),
                value.clone(),
                command.clone(),
            )
        });
    let modal_focusable = modal_focusable.expect("modal focusable id");

    begin_frame(&mut app, window);
    let _ = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "modal-reopen-keyctx-base",
        |cx| {
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
                |_cx, _st, _id| Vec::new(),
            )]
        },
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
            children: modal_children_closed,
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let closed_input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("closed modal input context snapshot");
    assert!(
        !closed_input_ctx.focus_is_text_input,
        "keeping the attached modal closed should not move the published input context into text-input mode"
    );
    let closed_key_contexts = app
        .global::<fret_runtime::WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(window))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    assert!(
        closed_key_contexts.is_empty(),
        "keeping the attached modal closed should not publish the modal key context onto the authoritative window routing stack"
    );
    let closed_availability = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &command));
    assert_eq!(
        closed_availability,
        Some(false),
        "keeping the attached modal closed should leave the modal widget command unavailable to same-frame consumers"
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    let modal_children_open =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-reopen-keyctx", |cx| {
            build_modal_children(cx, None, value.clone(), command.clone())
        });

    begin_frame(&mut app, window);
    let _ = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "modal-reopen-keyctx-base",
        |cx| {
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
                |_cx, _st, _id| Vec::new(),
            )]
        },
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
            children: modal_children_open,
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let modal_focus_node = fret_ui::elements::node_for_element(&mut app, window, modal_focusable)
        .expect("modal focus node");
    assert_eq!(
        ui.focus(),
        Some(modal_focus_node),
        "reopening the attached modal should hand focus to the requested modal element in the render frame"
    );

    let reopened_input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("reopened modal input context snapshot");
    assert!(
        reopened_input_ctx.focus_is_text_input,
        "reopening an attached modal should republish the window input context after focus moves into the modal text input"
    );
    let reopened_key_contexts = app
        .global::<fret_runtime::WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(window))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    assert_eq!(
        reopened_key_contexts,
        vec![Arc::<str>::from("modal.reopen")],
        "reopening an attached modal should republish the window key-context stack after focus moves into the modal subtree"
    );
    let reopened_availability = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &command));
    assert_eq!(
        reopened_availability,
        Some(true),
        "reopening an attached modal should republish widget command availability after focus moves into the modal subtree"
    );
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

    fn build_modal_children<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
        focusable_out: Option<&mut Option<GlobalElementId>>,
    ) -> Vec<fret_ui::element::AnyElement> {
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
            move |_cx, _st, id| {
                if let Some(out) = focusable_out {
                    *out = Some(id);
                }
                Vec::new()
            },
        )]
    }

    let modal_id = GlobalElementId(0xabc);
    let mut modal_focusable: Option<GlobalElementId> = None;
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            build_modal_children(cx, Some(&mut modal_focusable))
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
    let modal_children_for_open =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            build_modal_children(cx, None)
        });
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
            children: modal_children_for_open,
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

#[test]
fn popover_requested_after_modal_stays_above_parent_modal_layer() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let parent_open = app.models_mut().insert(true);
    let child_open = app.models_mut().insert(true);
    let parent_clicked = app.models_mut().insert(false);
    let child_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(320.0), Px(240.0)),
    );

    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        child_open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        child_open.clone(),
    );

    let parent_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "parent-modal-child", |cx| {
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
                    cx.pressable_toggle_bool(&parent_clicked);
                    Vec::new()
                },
            )]
        });

    let child_id = GlobalElementId(0xcafe);
    let child_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "child-popover", |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            left: Some(Px(120.0)).into(),
                            top: Some(Px(48.0)).into(),
                            ..Default::default()
                        },
                        size: SizeStyle {
                            width: Length::Px(Px(96.0)),
                            height: Length::Px(Px(44.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&child_clicked);
                    Vec::new()
                },
            )]
        });

    let parent_id = GlobalElementId(0xbeef);
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: parent_id,
            root_name: modal_root_name(parent_id),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: parent_open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: parent_children,
        },
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: child_id,
            root_name: popover_root_name(child_id),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: child_open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: child_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let (parent_layer, child_layer) =
        app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            let parent_layer = overlays
                .modals
                .get(&(window, parent_id))
                .map(|entry| entry.layer)
                .expect("parent modal layer");
            let child_layer = overlays
                .popovers
                .get(&(window, child_id))
                .map(|entry| entry.layer)
                .expect("child popover layer");
            (parent_layer, child_layer)
        });

    let layers = ui.debug_layers_in_paint_order();
    let parent_index = layers
        .iter()
        .position(|layer| layer.id == parent_layer)
        .expect("parent layer index");
    let child_index = layers
        .iter()
        .position(|layer| layer.id == child_layer)
        .expect("child layer index");
    assert!(
        child_index > parent_index,
        "expected child popover layer above parent modal; parent_index={parent_index} child_index={child_index} layers={layers:?}"
    );

    let point = Point::new(Px(124.0), Px(52.0));
    let hit = ui.debug_hit_test(point);
    let hit_node = hit.hit.expect("hit node");
    let hit_layer = ui.node_layer(hit_node);
    assert!(
        hit_layer == Some(child_layer),
        "expected point inside child popover to route to child layer; point={point:?} hit={hit:?} hit_layer={hit_layer:?} child_layer={child_layer:?}"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: point,
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
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: point,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&child_clicked), Some(true));
    assert_eq!(app.models().get_copied(&parent_clicked), Some(false));
}
