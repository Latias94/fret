use super::*;

#[test]
fn pressable_state_reports_focused_when_focused() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let focused = Rc::new(Cell::new(false));
    let pressable_element_id: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));

    fn render_frame(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        focused_out: Rc<Cell<bool>>,
        pressable_id_out: Rc<Cell<Option<crate::elements::GlobalElementId>>>,
    ) -> NodeId {
        render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "pressable-state-reports-focused",
            move |cx| {
                let focused_out = focused_out.clone();
                let pressable_id_out = pressable_id_out.clone();
                vec![cx.pressable_with_id(
                    crate::element::PressableProps::default(),
                    move |cx, st, id| {
                        pressable_id_out.set(Some(id));
                        focused_out.set(st.focused);
                        vec![cx.text("pressable")]
                    },
                )]
            },
        )
    }

    // First frame: render once to establish stable identity + node mapping.
    let root = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        focused.clone(),
        pressable_element_id.clone(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(!focused.get());

    let pressable_element = pressable_element_id.get().expect("pressable element id");
    let pressable_node = crate::elements::node_for_element(&mut app, window, pressable_element)
        .expect("pressable node");
    ui.set_focus(Some(pressable_node));

    // Second frame: the authoring context should observe the focused element.
    app.advance_frame();
    let root = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        focused.clone(),
        pressable_element_id.clone(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(focused.get());
}


#[test]
fn pressable_on_activate_hook_runs_on_pointer_activation() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let activated = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-on-activate-hook-pointer",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let activated = activated.clone();
                    cx.pressable_on_activate(Arc::new(move |host, _cx, reason| {
                        assert_eq!(reason, ActivateReason::Pointer);
                        let _ = host
                            .models_mut()
                            .update(&activated, |v: &mut bool| *v = true);
                    }));
                    vec![cx.text("activate")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&activated), Some(false));

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let position = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
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
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&activated), Some(true));
}


#[test]
fn pressable_clears_pressed_and_releases_capture_on_move_without_buttons() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let pressable_element_id_cell: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let pressable_element_id_cell_for_render = pressable_element_id_cell.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clear-pressed-on-move-without-buttons",
        move |cx| {
            let pressable_element_id_cell = pressable_element_id_cell_for_render.clone();
            vec![cx.pressable_with_id(
                crate::element::PressableProps::default(),
                move |cx, _state, id| {
                    pressable_element_id_cell.set(Some(id));
                    vec![cx.text("pressable")]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_element_id = pressable_element_id_cell
        .get()
        .expect("missing pressable element id");
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
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

    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        Some(pressable_node)
    );
    assert!(
        crate::elements::is_pressed_pressable(&mut app, window, pressable_element_id),
        "expected pressable to set pressed state on pointer down"
    );

    // Simulate a runner/platform edge case: we never receive `PointerEvent::Up`, but we do observe
    // that no buttons are pressed anymore.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(inside.x.0 + 10.0), inside.y),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.captured_for(fret_core::PointerId(0)), None);
    assert!(!crate::elements::is_pressed_pressable(
        &mut app,
        window,
        pressable_element_id
    ));
}


#[test]
fn pressable_clears_pressed_state_on_pointer_cancel() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let pressable_element_id_cell: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let pressable_element_id_cell_for_render = pressable_element_id_cell.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clear-pressed-on-pointer-cancel",
        move |cx| {
            let pressable_element_id_cell = pressable_element_id_cell_for_render.clone();
            vec![cx.pressable_with_id(
                crate::element::PressableProps::default(),
                move |cx, _state, id| {
                    pressable_element_id_cell.set(Some(id));
                    vec![cx.text("pressable")]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_element_id = pressable_element_id_cell
        .get()
        .expect("missing pressable element id");
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    let pointer_id = fret_core::PointerId(0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.captured_for(pointer_id), Some(pressable_node));
    assert!(crate::elements::is_pressed_pressable(
        &mut app,
        window,
        pressable_element_id
    ));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id,
            position: Some(inside),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );

    assert_eq!(ui.captured_for(pointer_id), None);
    assert!(!crate::elements::is_pressed_pressable(
        &mut app,
        window,
        pressable_element_id
    ));
}


#[test]
fn pressable_clears_pressed_state_when_element_is_removed() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let pressable_element_id_cell: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let pressable_element_id_cell_for_render = pressable_element_id_cell.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clear-pressed-on-unmount",
        move |cx| {
            let pressable_element_id_cell = pressable_element_id_cell_for_render.clone();
            vec![cx.pressable_with_id(
                crate::element::PressableProps::default(),
                move |cx, _state, id| {
                    pressable_element_id_cell.set(Some(id));
                    vec![cx.text("pressable")]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_element_id = pressable_element_id_cell
        .get()
        .expect("missing pressable element id");
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
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
    assert!(
        crate::elements::is_pressed_pressable(&mut app, window, pressable_element_id),
        "expected pressable to be pressed after pointer down"
    );

    // Drop the pressable element without sending pointer up/cancel events (e.g. overlay closes).
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clear-pressed-on-unmount",
        |_cx| Vec::new(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        !crate::elements::is_pressed_pressable(&mut app, window, pressable_element_id),
        "expected pressed state to clear when the element is removed"
    );
}


#[test]
fn pressable_on_hover_change_hook_runs_on_pointer_move() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let hovered = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-on-hover-change-hook",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let hovered = hovered.clone();
                    cx.pressable_on_hover_change(Arc::new(move |host, _cx, is_hovered| {
                        let _ = host
                            .models_mut()
                            .update(&hovered, |v: &mut bool| *v = is_hovered);
                    }));
                    vec![cx.text("hover me")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&hovered), Some(false));

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: inside,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&hovered), Some(true));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(pressable_bounds.origin.x.0 + 200.0), Px(2.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&hovered), Some(false));
}


#[test]
fn pressable_on_hover_change_hook_runs_after_wheel_scroll_without_pointer_move() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let hovered = app.models_mut().insert(None::<u32>);
    let handle = crate::scroll::ScrollHandle::default();
    let item_h = Px(20.0);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-hover-after-wheel-scroll",
        |cx| {
            let scroll = cx.scroll(
                crate::element::ScrollProps {
                    layout: {
                        let mut layout = crate::element::LayoutStyle::default();
                        layout.size.width = crate::element::Length::Fill;
                        layout.size.height = crate::element::Length::Fill;
                        layout.overflow = crate::element::Overflow::Clip;
                        layout
                    },
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(crate::element::ColumnProps::default(), |cx| {
                        (0..20)
                            .map(|idx| {
                                let hovered = hovered.clone();
                                cx.keyed(idx, move |cx| {
                                    cx.pressable(
                                        crate::element::PressableProps {
                                            layout: {
                                                let mut layout =
                                                    crate::element::LayoutStyle::default();
                                                layout.size.width = crate::element::Length::Fill;
                                                layout.size.height =
                                                    crate::element::Length::Px(item_h);
                                                layout
                                            },
                                            ..Default::default()
                                        },
                                        move |cx, _state| {
                                            cx.pressable_on_hover_change(Arc::new(
                                                move |host, _cx, is_hovered| {
                                                    if !is_hovered {
                                                        return;
                                                    }
                                                    let _ = host
                                                        .models_mut()
                                                        .update(&hovered, |v: &mut Option<u32>| {
                                                            *v = Some(idx as u32)
                                                        });
                                                },
                                            ));
                                            vec![cx.text(format!("Item {idx}"))]
                                        },
                                    )
                                })
                            })
                            .collect::<Vec<_>>()
                    })]
                },
            );
            vec![scroll]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&hovered), Some(None));

    let position = Point::new(Px(10.0), Px(10.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(Some(0)));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            pointer_id: fret_core::PointerId(0),
            position,
            delta: Point::new(Px(0.0), Px(-20.0)),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(Some(1)));
}


