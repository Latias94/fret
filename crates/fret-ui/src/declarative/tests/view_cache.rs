use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn view_cache_skips_child_render_when_clean_and_preserves_element_state() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let renders = Arc::new(AtomicUsize::new(0));
    let leaf_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));

    let mut root: Option<NodeId> = None;

    for frame in 0..6 {
        let renders = renders.clone();
        let leaf_id = leaf_id.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "view-cache-reuse",
            move |cx| {
                vec![
                    cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);

                        let leaf = cx.text("leaf");
                        *leaf_id.lock().unwrap() = Some(leaf.id);

                        cx.with_state_for(leaf.id, || 123u32, |_| {});

                        vec![leaf]
                    }),
                ]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        app.advance_frame();
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "view cache should prevent re-running child render when clean"
    );

    let leaf = leaf_id.lock().unwrap().expect("leaf id should be recorded");
    let value = crate::elements::with_element_state(&mut app, window, leaf, || 0u32, |v| *v);
    assert_eq!(value, 123, "element state should survive cache-hit frames");

    #[cfg(feature = "diagnostics")]
    {
        let debug_path = app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
            runtime.debug_path_for_element(window, leaf)
        });
        assert!(
            debug_path.is_some(),
            "debug identity should survive cache-hit frames"
        );
    }
}

#[test]
fn view_cache_preserves_pressable_activate_handler_across_cache_hit_frames() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let renders = Arc::new(AtomicUsize::new(0));
    let invoked = app.models_mut().insert(0u32);

    let pressable_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));

    let mut root: Option<NodeId> = None;
    for frame in 0..2 {
        let renders = renders.clone();
        let invoked = invoked.clone();
        let pressable_id_for_render = pressable_id.clone();

        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "view-cache-pressable-activate",
            move |cx| {
                vec![
                    cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);

                        let mut props = crate::element::PressableProps::default();
                        props.layout.size.width = Length::Px(Px(120.0));
                        props.layout.size.height = Length::Px(Px(32.0));
                        props.focusable = true;

                        let invoked = invoked.clone();
                        let pressable = cx.pressable_with_id(props, move |cx, _st, id| {
                            *pressable_id_for_render.lock().unwrap() = Some(id);
                            cx.pressable_on_activate_for(
                                id,
                                Arc::new(move |host, _cx, _reason| {
                                    let _ = host
                                        .models_mut()
                                        .update(&invoked, |v: &mut u32| *v = v.saturating_add(1));
                                }),
                            );
                            vec![cx.text("pressable")]
                        });

                        vec![pressable]
                    }),
                ]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        if frame == 1 {
            let pressable_element = pressable_id
                .lock()
                .unwrap()
                .expect("pressable element id should be recorded on first frame");
            let pressable_node =
                crate::elements::node_for_element(&mut app, window, pressable_element)
                    .expect("pressable node");
            let pressable_bounds = ui
                .debug_node_bounds(pressable_node)
                .expect("pressable bounds");
            let pos = Point::new(
                Px(pressable_bounds.origin.x.0 + 2.0),
                Px(pressable_bounds.origin.y.0 + 2.0),
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
                    click_count: 1,
                    pointer_id: fret_core::PointerId(0),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );
        }

        app.advance_frame();
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "view cache should prevent re-running child render when clean"
    );
    assert_eq!(
        app.models().get_copied(&invoked).unwrap_or_default(),
        1,
        "pressable activate hook should remain wired on cache-hit frames"
    );
}

#[test]
fn view_cache_preserves_key_hooks_across_cache_hit_frames() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let invoked = app.models_mut().insert(0u32);
    let renders = Arc::new(AtomicUsize::new(0));
    let focusable_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let mut focusable_node: Option<NodeId> = None;

    for frame in 0..2 {
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "view-cache-key-hook",
            |cx| {
                let renders = renders.clone();
                let invoked = invoked.clone();
                let focusable_id = focusable_id.clone();
                vec![
                    cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);

                        let mut props = crate::element::PressableProps::default();
                        props.layout.size.width = Length::Px(Px(120.0));
                        props.layout.size.height = Length::Px(Px(32.0));
                        props.focusable = true;

                        let widget = cx.pressable_with_id(props, move |cx, _st, id| {
                            *focusable_id.lock().unwrap() = Some(id);
                            vec![cx.text("focusable")]
                        });

                        cx.key_on_key_down_for(
                            widget.id,
                            Arc::new(move |host, _cx, down| {
                                if down.repeat || down.key != fret_core::KeyCode::ArrowDown {
                                    return false;
                                }
                                let _ = host.models_mut().update(&invoked, |v: &mut u32| *v += 1);
                                true
                            }),
                        );

                        vec![widget]
                    }),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        if frame == 0 {
            let focusable_element = focusable_id
                .lock()
                .unwrap()
                .expect("focusable element id should be recorded");
            let node = crate::elements::node_for_element(&mut app, window, focusable_element)
                .expect("focusable node");
            focusable_node = Some(node);

            let b = ui.debug_node_bounds(node).expect("focusable bounds");
            let pos = Point::new(Px(b.origin.x.0 + 2.0), Px(b.origin.y.0 + 2.0));

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
                    click_count: 1,
                    pointer_id: fret_core::PointerId(0),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );

            assert_eq!(ui.focus(), Some(node));

            scene.clear();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        } else {
            ui.dispatch_event(
                &mut app,
                &mut services,
                &fret_core::Event::KeyDown {
                    key: fret_core::KeyCode::ArrowDown,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
        }

        app.advance_frame();
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "view cache should prevent re-running child render when clean"
    );
    assert_eq!(ui.focus(), focusable_node);
    assert_eq!(app.models().get_copied(&invoked).unwrap_or_default(), 1);
}
