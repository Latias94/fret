use super::*;

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
    let layer = ui.push_overlay_root(overlay_root, false);
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
            pointer_id: fret_core::PointerId(0),
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
            pointer_id: fret_core::PointerId(0),
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

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-hook-escape",
        |cx| {
            let dismissed = dismissed.clone();
            cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, req| {
                assert_eq!(req.reason, DismissReason::Escape);
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

    let layer = ui.push_overlay_root(overlay_root, false);
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

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-hook-outside-press",
        |cx| {
            let dismissed = dismissed.clone();
            cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, req| {
                match req.reason {
                    DismissReason::OutsidePress { pointer: Some(cx) } => {
                        assert_eq!(cx.pointer_id, fret_core::PointerId(0));
                        assert_eq!(cx.pointer_type, fret_core::PointerType::Mouse);
                        assert_eq!(cx.button, MouseButton::Left);
                        assert_eq!(cx.modifiers, Modifiers::default());
                        assert_eq!(cx.click_count, 1);
                    }
                    other => panic!("expected outside-press dismissal, got {other:?}"),
                }
                let _ = host
                    .models_mut()
                    .update(&dismissed, |v: &mut bool| *v = true);
            }));
            Vec::new()
        },
    );

    let layer = ui.push_overlay_root(overlay_root, false);
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
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&dismissed), Some(true));
}

#[test]
fn dismissible_scroll_dismiss_ignores_stale_detached_node_entry() {
    use crate::elements::NodeEntry;
    use std::cell::Cell;
    use std::rc::Rc;

    struct DetachedDummy;

    impl<H: UiHost> Widget<H> for DetachedDummy {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(Px(0.0), Px(0.0))
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let handle = crate::scroll::ScrollHandle::default();
    let handle_for_ui = handle.clone();
    let scroll_element: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let dismissed = app.models_mut().insert(false);
    let root_name = "dismissible-scroll-dismiss-stale-node-entry";

    let base_root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        {
            let scroll_element = scroll_element.clone();
            move |cx| {
                let scroll = cx.keyed("scroll-root", |cx| {
                    cx.scroll(
                        crate::element::ScrollProps {
                            layout: {
                                let mut layout = crate::element::LayoutStyle::default();
                                layout.size.width = crate::element::Length::Fill;
                                layout.size.height = crate::element::Length::Fill;
                                layout.overflow = crate::element::Overflow::Clip;
                                layout
                            },
                            axis: crate::element::ScrollAxis::Y,
                            scroll_handle: Some(handle_for_ui.clone()),
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.column(crate::element::ColumnProps::default(), |cx| {
                                (0..30)
                                    .map(|idx| {
                                        cx.keyed(idx, |cx| {
                                            cx.container(
                                                crate::element::ContainerProps {
                                                    layout: {
                                                        let mut layout =
                                                            crate::element::LayoutStyle::default();
                                                        layout.size.width =
                                                            crate::element::Length::Fill;
                                                        layout.size.height =
                                                            crate::element::Length::Px(Px(20.0));
                                                        layout
                                                    },
                                                    ..Default::default()
                                                },
                                                move |cx| vec![cx.text(format!("row {idx}"))],
                                            )
                                        })
                                    })
                                    .collect::<Vec<_>>()
                            })]
                        },
                    )
                });
                scroll_element.set(Some(scroll.id));
                vec![scroll]
            }
        },
    );
    ui.set_root(base_root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_element = scroll_element.get().expect("scroll element id");
    let scroll_node =
        crate::declarative::node_for_element_in_window_frame(&mut app, window, scroll_element)
            .expect("live scroll node");
    let scroll_bounds = ui.debug_node_bounds(scroll_node).expect("scroll bounds");

    assert!(
        handle.max_offset().y.0 > 0.01,
        "expected scrollable content (max_offset={:?})",
        handle.max_offset()
    );

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-scroll-dismiss-overlay",
        |cx| {
            let dismissed = dismissed.clone();
            cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, req| {
                assert_eq!(req.reason, DismissReason::Scroll);
                let _ = host
                    .models_mut()
                    .update(&dismissed, |v: &mut bool| *v = true);
            }));
            Vec::new()
        },
    );
    let layer = ui.push_overlay_root(overlay_root, false);
    ui.set_layer_visible(layer, true);
    ui.set_layer_scroll_dismiss_elements(layer, vec![scroll_element]);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let stale_detached = ui.create_node_for_element(scroll_element, DetachedDummy);
    let frame_id = app.frame_id();
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            scroll_element,
            NodeEntry {
                node: stale_detached,
                last_seen_frame: frame_id,
                root: crate::elements::global_root(window, root_name),
            },
        );
    });

    let position = Point::new(
        Px(scroll_bounds.origin.x.0 + 4.0),
        Px(scroll_bounds.origin.y.0 + 4.0),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            pointer_id: fret_core::PointerId(0),
            position,
            delta: Point::new(Px(0.0), Px(-24.0)),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        app.models().get_copied(&dismissed).unwrap_or(false),
        "expected wheel scroll-dismiss lookup to use the live attached scroll element instead of a stale detached node_entry seed"
    );
}

#[test]
fn dismissible_outside_press_prevent_default_keeps_focus() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    let mut services = FakeTextService::default();

    // Base root provides a hit-test target so the pointer down is "outside" the overlay.
    let base_root = ui.create_node(FillStack);
    ui.set_root(base_root);

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-outside-press-prevent-default-keeps-focus",
        |cx| {
            cx.dismissible_on_dismiss_request(Arc::new(move |_host, _cx, req| {
                if matches!(req.reason, DismissReason::OutsidePress { .. }) {
                    req.prevent_default();
                }
            }));

            let mut props = crate::element::PressableProps {
                enabled: true,
                focusable: true,
                ..Default::default()
            };
            props.layout.size.width = crate::element::Length::Px(Px(10.0));
            props.layout.size.height = crate::element::Length::Px(Px(10.0));
            vec![cx.pressable(props, |cx, _| vec![cx.text("child")])]
        },
    );

    let layer = ui.push_overlay_root(overlay_root, false);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);
    ui.set_layer_visible(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let focused = ui.children(overlay_root)[0];
    ui.set_focus(Some(focused));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(190.0), Px(70.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        ui.focus(),
        Some(focused),
        "expected preventDefault outside-press dismissal to keep focus stable"
    );
}

#[test]
fn dismissible_outside_press_without_prevent_default_clears_focus() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    let mut services = FakeTextService::default();

    // Base root provides a hit-test target so the pointer down is "outside" the overlay.
    let base_root = ui.create_node(FillStack);
    ui.set_root(base_root);

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-outside-press-without-prevent-default-clears-focus",
        |cx| {
            cx.dismissible_on_dismiss_request(Arc::new(move |_host, _cx, _req| {}));

            let mut props = crate::element::PressableProps {
                enabled: true,
                focusable: true,
                ..Default::default()
            };
            props.layout.size.width = crate::element::Length::Px(Px(10.0));
            props.layout.size.height = crate::element::Length::Px(Px(10.0));
            vec![cx.pressable(props, |cx, _| vec![cx.text("child")])]
        },
    );

    let layer = ui.push_overlay_root(overlay_root, false);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);
    ui.set_layer_visible(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let focused = ui.children(overlay_root)[0];
    ui.set_focus(Some(focused));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(190.0), Px(70.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        ui.focus(),
        None,
        "expected outside-press default behavior to clear focus when policy does not prevent it"
    );
}
