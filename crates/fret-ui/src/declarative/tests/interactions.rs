#![allow(clippy::arc_with_non_send_sync)]

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
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&counter), Some(111));
}

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
                },
                |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0),
                            ..Default::default()
                        },
                        |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();
                            out.push(cx.selectable_text(fret_core::RichText::new(
                                "hello selectable text",
                                Arc::<[fret_core::TextRun]>::from([]),
                            )));
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
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.focus(), Some(semantics_node));
}

#[test]
fn dismissible_layer_pointer_move_observer_does_not_break_click_through() {
    struct CountPointerDown {
        clicks: fret_runtime::Model<u32>,
    }

    impl<H: UiHost> Widget<H> for CountPointerDown {
        fn hit_test(&self, bounds: Rect, position: Point) -> bool {
            bounds.contains(position)
        }

        fn event(&mut self, cx: &mut crate::widget::EventCx<'_, H>, event: &fret_core::Event) {
            if matches!(
                event,
                fret_core::Event::Pointer(fret_core::PointerEvent::Down { .. })
            ) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.clicks, |v: &mut u32| *v = v.saturating_add(1));
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let clicks = app.models_mut().insert(0u32);
    let base = ui.create_node(CountPointerDown {
        clicks: clicks.clone(),
    });
    ui.set_root(base);

    let moves = app.models_mut().insert(0u32);
    let moves_for_hook = moves.clone();
    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-pointer-move-observer",
        move |cx| {
            cx.dismissible_on_pointer_move(Arc::new(move |host, _acx, _mv| {
                let _ = host
                    .models_mut()
                    .update(&moves_for_hook, |v: &mut u32| *v = v.saturating_add(1));
                false
            }));
            Vec::new()
        },
    );
    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_visible(layer, true);
    ui.set_layer_hit_testable(layer, true);
    ui.set_layer_wants_pointer_move_events(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let p = Point::new(Px(10.0), Px(10.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: p,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&moves), Some(1));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: p,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        app.models().get_copied(&clicks),
        Some(1),
        "expected click-through dispatch to reach the underlay"
    );
}

#[test]
fn declarative_resizable_panel_group_updates_model_on_drag() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(40.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(vec![0.33, 0.34, 0.33]);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "resizable-panel-group-drag",
        |cx| {
            let mut props = crate::element::ResizablePanelGroupProps::new(
                fret_core::Axis::Horizontal,
                model.clone(),
            );
            props.min_px = vec![Px(10.0)];
            props.chrome = crate::ResizablePanelGroupStyle {
                hit_thickness: Px(10.0),
                ..Default::default()
            };
            vec![cx.resizable_panel_group(props, |cx| {
                vec![
                    cx.spacer(crate::element::SpacerProps::default()),
                    cx.spacer(crate::element::SpacerProps::default()),
                    cx.spacer(crate::element::SpacerProps::default()),
                ]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let fractions_now = app.models().get_cloned(&model).unwrap_or_default();
    let layout = crate::resizable_panel_group::compute_resizable_panel_group_layout(
        fret_core::Axis::Horizontal,
        bounds,
        3,
        fractions_now,
        Px(0.0),
        Px(10.0),
        &[Px(10.0)],
    );
    let down_x = layout.handle_centers.first().copied().unwrap_or(0.0);
    let down = Point::new(Px(down_x), Px(20.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(128.0), Px(20.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(128.0), Px(20.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let v = app.models().get_cloned(&model).unwrap_or_default();
    assert!(
        v.first().copied().unwrap_or(0.0) > 0.33,
        "expected left panel to grow, got {v:?}"
    );
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
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&activated), Some(true));
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
fn dismissible_on_dismiss_request_hook_runs_on_escape() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let dismissed = app.models_mut().insert(false);

    let base_root = ui.create_node(FillStack);
    ui.set_root(base_root);

    let overlay_root = super::super::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-hook-escape",
        |cx| {
            let dismissed = dismissed.clone();
            cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, reason| {
                assert_eq!(reason, DismissReason::Escape);
                let _ = host
                    .models_mut()
                    .update(&dismissed, |v: &mut bool| *v = true);
            }));

            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _| {
                    vec![cx.text("child")]
                }),
            ]
        },
    );

    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_visible(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Focus a descendant in the overlay so Escape bubbles up to the dismissible layer.
    let focused = ui.children(overlay_root)[0];
    ui.set_focus(Some(focused));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(app.models().get_copied(&dismissed), Some(true));
}

#[test]
fn dismissible_on_dismiss_request_hook_runs_on_outside_press_observer() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let dismissed = app.models_mut().insert(false);

    // Base root provides a hit-test target so the pointer down is "outside" the overlay.
    let base_root = ui.create_node(FillStack);
    ui.set_root(base_root);

    let overlay_root = super::super::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-hook-outside-press",
        |cx| {
            let dismissed = dismissed.clone();
            cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, reason| {
                assert_eq!(reason, DismissReason::OutsidePress);
                let _ = host
                    .models_mut()
                    .update(&dismissed, |v: &mut bool| *v = true);
            }));
            Vec::new()
        },
    );

    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);
    ui.set_layer_visible(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Pointer down hits the base root (overlay has no children and is hit-test transparent),
    // so outside-press observer dispatch runs for the overlay root.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(2.0), Px(2.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&dismissed), Some(true));
}

