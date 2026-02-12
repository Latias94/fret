use super::*;

#[test]
fn non_modal_overlay_open_auto_focus_handler_can_prevent_default_focus() {
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

    // First frame: base root to establish stable element mappings for the trigger.
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

    let overlay_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "popover-child", |cx| {
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
                |_cx, _st| Vec::new(),
            )]
        });

    let on_open_auto_focus: fret_ui::action::OnOpenAutoFocus =
        Arc::new(|_host, _cx, req| req.prevent_default());

    // Second frame: mount a non-modal overlay and suppress default initial focus.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: Some(on_open_auto_focus),
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: overlay_children,
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(trigger_node));
}

#[test]
fn non_modal_overlay_can_remain_present_while_pointer_transparent_during_close_animation() {
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

    // Base layer contains a full-size pressable we expect to receive the click.
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

    // Install a non-modal layer that is still `present` but `open=false` (closing animation).
    begin_frame(&mut app, window);
    let trigger = GlobalElementId(0xdead);
    let overlay_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "popover-child", |cx| {
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

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open,
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: overlay_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

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

    assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
    assert_eq!(app.models().get_copied(&overlay_clicked), Some(false));
}

#[test]
fn non_modal_overlay_disable_outside_pointer_events_does_not_block_underlay_while_closing() {
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

    begin_frame(&mut app, window);
    let trigger = GlobalElementId(0xdead);
    let overlay_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "popover-child", |cx| {
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

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: overlay_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

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

    assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
    assert_eq!(app.models().get_copied(&overlay_clicked), Some(false));
}

#[test]
fn non_modal_overlay_does_not_request_outside_press_observer_while_closing() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_| Vec::new(),
    );
    ui.set_root(base);

    // Install a non-modal layer that is still `present` but `open=false` (closing animation).
    begin_frame(&mut app, window);
    let trigger = GlobalElementId(0xdead);
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open,
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays.popovers.get(&(window, trigger)).map(|p| p.layer)
        })
        .expect("popover layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("popover debug layer info");

    assert!(info.visible);
    assert!(!info.blocks_underlay_input);
    assert!(!info.hit_testable);
    assert_eq!(
        info.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None,
        "expected non-modal overlays to drop pointer occlusion during close transitions"
    );
    assert!(!info.wants_pointer_down_outside_events);
    assert!(
        !info.wants_pointer_move_events,
        "expected non-modal overlays to stop receiving pointer-move observers during close transitions"
    );
}

#[test]
fn non_modal_overlay_does_not_request_pointer_move_observer_while_closing() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_| Vec::new(),
    );
    ui.set_root(base);

    let trigger = GlobalElementId(0xdead);
    let on_pointer_move: fret_ui::action::OnDismissiblePointerMove =
        Arc::new(|_host, _cx, _move| false);

    // First frame: open the overlay so we know it would normally request pointer-move observers.
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: Some(on_pointer_move.clone()),
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays.popovers.get(&(window, trigger)).map(|p| p.layer)
        })
        .expect("popover layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("popover debug layer info");

    assert!(info.visible);
    assert!(info.hit_testable);
    assert!(info.wants_pointer_move_events);

    // Second frame: close the overlay but keep it present for a close transition. It must not
    // install pointer-move observers while closing.
    let _ = app.models_mut().update(&open, |v| *v = false);
    begin_frame(&mut app, window);
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open,
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: Some(on_pointer_move),
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("popover debug layer info");

    assert!(info.visible);
    assert!(!info.hit_testable);
    assert!(!info.wants_pointer_down_outside_events);
    assert!(!info.wants_pointer_move_events);
}

#[test]
fn non_modal_overlay_restores_focus_when_focus_is_missing_on_unmount() {
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

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Second frame: request and render a dismissible popover (open=true).
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    ui.set_focus(None);

    // Close the overlay so the cached request is no longer synthesized.
    //
    // With cached request declarations (for view caching), an open overlay can be synthesized even
    // if the subtree that emits requests is skipped for a frame. To model a true unmount, ensure
    // `open=false` before the frame where the overlay is no longer requested.
    let _ = app.models_mut().update(&open, |v| *v = false);

    // Third frame: do not request the popover (unmounted), and expect focus to be restored to
    // the trigger since focus is missing.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    assert_eq!(ui.focus(), Some(trigger_node));
}