#[test]
fn pressable_hover_state_ignores_touch_pointer_moves() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let hovered = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-hover-ignores-touch",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let hovered = hovered.clone();
                    cx.pressable_on_hover_change(Arc::new(move |host, _cx, is_hovered| {
                        let _ = host
                            .models_mut()
                            .update(&hovered, |v: &mut bool| *v = is_hovered);
                    }));
                    vec![cx.text("hover me")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: inside,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(true));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(pressable_bounds.origin.x.0 + 200.0), Px(2.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Touch,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(true));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(pressable_bounds.origin.x.0 + 200.0), Px(2.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(false));
}


#[test]
fn pressable_on_activate_hook_runs_on_keyboard_activation() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let activated = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-on-activate-hook-keyboard",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let activated = activated.clone();
                    cx.pressable_on_activate(Arc::new(move |host, _cx, reason| {
                        assert_eq!(reason, ActivateReason::Keyboard);
                        let _ = host
                            .models_mut()
                            .update(&activated, |v: &mut bool| *v = true);
                    }));
                    vec![cx.text("activate")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&activated), Some(false));

    let pressable_node = ui.children(root)[0];
    ui.set_focus(Some(pressable_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyUp {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
        },
    );

    assert_eq!(app.models().get_copied(&activated), Some(true));
}


#[test]
fn pressable_semantics_checked_is_exposed() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-pressable-checked",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    enabled: true,
                    a11y: crate::element::PressableA11y {
                        role: Some(fret_core::SemanticsRole::Checkbox),
                        label: Some(Arc::from("checked")),
                        checked: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |cx, _state| vec![cx.text("x")],
            )]
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
            n.role == fret_core::SemanticsRole::Checkbox && n.label.as_deref() == Some("checked")
        })
        .expect("expected checkbox semantics node");

    assert_eq!(node.flags.checked, Some(true));
    assert!(node.actions.invoke, "expected checkbox to be invokable");
}