#[test]
fn roving_flex_arrow_keys_move_focus_and_update_selection() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let model = app
        .models_mut()
        .insert(Option::<Arc<str>>::Some(Arc::from("a")));
    let values: Arc<[Arc<str>]> = Arc::from([Arc::from("a"), Arc::from("b"), Arc::from("c")]);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "roving-flex",
        |cx| {
            let props = crate::element::RovingFlexProps {
                flex: crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                roving: crate::element::RovingFocusProps {
                    enabled: true,
                    wrap: true,
                    disabled: Arc::from([false, true, false]),
                },
            };

            vec![cx.roving_flex(props, |cx| {
                let values = values.clone();
                let model = model.clone();
                cx.roving_on_navigate(Arc::new(|_host, _cx, it| {
                    use crate::action::RovingNavigateResult;
                    use fret_core::KeyCode;

                    let Some(current) = it.current else {
                        return RovingNavigateResult::NotHandled;
                    };

                    let forward = match it.key {
                        KeyCode::ArrowDown => true,
                        KeyCode::ArrowUp => false,
                        _ => return RovingNavigateResult::NotHandled,
                    };

                    let len = it.len;
                    let is_disabled =
                        |idx: usize| -> bool { it.disabled.get(idx).copied().unwrap_or(false) };

                    let mut target: Option<usize> = None;
                    if it.wrap {
                        for step in 1..=len {
                            let idx = if forward {
                                (current + step) % len
                            } else {
                                (current + len - (step % len)) % len
                            };
                            if !is_disabled(idx) {
                                target = Some(idx);
                                break;
                            }
                        }
                    } else if forward {
                        target = ((current + 1)..len).find(|&i| !is_disabled(i));
                    } else if current > 0 {
                        target = (0..current).rev().find(|&i| !is_disabled(i));
                    }

                    RovingNavigateResult::Handled { target }
                }));
                cx.roving_on_active_change(Arc::new(move |host, _cx, idx| {
                    let Some(value) = values.get(idx).cloned() else {
                        return;
                    };
                    let next = Some(value);
                    let _ = host
                        .models_mut()
                        .update(&model, |v: &mut Option<Arc<str>>| *v = next);
                }));

                let mut make = |label: &'static str| {
                    cx.pressable(
                        crate::element::PressableProps::default(),
                        |child_cx, _st| vec![child_cx.text(label)],
                    )
                };
                vec![make("a"), make("b"), make("c")]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let roving = ui.children(root)[0];
    let a = ui.children(roving)[0];
    let c = ui.children(roving)[2];
    ui.set_focus(Some(a));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(
        ui.focus(),
        Some(c),
        "expected ArrowDown to skip disabled child"
    );
    assert_eq!(
        app.models().get_cloned(&model).flatten().as_deref(),
        Some("c"),
    );
}

#[test]
fn roving_flex_typeahead_hook_can_choose_target_index() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

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
        "roving-flex-typeahead-hook",
        |cx| {
            let props = crate::element::RovingFlexProps {
                flex: crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                roving: crate::element::RovingFocusProps {
                    enabled: true,
                    wrap: true,
                    disabled: Arc::from([false, false, false]),
                },
            };

            vec![cx.roving_flex(props, |cx| {
                cx.roving_on_typeahead(Arc::new(
                    |_host, _cx, it| {
                        if it.input == 'c' { Some(2) } else { None }
                    },
                ));

                let mut make = |label: &'static str| {
                    cx.pressable(
                        crate::element::PressableProps::default(),
                        |child_cx, _st| vec![child_cx.text(label)],
                    )
                };
                vec![make("a"), make("b"), make("c")]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let roving = ui.children(root)[0];
    let a = ui.children(roving)[0];
    let c = ui.children(roving)[2];
    ui.set_focus(Some(a));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::KeyC,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(ui.focus(), Some(c));
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
