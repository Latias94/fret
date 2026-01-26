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
fn request_animation_frame_marks_view_cache_root_dirty() {
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
    let mut root: Option<NodeId> = None;

    for frame in 0..2 {
        let renders = renders.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "view-cache-animation-frame",
            move |cx| {
                vec![
                    cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);
                        cx.request_animation_frame();
                        vec![cx.text("leaf")]
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
        2,
        "request_animation_frame should mark the nearest view-cache root dirty (disable reuse)"
    );
}

#[test]
fn view_cache_gates_reuse_on_explicit_cache_key() {
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
    let mut root: Option<NodeId> = None;

    for frame in 0..3 {
        let renders = renders.clone();
        let cache_key = if frame < 2 { 1u64 } else { 2u64 };
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "view-cache-explicit-key",
            move |cx| {
                vec![cx.view_cache(
                    crate::element::ViewCacheProps {
                        cache_key,
                        ..Default::default()
                    },
                    |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);
                        vec![cx.text(format!("key={cache_key}"))]
                    },
                )]
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
        2,
        "expected cache_key mismatch to force re-running the view-cache child render closure"
    );
}

#[test]
fn view_cache_inherits_model_observations_on_cache_hit_layout() {
    let mut app = TestHost::new();
    let model = app.models_mut().insert(0u32);

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
    let mut root: Option<NodeId> = None;

    fn build_cached(
        cx: &mut ElementContext<'_, TestHost>,
        renders: &Arc<AtomicUsize>,
        model: &fret_runtime::Model<u32>,
    ) -> AnyElement {
        cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
            renders.fetch_add(1, Ordering::SeqCst);
            cx.observe_model(model, Invalidation::Layout);
            let v = cx.app.models().get_copied(model).unwrap_or_default();
            vec![cx.text(format!("Value {v}"))]
        })
    }

    // Frame 0: populate model observations under the view-cache subtree.
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "view-cache-observation-inheritance",
        |cx| vec![build_cached(cx, &renders, &model)],
    );
    root.get_or_insert(root0);
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    app.advance_frame();

    // Frame 1: cache hit (child closure skipped), but the layout pass still runs.
    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "view-cache-observation-inheritance",
        |cx| vec![build_cached(cx, &renders, &model)],
    );
    root.get_or_insert(root1);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "expected cache-hit frame to skip the child render closure"
    );

    // If cache-hit frames do not inherit per-frame model observations, the layout pass above would
    // clear the observation index for the view-cache subtree, and model changes would stop
    // invalidating the cache root.
    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(
        ui.propagate_model_changes(&mut app, &changed),
        "expected model change to invalidate the cached subtree"
    );

    app.advance_frame();

    // Frame 2: invalidation should force re-executing the child render closure.
    let _ = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "view-cache-observation-inheritance",
        |cx| vec![build_cached(cx, &renders, &model)],
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        renders.load(Ordering::SeqCst),
        2,
        "expected invalidation to disable view-cache reuse and re-run the child render closure"
    );
}

#[test]
fn view_cache_does_not_rerender_for_unrelated_model_changes() {
    let mut app = TestHost::new();
    let observed = app.models_mut().insert(0u32);
    let unrelated = app.models_mut().insert(0u32);

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
    let mut root: Option<NodeId> = None;

    fn build_cached(
        cx: &mut ElementContext<'_, TestHost>,
        renders: &Arc<AtomicUsize>,
        observed: &fret_runtime::Model<u32>,
    ) -> AnyElement {
        cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
            renders.fetch_add(1, Ordering::SeqCst);
            cx.observe_model(observed, Invalidation::Layout);
            let v = cx.app.models().get_copied(observed).unwrap_or_default();
            vec![cx.text(format!("Value {v}"))]
        })
    }

    for frame in 0..3 {
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "view-cache-unrelated-model",
            |cx| vec![build_cached(cx, &renders, &observed)],
        );
        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        app.advance_frame();

        if frame == 1 {
            let _ = unrelated.update(&mut app, |v, _cx| *v += 1);
            let changed = app.take_changed_models();
            assert!(
                !ui.propagate_model_changes(&mut app, &changed),
                "unrelated model changes should not invalidate the cached subtree"
            );
        }
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "unrelated model changes should not force re-running the cached subtree render closure"
    );
}

#[test]
fn view_cache_is_disabled_under_inspection() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_inspection_active(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let renders = Arc::new(AtomicUsize::new(0));
    let mut root: Option<NodeId> = None;

    for frame in 0..3 {
        let root_node = {
            let renders = renders.clone();
            render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                "view-cache-inspection-disabled",
                move |cx| {
                    vec![
                        cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                            renders.fetch_add(1, Ordering::SeqCst);
                            vec![cx.text("leaf")]
                        }),
                    ]
                },
            )
        };

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
        3,
        "view cache should be disabled under inspection, forcing subtree execution"
    );
}

#[test]
fn view_cache_matches_non_cached_output_for_stable_frames() {
    fn run(view_cache_enabled: bool) -> (Vec<u64>, Vec<Option<crate::elements::GlobalElementId>>) {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_view_cache_enabled(view_cache_enabled);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );
        let mut services = FakeTextService::default();

        let pressable_id = Arc::new(std::sync::Mutex::new(
            None::<crate::elements::GlobalElementId>,
        ));
        let mut root: Option<NodeId> = None;

        let mut fps: Vec<u64> = Vec::new();
        let mut hits: Vec<Option<crate::elements::GlobalElementId>> = Vec::new();

        for frame in 0..4 {
            let pressable_id = pressable_id.clone();
            let root_node = render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                "view-cache-diff-output",
                move |cx| {
                    vec![
                        cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                            let pressable_id = pressable_id.clone();

                            let mut bg = crate::element::ContainerProps::default();
                            bg.layout.size.width = crate::element::Length::Fill;
                            bg.layout.size.height = crate::element::Length::Fill;
                            bg.background = Some(fret_core::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            });

                            vec![cx.container(bg, move |cx| {
                                let mut props = crate::element::PressableProps::default();
                                props.layout.size.width = crate::element::Length::Fill;
                                props.layout.size.height = crate::element::Length::Fill;
                                props.enabled = true;
                                props.focusable = true;
                                vec![cx.pressable_with_id(props, move |_cx, _st, id| {
                                    *pressable_id.lock().unwrap() = Some(id);
                                    Vec::new()
                                })]
                            })]
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
            fps.push(scene.fingerprint());

            let hit = ui
                .debug_hit_test(fret_core::Point::new(Px(10.0), Px(10.0)))
                .hit
                .and_then(|node| ui.node_element(node));
            hits.push(hit);

            app.advance_frame();
        }

        (fps, hits)
    }

    let (no_cache_fps, no_cache_hits) = run(false);
    let (cache_fps, cache_hits) = run(true);

    assert_eq!(
        cache_fps, no_cache_fps,
        "view cache should not change the recorded scene fingerprint for stable frames"
    );
    assert_eq!(
        cache_hits, no_cache_hits,
        "view cache should not change hit-test outcomes for stable frames"
    );
}
