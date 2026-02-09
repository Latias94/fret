use super::*;

#[test]
fn declarative_pointer_region_can_capture_and_receive_move_up() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let counter = app.models_mut().insert(0u32);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-capture-move-up",
        |cx| {
            let counter_down = counter.clone();
            let counter_move = counter.clone();
            let counter_up = counter.clone();

            let on_down = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      down: crate::action::PointerDownCx| {
                    if down.button != MouseButton::Left {
                        return false;
                    }
                    host.capture_pointer();
                    let _ = host
                        .models_mut()
                        .update(&counter_down, |v: &mut u32| *v = v.saturating_add(1));
                    host.request_redraw(cx.window);
                    true
                },
            );

            let on_move = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      _cx: crate::action::ActionCx,
                      _mv: crate::action::PointerMoveCx| {
                    let _ = host
                        .models_mut()
                        .update(&counter_move, |v: &mut u32| *v = v.saturating_add(10));
                    true
                },
            );

            let on_up = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      up: crate::action::PointerUpCx| {
                    if up.button == MouseButton::Left {
                        host.release_pointer_capture();
                    }
                    let _ = host
                        .models_mut()
                        .update(&counter_up, |v: &mut u32| *v = v.saturating_add(100));
                    host.request_redraw(cx.window);
                    true
                },
            );

            let mut props = crate::element::PointerRegionProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(on_down);
                cx.pointer_region_on_pointer_move(on_move);
                cx.pointer_region_on_pointer_up(on_up);
                Vec::new()
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region = ui.children(root)[0];
    let region_bounds = ui.debug_node_bounds(region).expect("pointer region bounds");

    let inside = Point::new(
        Px(region_bounds.origin.x.0 + 5.0),
        Px(region_bounds.origin.y.0 + 5.0),
    );
    let outside = Point::new(Px(region_bounds.origin.x.0 + 250.0), inside.y);

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
            position: outside,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: outside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&counter), Some(111));
}

