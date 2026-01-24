#![allow(clippy::arc_with_non_send_sync)]

use super::*;

#[test]
fn keyed_elements_reuse_node_ids_across_reorder() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut text = FakeTextService::default();

    let mut items: Vec<u64> = vec![1, 2, 3];
    let mut ids: Vec<(u64, crate::elements::GlobalElementId)> = Vec::new();

    let mut prev: std::collections::HashMap<
        u64,
        (crate::elements::GlobalElementId, fret_core::NodeId),
    > = std::collections::HashMap::new();

    let mut root: Option<fret_core::NodeId> = None;

    for pass in 0..2 {
        ids.clear();
        let r = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp49",
            |cx| build_keyed_rows(cx, &items, &mut ids),
        );
        root.get_or_insert(r);

        let cur: std::collections::HashMap<
            u64,
            (crate::elements::GlobalElementId, fret_core::NodeId),
        > = app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, app| {
            runtime.prepare_window_for_frame(window, app.frame_id());
            let st = runtime.for_window_mut(window);
            ids.iter()
                .map(|(item, id)| (*item, (*id, st.node_entry(*id).unwrap().node)))
                .collect()
        });

        if pass == 1 {
            for item in [1u64, 2u64, 3u64] {
                let (prev_id, prev_node) = prev.get(&item).copied().unwrap();
                let (cur_id, cur_node) = cur.get(&item).copied().unwrap();
                assert_eq!(
                    prev_id, cur_id,
                    "element id should be stable for item {item}"
                );
                assert_eq!(
                    prev_node, cur_node,
                    "node id should be stable for item {item}"
                );
            }
        }

        prev = cur;
        items.reverse();
        app.advance_frame();
    }

    assert_eq!(ui.children(root.unwrap()).len(), 3);
}