#[test]
fn non_modal_overlay_does_not_restore_focus_when_focus_moves_to_underlay_on_unmount() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let (trigger, underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click on trigger.
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
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request and render a dismissible popover.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    let overlay_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "overlay-children", |cx| {
            vec![cx.container(
                ContainerProps {
                    layout: {
                        LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                left: Some(Px(0.0)),
                                top: Some(Px(40.0)),
                                ..Default::default()
                            },
                            size: SizeStyle {
                                width: Length::Px(Px(200.0)),
                                height: Length::Px(Px(40.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    },
                    ..Default::default()
                },
                |_| Vec::new(),
            )]
        });
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: overlay_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Click underlay while popover is open: outside press is click-through, so focus should move
    // to the underlay target (and the popover should close).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
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
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&open), Some(false));
    assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));

    let underlay_node =
        fret_ui::elements::node_for_element(&mut app, window, underlay).expect("underlay node");
    assert_eq!(ui.focus(), Some(underlay_node));

    // Third frame: unmount the popover. Focus restoration must not override the new underlay focus.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(underlay_node));
}

#[test]
fn non_modal_overlay_can_consume_outside_press_to_block_underlay_activation() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click on trigger.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
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
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request and render a dismissible popover that consumes outside presses.
    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    let overlay_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "overlay-children", |cx| {
            vec![cx.container(
                ContainerProps {
                    layout: {
                        LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                left: Some(Px(0.0)),
                                top: Some(Px(40.0)),
                                ..Default::default()
                            },
                            size: SizeStyle {
                                width: Length::Px(Px(200.0)),
                                height: Length::Px(Px(40.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    },
                    ..Default::default()
                },
                |_| Vec::new(),
            )]
        });
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: overlay_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Click underlay while popover is open: outside press closes, but must not activate the underlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&open), Some(false));
    assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));

    // Third frame: unmount the popover. Focus should restore to the trigger (since focus stayed inside the overlay).
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    assert_eq!(ui.focus(), Some(trigger_node));
}

#[test]
fn non_modal_overlay_dismiss_handler_can_prevent_default_close() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);
    let underlay_clicked = app.models_mut().insert(false);
    let dismiss_called = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    begin_frame(&mut app, window);
    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    begin_frame(&mut app, window);
    let dismiss_called_for_handler = dismiss_called.clone();
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: false,
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
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Outside press should invoke the dismiss handler, but not close nor activate the underlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&dismiss_called), Some(true));
    assert_eq!(app.models().get_copied(&open), Some(true));
    assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));
}

#[test]
fn non_modal_overlay_can_disable_outside_pointer_events_while_open() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_moved = app.models_mut().insert(false);
    let underlay_scroll = ScrollHandle::default();
    underlay_scroll.set_viewport_size(fret_core::Size::new(Px(160.0), Px(32.0)));
    underlay_scroll.set_content_size(fret_core::Size::new(Px(160.0), Px(200.0)));

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base layer with a pointer region that flips `underlay_moved` on pointer move.
    let (trigger, _underlay) = render_base_with_trigger_and_underlay_pointer_move(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_moved.clone(),
        underlay_scroll.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(10.0), Px(130.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&underlay_moved), Some(true));
    let _ = app.models_mut().update(&underlay_moved, |v| *v = false);

    // Second frame: request and render a dismissible popover that disables outside pointer events
    // while open (Radix `disableOutsidePointerEvents` outcome).
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay_pointer_move(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_moved.clone(),
        underlay_scroll.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(10.0), Px(130.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&underlay_moved), Some(false));

    // Underlay scroll should still be reachable while outside pointer events are disabled:
    // the default policy uses an "except scroll" occlusion mode.
    let prev_scroll_y = underlay_scroll.offset().y;
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(130.0)),
            delta: Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(underlay_scroll.offset().y.0 > prev_scroll_y.0);
}

#[test]
fn non_modal_menu_trigger_press_closes_without_reopening_under_occlusion() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
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
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: render a menu-like dismissible popover that disables outside pointer events.
    begin_frame(&mut app, window);
    let _trigger2 = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Pressing the trigger while open should close the menu-like overlay without immediately
    // re-opening it (a common edge when outside-press dismissal runs before trigger activation).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
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
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn non_modal_menu_blocks_underlay_click_but_allows_wheel() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let underlay_scroll = ScrollHandle::default();
    underlay_scroll.set_viewport_size(fret_core::Size::new(Px(160.0), Px(32.0)));
    underlay_scroll.set_content_size(fret_core::Size::new(Px(160.0), Px(200.0)));

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: base layer with trigger + underlay pressable + wheel region.
    let (trigger, _underlay) = render_base_with_trigger_and_underlay_pressable_wheel(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
        underlay_scroll.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click on the trigger.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
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
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request a menu-like dismissible popover (consume outside presses + occlude mouse).
    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay_pressable_wheel(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
        underlay_scroll.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Wheel should still reach the underlay scroll target even while mouse interactions are blocked.
    let prev_scroll_y = underlay_scroll.offset().y;
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(130.0)),
            delta: Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(underlay_scroll.offset().y.0 > prev_scroll_y.0);

    // Clicking the underlay should dismiss without activating the underlay pressable.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
    assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));
}