#[test]
fn declarative_pointer_region_pointer_down_runs_when_descendant_pressable_stops_bubble() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let counter = app.models_mut().insert(0u32);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-down-with-pressable-child",
        |cx| {
            let counter = counter.clone();
            let on_down = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      down: crate::action::PointerDownCx| {
                    if down.button != MouseButton::Left {
                        return false;
                    }
                    let _ = host
                        .models_mut()
                        .update(&counter, |v: &mut u32| *v = v.saturating_add(1));
                    host.request_redraw(cx.window);
                    false
                },
            );

            let mut region_props = crate::element::PointerRegionProps::default();
            region_props.layout.size.width = Length::Fill;
            region_props.layout.size.height = Length::Fill;

            let mut pressable_props = crate::element::PressableProps::default();
            pressable_props.layout.size.width = Length::Fill;
            pressable_props.layout.size.height = Length::Fill;

            vec![cx.pointer_region(region_props, move |cx| {
                cx.pointer_region_on_pointer_down(on_down);
                vec![cx.pressable(pressable_props, |cx, _st| vec![cx.text("child")])]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region = ui.children(root)[0];
    let region_bounds = ui.debug_node_bounds(region).expect("pointer region bounds");
    let inside = Point::new(
        Px(region_bounds.origin.x.0 + 5.0),
        Px(region_bounds.origin.y.0 + 5.0),
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

    assert_eq!(app.models().get_copied(&counter), Some(1));
}

#[test]
fn declarative_pointer_region_can_handle_pointer_cancel() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let counter = app.models_mut().insert(0u32);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-pointer-cancel",
        |cx| {
            let counter_down = counter.clone();
            let counter_cancel = counter.clone();

            let on_down = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      down: crate::action::PointerDownCx| {
                    if down.button != MouseButton::Left {
                        return false;
                    }
                    host.capture_pointer();
                    let _ = host
                        .models_mut()
                        .update(&counter_down, |v: &mut u32| *v = v.saturating_add(1));
                    host.request_redraw(cx.window);
                    true
                },
            );

            let on_cancel = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      cancel: crate::action::PointerCancelCx| {
                    host.release_pointer_capture();
                    let _ = host.models_mut().update(&counter_cancel, |v: &mut u32| {
                        *v = v.saturating_add(match cancel.reason {
                            fret_core::PointerCancelReason::LeftWindow => 100,
                        })
                    });
                    host.request_redraw(cx.window);
                    true
                },
            );

            let mut props = crate::element::PointerRegionProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(on_down);
                cx.pointer_region_on_pointer_cancel(on_cancel);
                Vec::new()
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region = ui.children(root)[0];
    let region_bounds = ui.debug_node_bounds(region).expect("pointer region bounds");
    let inside = Point::new(
        Px(region_bounds.origin.x.0 + 5.0),
        Px(region_bounds.origin.y.0 + 5.0),
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

    assert_eq!(ui.captured_for(fret_core::PointerId(0)), Some(region));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: fret_core::PointerId(0),
            position: None,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );

    assert_eq!(ui.captured_for(fret_core::PointerId(0)), None);
    assert_eq!(app.models().get_copied(&counter), Some(101));
}

#[test]
fn declarative_pointer_region_can_handle_wheel() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let counter = app.models_mut().insert(0u32);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-wheel",
        |cx| {
            let counter_wheel = counter.clone();
            let on_wheel = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      _wheel: crate::action::WheelCx| {
                    let _ = host
                        .models_mut()
                        .update(&counter_wheel, |v: &mut u32| *v = v.saturating_add(1));
                    host.request_redraw(cx.window);
                    true
                },
            );

            let mut props = crate::element::PointerRegionProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            vec![cx.pointer_region(props, |cx| {
                cx.pointer_region_on_wheel(on_wheel);
                Vec::new()
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let inside = Point::new(Px(10.0), Px(10.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: inside,
            delta: Point::new(Px(0.0), Px(10.0)),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let v = app.models_mut().read(&counter, |v| *v).unwrap_or_default();
    assert_eq!(v, 1);
}

#[test]
fn declarative_pointer_region_can_handle_pinch_gesture() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let counter = app.models_mut().insert(0u32);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-pinch",
        |cx| {
            let counter_pinch = counter.clone();
            let on_pinch = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      _pinch: crate::action::PinchGestureCx| {
                    let _ = host
                        .models_mut()
                        .update(&counter_pinch, |v: &mut u32| *v = v.saturating_add(1));
                    host.request_redraw(cx.window);
                    true
                },
            );

            let mut props = crate::element::PointerRegionProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            vec![cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pinch_gesture(on_pinch);
                Vec::new()
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let inside = Point::new(Px(10.0), Px(10.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::PinchGesture {
            position: inside,
            delta: 0.5,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let v = app.models_mut().read(&counter, |v| *v).unwrap_or_default();
    assert_eq!(v, 1);
}

#[test]
fn declarative_internal_drag_region_can_handle_internal_drag_events() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut services = FakeTextService::default();

    let counter = app.models_mut().insert(0u32);
    let drag_kind = fret_runtime::DragKindId(0x465245545F494452); // "FRET_IDR"

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "internal-drag-region-basic",
        |cx| {
            let counter = counter.clone();
            let mut props = crate::element::InternalDragRegionProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            vec![cx.internal_drag_region(props, |cx| {
                cx.internal_drag_region_on_internal_drag(Arc::new(
                    move |host: &mut dyn crate::action::UiDragActionHost,
                          acx: crate::action::ActionCx,
                          drag: crate::action::InternalDragCx| {
                        let Some(session) = host.drag(drag.pointer_id) else {
                            return false;
                        };
                        if session.kind != drag_kind {
                            return false;
                        }
                        if drag.kind == fret_core::InternalDragKind::Over {
                            let _ = host
                                .models_mut()
                                .update(&counter, |v: &mut u32| *v = v.saturating_add(1));
                            host.request_redraw(acx.window);
                            return true;
                        }
                        false
                    },
                ));
                vec![cx.text("drop target")]
            })]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        drag_kind,
        window,
        Point::new(Px(4.0), Px(4.0)),
        (),
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::InternalDrag(fret_core::InternalDragEvent {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(10.0)),
            kind: fret_core::InternalDragKind::Over,
            modifiers: Modifiers::default(),
        }),
    );

    let value = app.models_mut().read(&counter, |v| *v).unwrap_or_default();
    assert_eq!(value, 1);
}

#[test]
fn declarative_command_availability_hooks_participate_in_dispatch_path_queries() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "command-availability-hooks",
        |cx| {
            vec![
                cx.container(crate::element::ContainerProps::default(), |cx| {
                    let id = cx.root_id();
                    cx.command_on_command_availability_for(
                        id,
                        Arc::new(|_host, acx, command| {
                            if command.as_str() != "edit.copy" {
                                return crate::widget::CommandAvailability::NotHandled;
                            }
                            if !acx.focus_in_subtree {
                                return crate::widget::CommandAvailability::NotHandled;
                            }
                            crate::widget::CommandAvailability::Available
                        }),
                    );
                    vec![cx.text("child")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let child_node = ui.children(container_node)[0];
    ui.set_focus(Some(child_node));

    let copy = CommandId::from("edit.copy");
    assert!(
        ui.is_command_available(&mut app, &copy),
        "expected edit.copy to be available via declarative availability hook"
    );

    ui.set_focus(None);
    assert!(
        !ui.is_command_available(&mut app, &copy),
        "expected edit.copy to be unavailable when no dispatch path exists"
    );
}

#[test]
fn declarative_pointer_region_hook_can_request_focus_for_other_element() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-can-request-focus-other-element",
        |cx| {
            vec![cx.semantics(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Slider,
                    label: Some(Arc::from("focus-target")),
                    ..Default::default()
                },
                |cx| {
                    let target = cx.root_id();

                    vec![cx.pointer_region(
                        crate::element::PointerRegionProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: crate::element::Length::Fill,
                                    height: crate::element::Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            enabled: true,
                        },
                        |cx| {
                            cx.pointer_region_on_pointer_down(Arc::new(move |host, _cx, down| {
                                if down.button != MouseButton::Left {
                                    return false;
                                }
                                host.request_focus(target);
                                true
                            }));
                            vec![]
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let semantics_node = ui.children(root)[0];
    let pointer_node = ui.children(semantics_node)[0];
    let pointer_bounds = ui.debug_node_bounds(pointer_node).expect("pointer bounds");
    let position = Point::new(
        Px(pointer_bounds.origin.x.0 + 2.0),
        Px(pointer_bounds.origin.y.0 + 2.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.focus(), Some(semantics_node));
}