#[test]
fn opacity_element_emits_opacity_stack_ops() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "opacity-element-emits-ops",
        |cx| {
            vec![cx.opacity(0.5, |cx| {
                let mut props = crate::element::ContainerProps::default();
                props.layout.size.width = Length::Fill;
                props.layout.size.height = Length::Fill;
                props.background = Some(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                });
                vec![cx.container(props, |_| Vec::new())]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);
    assert!(matches!(
        scene.ops()[0],
        SceneOp::PushOpacity { opacity } if (opacity - 0.5).abs() < 1e-6
    ));
    assert!(matches!(scene.ops()[1], SceneOp::Quad { .. }));
    assert!(matches!(scene.ops()[2], SceneOp::PopOpacity));
}

#[test]
fn effect_layer_element_emits_effect_stack_ops() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "effect-layer-element-emits-ops",
        |cx| {
            let chain =
                fret_core::EffectChain::from_steps(&[fret_core::EffectStep::Pixelate { scale: 6 }]);
            vec![
                cx.effect_layer(fret_core::EffectMode::FilterContent, chain, |cx| {
                    let mut props = crate::element::ContainerProps::default();
                    props.layout.size.width = Length::Fill;
                    props.layout.size.height = Length::Fill;
                    props.background = Some(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    });
                    vec![cx.container(props, |_| Vec::new())]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);
    assert!(matches!(
        scene.ops()[0],
        SceneOp::PushEffect {
            mode: fret_core::EffectMode::FilterContent,
            ..
        }
    ));
    assert!(matches!(scene.ops()[1], SceneOp::Quad { .. }));
    assert!(matches!(scene.ops()[2], SceneOp::PopEffect));
}

#[test]
fn visual_transform_element_emits_transform_stack_ops() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "visual-transform-element-emits-ops",
        |cx| {
            vec![cx.visual_transform(
                Transform2D::translation(Point::new(Px(10.0), Px(0.0))),
                |cx| {
                    let mut props = crate::element::ContainerProps::default();
                    props.layout.size.width = Length::Fill;
                    props.layout.size.height = Length::Fill;
                    props.background = Some(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    });
                    vec![cx.container(props, |_| Vec::new())]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);
    assert!(matches!(
        scene.ops()[0],
        SceneOp::PushTransform { transform } if (transform.tx - 10.0).abs() < 1e-6
    ));
    assert!(matches!(scene.ops()[1], SceneOp::Quad { .. }));
    assert!(matches!(scene.ops()[2], SceneOp::PopTransform));
}

#[test]
fn visual_transform_does_not_affect_hit_testing() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "visual-transform-hit-test",
        |cx| {
            let transform = Transform2D::translation(Point::new(Px(50.0), Px(0.0)));
            vec![cx.visual_transform(transform, |cx| {
                let mut props = crate::element::PressableProps::default();
                props.layout.size.width = Length::Px(Px(20.0));
                props.layout.size.height = Length::Px(Px(20.0));
                vec![cx.pressable(props, |_cx, _state| Vec::new())]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let transform_node = ui.children(root)[0];
    let pressable_node = ui.children(transform_node)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");

    let original_hit_pos = Point::new(
        Px(pressable_bounds.origin.x.0 + 2.0),
        Px(pressable_bounds.origin.y.0 + 2.0),
    );
    assert_eq!(
        ui.debug_hit_test(original_hit_pos).hit,
        Some(pressable_node)
    );

    let translated_hit_pos = Point::new(
        Px(pressable_bounds.origin.x.0 + 52.0),
        Px(pressable_bounds.origin.y.0 + 2.0),
    );
    assert_ne!(
        ui.debug_hit_test(translated_hit_pos).hit,
        Some(pressable_node)
    );
}

#[test]
fn render_transform_affects_hit_testing() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "render-transform-hit-test",
        |cx| {
            let transform = Transform2D::translation(Point::new(Px(50.0), Px(0.0)));
            vec![cx.render_transform(transform, |cx| {
                let mut props = crate::element::PressableProps::default();
                props.layout.size.width = Length::Px(Px(20.0));
                props.layout.size.height = Length::Px(Px(20.0));
                vec![cx.pressable(props, |_cx, _state| Vec::new())]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let transform_node = ui.children(root)[0];
    let pressable_node = ui.children(transform_node)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");

    let original_hit_pos = Point::new(
        Px(pressable_bounds.origin.x.0 + 2.0),
        Px(pressable_bounds.origin.y.0 + 2.0),
    );
    assert_ne!(
        ui.debug_hit_test(original_hit_pos).hit,
        Some(pressable_node)
    );

    let translated_hit_pos = Point::new(
        Px(pressable_bounds.origin.x.0 + 52.0),
        Px(pressable_bounds.origin.y.0 + 2.0),
    );
    assert_eq!(
        ui.debug_hit_test(translated_hit_pos).hit,
        Some(pressable_node)
    );
}

#[test]
fn key_hook_runs_for_focused_text_input() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let value = app.models_mut().insert(String::new());
    let invoked = app.models_mut().insert(0u32);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "key-hook-text-input",
        |cx| {
            let mut props = TextInputProps::new(value);
            props.layout.size.width = Length::Px(Px(160.0));
            props.layout.size.height = Length::Px(Px(32.0));
            let input = cx.text_input(props);

            let invoked = invoked.clone();
            cx.key_on_key_down_for(
                input.id,
                Arc::new(move |host, _cx, down| {
                    if down.repeat {
                        return false;
                    }
                    if down.key != fret_core::KeyCode::ArrowDown {
                        return false;
                    }
                    let _ = host.models_mut().update(&invoked, |v: &mut u32| *v += 1);
                    true
                }),
            );

            vec![input]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let input_node = ui.children(root)[0];
    let input_bounds = ui.debug_node_bounds(input_node).expect("input bounds");
    let pos = Point::new(
        Px(input_bounds.origin.x.0 + 2.0),
        Px(input_bounds.origin.y.0 + 2.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(app.models().get_copied(&invoked).unwrap_or_default(), 1);
}

#[test]
fn key_hook_can_request_focus() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let mut first_id: Option<crate::elements::GlobalElementId> = None;
    let mut second_id: Option<crate::elements::GlobalElementId> = None;

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "key-hook-focus",
        |cx| {
            let mut props = crate::element::PressableProps::default();
            props.layout.size.width = Length::Px(Px(80.0));
            props.layout.size.height = Length::Px(Px(32.0));
            props.focusable = true;

            let first = cx.pressable_with_id(props, |_cx, _st, id| {
                first_id = Some(id);
                Vec::new()
            });

            let mut props2 = crate::element::PressableProps::default();
            props2.layout.size.width = Length::Px(Px(80.0));
            props2.layout.size.height = Length::Px(Px(32.0));
            props2.focusable = true;

            let second = cx.pressable_with_id(props2, |_cx, _st, id| {
                second_id = Some(id);
                Vec::new()
            });

            let second_target = second.id;
            cx.key_on_key_down_for(
                first.id,
                Arc::new(move |host, _cx, down| {
                    if down.repeat || down.key != fret_core::KeyCode::ArrowRight {
                        return false;
                    }
                    host.request_focus(second_target);
                    true
                }),
            );

            vec![cx.column(crate::element::ColumnProps::default(), |_| {
                vec![first, second]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let first_id = first_id.expect("first element id");
    let second_id = second_id.expect("second element id");

    let first_node = crate::elements::node_for_element(&mut app, window, first_id).expect("first");
    let second_node =
        crate::elements::node_for_element(&mut app, window, second_id).expect("second");

    let first_bounds = ui.debug_node_bounds(first_node).expect("first bounds");
    let pos = Point::new(
        Px(first_bounds.origin.x.0 + 2.0),
        Px(first_bounds.origin.y.0 + 2.0),
    );

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
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.focus(), Some(first_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(ui.focus(), Some(second_node));
}

#[test]
fn continuous_frames_lease_requests_animation_frames_while_held() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut services = FakeTextService::default();

    let mut lease: Option<ContinuousFrames> = None;

    let _root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |cx| {
            lease = Some(cx.begin_continuous_frames());
            Vec::<AnyElement>::new()
        },
    );

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
        "expected RequestAnimationFrame while beginning a continuous frames lease"
    );

    app.advance_frame();
    let _root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |_cx| Vec::<AnyElement>::new(),
    );

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
        "expected RequestAnimationFrame while continuous frames lease is held"
    );

    drop(lease.take());
    app.advance_frame();
    let _root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |_cx| Vec::<AnyElement>::new(),
    );

    let effects = app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
        "did not expect RequestAnimationFrame after dropping the last continuous frames lease"
    );
}

#[test]
fn stale_nodes_are_swept_after_gc_lag() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut text = FakeTextService::default();

    let mut ids: Vec<(u64, crate::elements::GlobalElementId)> = Vec::new();
    let _root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp49-sweep",
        |cx| build_keyed_rows(cx, &[1u64, 2u64], &mut ids),
    );

    let node_to_remove =
        app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _app| {
            runtime
                .for_window_mut(window)
                .node_entry(ids[1].1)
                .unwrap()
                .node
        });

    // Remove item 2 from the render output, but it should not be swept immediately.
    app.advance_frame();
    let _root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp49-sweep",
        |cx| build_keyed_rows(cx, &[1u64], &mut Vec::new()),
    );
    assert!(ui.debug_node_bounds(node_to_remove).is_some());

    // Advance frames until the GC lag is exceeded, then render again to trigger the sweep.
    app.advance_frame();
    let _ = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp49-sweep",
        |cx| build_keyed_rows(cx, &[1u64], &mut Vec::new()),
    );
    assert!(ui.debug_node_bounds(node_to_remove).is_some());

    app.advance_frame();
    let _ = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp49-sweep",
        |cx| build_keyed_rows(cx, &[1u64], &mut Vec::new()),
    );
    assert!(ui.debug_node_bounds(node_to_remove).is_none());
}

#[test]
fn dismissible_root_recreates_nodes_after_layer_removal() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut text = FakeTextService::default();

    let base_root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "base",
        |_cx| Vec::<AnyElement>::new(),
    );
    ui.set_root(base_root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "overlay-root",
        |cx| vec![cx.text("overlay")],
    );
    let layer = ui.push_overlay_root(overlay_root, true);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let _ = ui.remove_layer(&mut text, layer);
    assert!(ui.debug_node_bounds(overlay_root).is_none());

    app.advance_frame();
    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "overlay-root",
        |cx| vec![cx.text("overlay")],
    );
    let _layer = ui.push_overlay_root(overlay_root, true);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
}
