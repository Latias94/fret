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
fn pointer_down_payload_marks_hit_is_text_input_for_text_input_region_descendants() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let seen = app.models_mut().insert(false);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-down-hit-is-text-input",
        |cx| {
            let seen = seen.clone();
            let on_down = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      down: crate::action::PointerDownCx| {
                    let _ = host
                        .models_mut()
                        .update(&seen, |v: &mut bool| *v = down.hit_is_text_input);
                    host.request_redraw(cx.window);
                    false
                },
            );

            let mut region_props = crate::element::PointerRegionProps::default();
            region_props.layout.size.width = Length::Fill;
            region_props.layout.size.height = Length::Fill;

            let mut text_props = crate::element::TextInputRegionProps::default();
            text_props.layout.size.width = Length::Fill;
            text_props.layout.size.height = Length::Fill;

            vec![cx.pointer_region(region_props, move |cx| {
                cx.pointer_region_on_pointer_down(on_down);
                vec![cx.text_input_region(text_props, |_cx| Vec::<AnyElement>::new())]
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

    assert_eq!(app.models().get_copied(&seen), Some(true));
}

#[test]
fn pointer_down_payload_marks_hit_is_pressable_for_pressable_descendants() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let seen = app.models_mut().insert(false);
    let seen_target = app.models_mut().insert(None::<crate::GlobalElementId>);
    let seen_descendant = app.models_mut().insert(None::<bool>);
    let pressable_id = app.models_mut().insert(None::<crate::GlobalElementId>);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-down-hit-is-pressable",
        |cx| {
            let seen = seen.clone();
            let seen_target = seen_target.clone();
            let seen_descendant = seen_descendant.clone();
            let on_down = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      down: crate::action::PointerDownCx| {
                    let _ = host
                        .models_mut()
                        .update(&seen, |v: &mut bool| *v = down.hit_is_pressable);
                    let _ = host.models_mut().update(&seen_target, |v| {
                        *v = down.hit_pressable_target;
                    });
                    let _ = host.models_mut().update(&seen_descendant, |v| {
                        *v = Some(down.hit_pressable_target_in_descendant_subtree);
                    });
                    host.request_redraw(cx.window);
                    false
                },
            );

            let mut region_props = crate::element::PointerRegionProps::default();
            region_props.layout.size.width = Length::Fill;
            region_props.layout.size.height = Length::Fill;

            let pressable_id = pressable_id.clone();
            vec![cx.pointer_region(region_props, move |cx| {
                cx.pointer_region_on_pointer_down(on_down);
                vec![cx.pressable_with_id(
                    crate::element::PressableProps::default(),
                    move |cx, _state, id| {
                        let _ = cx.app.models_mut().update(&pressable_id, |v| *v = Some(id));
                        vec![cx.text("press")]
                    },
                )]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable = ui.children(ui.children(root)[0])[0];
    let pressable_el = app
        .models()
        .get_copied(&pressable_id)
        .flatten()
        .expect("pressable element id");
    let pressable_bounds = ui.debug_node_bounds(pressable).expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + pressable_bounds.size.width.0 * 0.5),
        Px(pressable_bounds.origin.y.0 + pressable_bounds.size.height.0 * 0.5),
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

    assert_eq!(app.models().get_copied(&seen), Some(true));
    assert_eq!(
        app.models().get_cloned(&seen_target),
        Some(Some(pressable_el)),
        "expected pointer down payload to report the deepest pressable hit target"
    );
    assert_eq!(
        app.models().get_cloned(&seen_descendant),
        Some(Some(true)),
        "expected pointer down payload to mark descendant pressable hits"
    );
}

#[test]
fn pointer_up_payload_exposes_pressable_hit_target_from_pointer_down() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let seen_target = app.models_mut().insert(None::<crate::GlobalElementId>);
    let seen_descendant = app.models_mut().insert(None::<bool>);
    let pressable_id = app.models_mut().insert(None::<crate::GlobalElementId>);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-up-down-hit-pressable-target",
        |cx| {
            let seen_target = seen_target.clone();
            let seen_descendant = seen_descendant.clone();
            let on_down = Arc::new(
                move |_host: &mut dyn crate::action::UiPointerActionHost,
                      _cx: crate::action::ActionCx,
                      _down: crate::action::PointerDownCx| { false },
            );
            let on_up = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      up: crate::action::PointerUpCx| {
                    let _ = host.models_mut().update(&seen_target, |v| {
                        *v = up.down_hit_pressable_target;
                    });
                    let _ = host.models_mut().update(&seen_descendant, |v| {
                        *v = Some(up.down_hit_pressable_target_in_descendant_subtree);
                    });
                    host.request_redraw(cx.window);
                    false
                },
            );

            let mut region_props = crate::element::PointerRegionProps::default();
            region_props.layout.size.width = Length::Fill;
            region_props.layout.size.height = Length::Fill;

            let pressable_id = pressable_id.clone();
            vec![cx.pointer_region(region_props, move |cx| {
                cx.pointer_region_on_pointer_down(on_down);
                cx.pointer_region_on_pointer_up(on_up);
                vec![cx.pressable_with_id(
                    crate::element::PressableProps::default(),
                    move |cx, _state, id| {
                        let _ = cx.app.models_mut().update(&pressable_id, |v| *v = Some(id));
                        vec![cx.text("press")]
                    },
                )]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable = ui.children(ui.children(root)[0])[0];
    let pressable_el = app
        .models()
        .get_copied(&pressable_id)
        .flatten()
        .expect("pressable element id");

    let pressable_bounds = ui.debug_node_bounds(pressable).expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + pressable_bounds.size.width.0 * 0.5),
        Px(pressable_bounds.origin.y.0 + pressable_bounds.size.height.0 * 0.5),
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
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_cloned(&seen_target),
        Some(Some(pressable_el)),
        "expected pointer up payload to carry the deepest pressable hit target from pointer down"
    );
    assert_eq!(
        app.models().get_cloned(&seen_descendant),
        Some(Some(true)),
        "expected pointer up payload to preserve descendant pressable hit classification"
    );
}

#[test]
fn pointer_payload_distinguishes_descendant_from_ancestor_pressable_targets() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(140.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let seen_down_target = app.models_mut().insert(None::<crate::GlobalElementId>);
    let seen_down_descendant = app.models_mut().insert(None::<bool>);
    let seen_up_target = app.models_mut().insert(None::<crate::GlobalElementId>);
    let seen_up_descendant = app.models_mut().insert(None::<bool>);
    let outer_pressable_id = app.models_mut().insert(None::<crate::GlobalElementId>);
    let inner_pressable_id = app.models_mut().insert(None::<crate::GlobalElementId>);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-pressable-hit-relation",
        |cx| {
            let seen_down_target = seen_down_target.clone();
            let seen_down_descendant = seen_down_descendant.clone();
            let seen_up_target = seen_up_target.clone();
            let seen_up_descendant = seen_up_descendant.clone();
            let outer_pressable_id = outer_pressable_id.clone();
            let inner_pressable_id = inner_pressable_id.clone();

            let on_down = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      down: crate::action::PointerDownCx| {
                    host.capture_pointer();
                    let _ = host.models_mut().update(&seen_down_target, |v| {
                        *v = down.hit_pressable_target;
                    });
                    let _ = host.models_mut().update(&seen_down_descendant, |v| {
                        *v = Some(down.hit_pressable_target_in_descendant_subtree);
                    });
                    host.request_redraw(cx.window);
                    false
                },
            );
            let on_up = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      up: crate::action::PointerUpCx| {
                    host.release_pointer_capture();
                    let _ = host.models_mut().update(&seen_up_target, |v| {
                        *v = up.down_hit_pressable_target;
                    });
                    let _ = host.models_mut().update(&seen_up_descendant, |v| {
                        *v = Some(up.down_hit_pressable_target_in_descendant_subtree);
                    });
                    host.request_redraw(cx.window);
                    false
                },
            );

            let mut outer_props = crate::element::PressableProps::default();
            outer_props.layout.size.width = Length::Fill;
            outer_props.layout.size.height = Length::Fill;

            let mut region_props = crate::element::PointerRegionProps::default();
            region_props.layout.size.width = Length::Fill;
            region_props.layout.size.height = Length::Fill;

            vec![
                cx.pressable_with_id(outer_props, move |cx, _state, outer_id| {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&outer_pressable_id, |v| *v = Some(outer_id));
                    vec![cx.pointer_region(region_props, move |cx| {
                        cx.pointer_region_on_pointer_down(on_down.clone());
                        cx.pointer_region_on_pointer_up(on_up.clone());
                        vec![cx.pressable_with_id(
                            crate::element::PressableProps::default(),
                            move |cx, _state, inner_id| {
                                let _ = cx
                                    .app
                                    .models_mut()
                                    .update(&inner_pressable_id, |v| *v = Some(inner_id));
                                vec![cx.text("inner")]
                            },
                        )]
                    })]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let outer_pressable = ui.children(root)[0];
    let pointer_region = ui.children(outer_pressable)[0];
    let inner_pressable = ui.children(pointer_region)[0];

    let outer_pressable_el = app
        .models()
        .get_copied(&outer_pressable_id)
        .flatten()
        .expect("outer pressable element id");
    let inner_pressable_el = app
        .models()
        .get_copied(&inner_pressable_id)
        .flatten()
        .expect("inner pressable element id");

    let center_of = |bounds: fret_core::Rect| {
        Point::new(
            Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
            Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
        )
    };

    let pointer_region_bounds = ui
        .debug_node_bounds(pointer_region)
        .expect("pointer region bounds");
    let inner_pressable_bounds = ui
        .debug_node_bounds(inner_pressable)
        .expect("inner pressable bounds");

    let nested_position = center_of(inner_pressable_bounds);
    let y_mid = Px(pointer_region_bounds.origin.y.0 + pointer_region_bounds.size.height.0 * 0.5);
    let x0 = pointer_region_bounds.origin.x.0;
    let w = pointer_region_bounds.size.width.0;
    let ancestor_position = [
        Point::new(Px(x0 + 4.0), y_mid),
        Point::new(Px(x0 + w * 0.5), y_mid),
        Point::new(Px(x0 + w - 4.0), y_mid),
    ]
    .into_iter()
    .find(|p| pointer_region_bounds.contains(*p) && !inner_pressable_bounds.contains(*p))
    .expect("expected a point inside the pointer region but outside the nested pressable");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: nested_position,
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
            position: nested_position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_cloned(&seen_down_target),
        Some(Some(inner_pressable_el))
    );
    assert_eq!(
        app.models().get_cloned(&seen_down_descendant),
        Some(Some(true))
    );
    assert_eq!(
        app.models().get_cloned(&seen_up_target),
        Some(Some(inner_pressable_el))
    );
    assert_eq!(
        app.models().get_cloned(&seen_up_descendant),
        Some(Some(true))
    );

    let _ = app
        .models_mut()
        .update(&seen_down_target, |v| *v = None::<crate::GlobalElementId>);
    let _ = app
        .models_mut()
        .update(&seen_down_descendant, |v| *v = None::<bool>);
    let _ = app
        .models_mut()
        .update(&seen_up_target, |v| *v = None::<crate::GlobalElementId>);
    let _ = app
        .models_mut()
        .update(&seen_up_descendant, |v| *v = None::<bool>);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: ancestor_position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: ancestor_position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_cloned(&seen_down_target),
        Some(Some(outer_pressable_el)),
        "expected pointer down to preserve the ambient ancestor pressable target"
    );
    assert_eq!(
        app.models().get_cloned(&seen_down_descendant),
        Some(Some(false)),
        "expected pointer down to exclude ancestor pressables from descendant classification"
    );
    assert_eq!(
        app.models().get_cloned(&seen_up_target),
        Some(Some(outer_pressable_el)),
        "expected pointer up to preserve the ambient ancestor pressable target"
    );
    assert_eq!(
        app.models().get_cloned(&seen_up_descendant),
        Some(Some(false)),
        "expected pointer up to exclude ancestor pressables from descendant classification"
    );
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
fn internal_drag_after_raw_rebuild_does_not_route_to_detached_stale_frame_region() {
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
        "internal-drag-region-raw-rebuild",
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
                vec![cx.text("stale drop target")]
            })]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let stale_region = ui.children(root)[0];
    let stale_record_present_before_rebuild =
        crate::declarative::with_window_frame(&mut app, window, |window_frame| {
            let window_frame = window_frame?;
            let record = window_frame.instances.get(stale_region)?;
            Some(matches!(
                record.instance,
                crate::declarative::ElementInstance::InternalDragRegion(_)
            ))
        })
        .unwrap_or(false);
    assert!(
        stale_record_present_before_rebuild,
        "expected the baseline internal-drag region to be recorded in the window frame"
    );

    let replacement_root = ui.create_node(FillStack);
    ui.set_root(replacement_root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let stale_record_present_after_rebuild =
        crate::declarative::with_window_frame(&mut app, window, |window_frame| {
            let window_frame = window_frame?;
            let record = window_frame.instances.get(stale_region)?;
            Some(matches!(
                record.instance,
                crate::declarative::ElementInstance::InternalDragRegion(_)
            ))
        })
        .unwrap_or(false);
    assert!(
        stale_record_present_after_rebuild,
        "raw rebuild audit requires the previous window-frame record to remain until a declarative refresh"
    );

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
    assert_eq!(
        value, 0,
        "detached stale window-frame records must not hijack internal-drag routing after a raw rebuild"
    );
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
                            ..Default::default()
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

#[test]
fn declarative_pointer_region_focus_request_ignores_stale_detached_node_entry() {
    use crate::elements::NodeEntry;

    #[derive(Default)]
    struct DetachedDummy;

    impl<H: UiHost> Widget<H> for DetachedDummy {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

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

    let target_id: std::rc::Rc<std::cell::Cell<Option<crate::elements::GlobalElementId>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));
    let target_id_for_children = target_id.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-focus-stale-node-entry",
        |cx| {
            vec![cx.semantics(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Slider,
                    label: Some(Arc::from("focus-target")),
                    ..Default::default()
                },
                |cx| {
                    let target = cx.root_id();
                    target_id_for_children.set(Some(target));

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
                            ..Default::default()
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

    let target_id = target_id.get().expect("target element id");
    let target_node =
        crate::elements::node_for_element(&mut app, window, target_id).expect("target node");
    let stale_detached = ui.create_node_for_element(target_id, DetachedDummy);

    let frame_id = app.frame_id();
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            target_id,
            NodeEntry {
                node: stale_detached,
                last_seen_frame: frame_id,
                root: target_id,
            },
        );
    });

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

    assert_eq!(ui.focus(), Some(target_node));
}

#[test]
fn declarative_pointer_region_capture_phase_pointer_moves_do_not_double_dispatch() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let moves = app.models_mut().insert(0u32);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-capture-phase-moves-no-double-dispatch",
        |cx| {
            let on_move = Arc::new({
                let moves = moves.clone();
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      _mv: crate::action::PointerMoveCx| {
                    let _ = host
                        .models_mut()
                        .update(&moves, |v: &mut u32| *v = v.saturating_add(1));
                    host.request_redraw(cx.window);
                    false
                }
            });

            vec![cx.pointer_region(
                crate::element::PointerRegionProps {
                    layout: {
                        let mut layout = crate::element::LayoutStyle::default();
                        layout.size.width = crate::element::Length::Fill;
                        layout.size.height = crate::element::Length::Fill;
                        layout
                    },
                    enabled: true,
                    capture_phase_pointer_moves: true,
                },
                move |cx| {
                    cx.pointer_region_on_pointer_move(on_move);
                    Vec::new()
                },
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&moves), Some(1));
}
