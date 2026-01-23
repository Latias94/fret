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
fn view_cache_cache_hit_produces_same_scene_ops_as_uncached_frame() {
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    // Cached run: render once, then ensure the cache-hit frame paints the same scene ops without
    // re-running the child render closure.
    let mut cached_app = TestHost::new();
    let mut cached_ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    cached_ui.set_window(window);
    cached_ui.set_view_cache_enabled(true);
    let mut cached_services = FakeTextService::default();

    let renders = Arc::new(AtomicUsize::new(0));
    let mut cached_root: Option<NodeId> = None;
    let mut cached_hit_ops: Option<Vec<String>> = None;

    for frame in 0..2 {
        let renders = renders.clone();
        let root_node = render_root(
            &mut cached_ui,
            &mut cached_app,
            &mut cached_services,
            window,
            bounds,
            "view-cache-scene-eq",
            move |cx| {
                vec![
                    cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);
                        vec![cx.text("hello"), cx.text("world")]
                    }),
                ]
            },
        );

        cached_root.get_or_insert(root_node);
        if frame == 0 {
            cached_ui.set_root(root_node);
        }

        cached_ui.layout_all(&mut cached_app, &mut cached_services, bounds, 1.0);
        let mut scene = Scene::default();
        cached_ui.paint_all(
            &mut cached_app,
            &mut cached_services,
            bounds,
            &mut scene,
            1.0,
        );

        if frame == 1 {
            cached_hit_ops = Some(scene.ops().iter().map(|op| format!("{op:?}")).collect());
        }

        cached_app.advance_frame();
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "expected the cache-hit frame to skip the child render closure"
    );

    // Uncached run: a single frame should paint the same scene ops as the cache-hit frame.
    let mut uncached_app = TestHost::new();
    let mut uncached_ui: UiTree<TestHost> = UiTree::new();
    uncached_ui.set_window(window);
    let mut uncached_services = FakeTextService::default();

    let root_node = render_root(
        &mut uncached_ui,
        &mut uncached_app,
        &mut uncached_services,
        window,
        bounds,
        "view-cache-scene-eq",
        |cx| {
            vec![
                cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                    vec![cx.text("hello"), cx.text("world")]
                }),
            ]
        },
    );
    uncached_ui.set_root(root_node);
    uncached_ui.layout_all(&mut uncached_app, &mut uncached_services, bounds, 1.0);
    let mut scene = Scene::default();
    uncached_ui.paint_all(
        &mut uncached_app,
        &mut uncached_services,
        bounds,
        &mut scene,
        1.0,
    );

    assert_eq!(
        cached_hit_ops.expect("expected cache-hit ops to be recorded"),
        scene
            .ops()
            .iter()
            .map(|op| format!("{op:?}"))
            .collect::<Vec<_>>(),
        "cache-hit frames should be behaviorally equivalent to uncached frames (scene ops match)"
    );
}

#[test]
fn view_cache_cache_hit_produces_same_semantics_and_hit_targets_as_uncached_frame() {
    fn projection(
        snap: &fret_core::SemanticsSnapshot,
    ) -> Vec<(
        String,
        fret_core::SemanticsRole,
        Rect,
        fret_core::SemanticsFlags,
        fret_core::SemanticsActions,
        Option<String>,
    )> {
        let mut out: Vec<_> = snap
            .nodes
            .iter()
            .filter_map(|n| {
                n.test_id.as_ref().map(|test_id| {
                    (
                        test_id.clone(),
                        n.role,
                        n.bounds,
                        n.flags,
                        n.actions,
                        n.label.clone(),
                    )
                })
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    fn hit_test_id<'a>(snap: &'a fret_core::SemanticsSnapshot, node: NodeId) -> Option<&'a str> {
        let mut current = Some(node);
        while let Some(id) = current {
            let n = snap.nodes.iter().find(|n| n.id == id)?;
            if let Some(test_id) = n.test_id.as_deref() {
                return Some(test_id);
            }
            current = n.parent;
        }
        None
    }

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut props = crate::element::PressableProps::default();
    props.layout.size.width = Length::Px(Px(40.0));
    props.layout.size.height = Length::Px(Px(20.0));
    props.a11y.role = Some(fret_core::SemanticsRole::Button);
    props.a11y.label = Some(Arc::from("Button"));
    props.a11y.test_id = Some(Arc::from("btn"));

    // Cached run: render once, then on the cache-hit frame assert that semantics + hit targets
    // remain stable and match the uncached behavior.
    let mut cached_app = TestHost::new();
    let mut cached_ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    cached_ui.set_window(window);
    cached_ui.set_view_cache_enabled(true);
    cached_ui.set_debug_enabled(true);
    let mut cached_services = FakeTextService::default();

    let renders = Arc::new(AtomicUsize::new(0));
    let mut cached_root: Option<NodeId> = None;
    let mut cached_projection: Option<
        Vec<(
            String,
            fret_core::SemanticsRole,
            Rect,
            fret_core::SemanticsFlags,
            fret_core::SemanticsActions,
            Option<String>,
        )>,
    > = None;
    let mut cached_hit: Option<Option<String>> = None;

    for frame in 0..2 {
        let renders = renders.clone();
        let props = props.clone();
        let root_node = render_root(
            &mut cached_ui,
            &mut cached_app,
            &mut cached_services,
            window,
            bounds,
            "view-cache-semantics-hit-eq",
            move |cx| {
                vec![
                    cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);
                        vec![cx.pressable(props, |_cx, _state| Vec::new())]
                    }),
                ]
            },
        );

        cached_root.get_or_insert(root_node);
        if frame == 0 {
            cached_ui.set_root(root_node);
        }

        cached_ui.request_semantics_snapshot();
        cached_ui.layout_all(&mut cached_app, &mut cached_services, bounds, 1.0);

        let snap = cached_ui.semantics_snapshot().expect("semantics snapshot");
        let btn_bounds = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("btn"))
            .map(|n| n.bounds)
            .expect("expected semantics node for btn");

        let hit_pos = fret_core::Point::new(
            Px(btn_bounds.origin.x.0 + 2.0),
            Px(btn_bounds.origin.y.0 + 2.0),
        );
        let hit = cached_ui.debug_hit_test(hit_pos).hit;
        let hit = hit.and_then(|hit| hit_test_id(snap, hit).map(str::to_string));

        if frame == 1 {
            cached_projection = Some(projection(snap));
            cached_hit = Some(hit);
        }

        cached_app.advance_frame();
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "expected the cache-hit frame to skip the child render closure"
    );

    // Uncached run: a single frame should produce the same semantics projection and hit target id.
    let mut uncached_app = TestHost::new();
    let mut uncached_ui: UiTree<TestHost> = UiTree::new();
    uncached_ui.set_window(window);
    uncached_ui.set_view_cache_enabled(false);
    uncached_ui.set_debug_enabled(true);
    let mut uncached_services = FakeTextService::default();

    let root_node = render_root(
        &mut uncached_ui,
        &mut uncached_app,
        &mut uncached_services,
        window,
        bounds,
        "view-cache-semantics-hit-eq",
        |cx| {
            vec![
                cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                    vec![cx.pressable(props, |_cx, _state| Vec::new())]
                }),
            ]
        },
    );
    uncached_ui.set_root(root_node);
    uncached_ui.request_semantics_snapshot();
    uncached_ui.layout_all(&mut uncached_app, &mut uncached_services, bounds, 1.0);

    let snap = uncached_ui
        .semantics_snapshot()
        .expect("semantics snapshot");
    let btn_bounds = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("btn"))
        .map(|n| n.bounds)
        .expect("expected semantics node for btn");
    let hit_pos = fret_core::Point::new(
        Px(btn_bounds.origin.x.0 + 2.0),
        Px(btn_bounds.origin.y.0 + 2.0),
    );
    let hit = uncached_ui.debug_hit_test(hit_pos).hit;
    let hit = hit.and_then(|hit| hit_test_id(snap, hit).map(str::to_string));

    assert_eq!(
        cached_projection.expect("expected cached projection"),
        projection(snap),
        "cache-hit frames should preserve semantics output (projection matches uncached frame)"
    );
    assert_eq!(
        cached_hit.expect("expected cached hit"),
        hit,
        "cache-hit frames should preserve high-level hit targets (resolved by semantics test_id)"
    );
}

#[test]
fn view_cache_modal_overlay_preserves_semantics_and_input_gating_on_cache_hit_frames() {
    fn projection(
        snap: &fret_core::SemanticsSnapshot,
    ) -> Vec<(
        String,
        fret_core::SemanticsRole,
        Rect,
        fret_core::SemanticsFlags,
        fret_core::SemanticsActions,
        Option<String>,
    )> {
        let mut out: Vec<_> = snap
            .nodes
            .iter()
            .filter_map(|n| {
                n.test_id.as_ref().map(|test_id| {
                    (
                        test_id.clone(),
                        n.role,
                        n.bounds,
                        n.flags,
                        n.actions,
                        n.label.clone(),
                    )
                })
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    fn hit_test_id<'a>(snap: &'a fret_core::SemanticsSnapshot, node: NodeId) -> Option<&'a str> {
        let mut current = Some(node);
        while let Some(id) = current {
            let n = snap.nodes.iter().find(|n| n.id == id)?;
            if let Some(test_id) = n.test_id.as_deref() {
                return Some(test_id);
            }
            current = n.parent;
        }
        None
    }

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    fn underlay_pressable(props: &mut crate::element::PressableProps) {
        props.layout.position = crate::element::PositionStyle::Absolute;
        props.layout.inset.left = Some(Px(10.0));
        props.layout.inset.top = Some(Px(10.0));
        props.layout.size.width = Length::Px(Px(40.0));
        props.layout.size.height = Length::Px(Px(20.0));
        props.a11y.role = Some(fret_core::SemanticsRole::Button);
        props.a11y.label = Some(Arc::from("Underlay"));
        props.a11y.test_id = Some(Arc::from("underlay_btn"));
    }

    fn overlay_pressable(props: &mut crate::element::PressableProps) {
        props.layout.position = crate::element::PositionStyle::Absolute;
        props.layout.inset.left = Some(Px(10.0));
        props.layout.inset.top = Some(Px(60.0));
        props.layout.size.width = Length::Px(Px(40.0));
        props.layout.size.height = Length::Px(Px(20.0));
        props.a11y.role = Some(fret_core::SemanticsRole::Button);
        props.a11y.label = Some(Arc::from("Overlay"));
        props.a11y.test_id = Some(Arc::from("overlay_btn"));
    }

    fn render_base(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        window: AppWindowId,
        bounds: Rect,
        renders: Arc<AtomicUsize>,
        underlay_activated: fret_runtime::Model<bool>,
    ) -> NodeId {
        render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "view-cache-modal-overlay-eq",
            move |cx| {
                let mut view_cache = crate::element::ViewCacheProps::default();
                view_cache.layout.size.width = Length::Fill;
                view_cache.layout.size.height = Length::Fill;

                vec![cx.view_cache(view_cache, |cx| {
                    renders.fetch_add(1, Ordering::SeqCst);

                    let mut props = crate::element::PressableProps::default();
                    underlay_pressable(&mut props);

                    vec![cx.pressable(props, |cx, _state| {
                        let underlay_activated = underlay_activated.clone();
                        cx.pressable_on_activate(Arc::new(move |host, _cx, _reason| {
                            let _ = host
                                .models_mut()
                                .update(&underlay_activated, |v: &mut bool| *v = true);
                        }));
                        Vec::new()
                    })]
                })]
            },
        )
    }

    fn render_overlay(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        window: AppWindowId,
        bounds: Rect,
        overlay_activated: fret_runtime::Model<bool>,
    ) -> NodeId {
        crate::declarative::render_dismissible_root_with_hooks(
            ui,
            app,
            services,
            window,
            bounds,
            "view-cache-modal-overlay",
            move |cx| {
                let mut props = crate::element::PressableProps::default();
                overlay_pressable(&mut props);

                vec![cx.pressable(props, |cx, _state| {
                    let overlay_activated = overlay_activated.clone();
                    cx.pressable_on_activate(Arc::new(move |host, _cx, _reason| {
                        let _ = host
                            .models_mut()
                            .update(&overlay_activated, |v: &mut bool| *v = true);
                    }));
                    Vec::new()
                })]
            },
        )
    }

    // Cached run: frame 0 mounts everything, frame 1 must be a cache hit for the base view-cache
    // root while a modal overlay layer blocks underlay input.
    let mut cached_app = TestHost::new();
    let mut cached_ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    cached_ui.set_window(window);
    cached_ui.set_view_cache_enabled(true);
    cached_ui.set_debug_enabled(true);
    let mut cached_services = FakeTextService::default();

    let underlay_activated = cached_app.models_mut().insert(false);
    let overlay_activated = cached_app.models_mut().insert(false);
    let renders = Arc::new(AtomicUsize::new(0));

    // Frame 0.
    let root0 = render_base(
        &mut cached_ui,
        &mut cached_app,
        &mut cached_services,
        window,
        bounds,
        renders.clone(),
        underlay_activated.clone(),
    );
    cached_ui.set_root(root0);

    let overlay_root0 = render_overlay(
        &mut cached_ui,
        &mut cached_app,
        &mut cached_services,
        window,
        bounds,
        overlay_activated.clone(),
    );

    let overlay_layer = cached_ui.push_overlay_root_ex(overlay_root0, true, true);
    cached_ui.set_layer_visible(overlay_layer, true);

    cached_ui.request_semantics_snapshot();
    cached_ui.layout_all(&mut cached_app, &mut cached_services, bounds, 1.0);
    cached_app.advance_frame();

    // Frame 1 (expected cache hit for base view-cache root).
    let _root1 = render_base(
        &mut cached_ui,
        &mut cached_app,
        &mut cached_services,
        window,
        bounds,
        renders.clone(),
        underlay_activated.clone(),
    );

    let overlay_root1 = render_overlay(
        &mut cached_ui,
        &mut cached_app,
        &mut cached_services,
        window,
        bounds,
        overlay_activated.clone(),
    );

    assert_eq!(
        overlay_root0, overlay_root1,
        "overlay root should be stable"
    );
    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "expected the base view-cache child render closure to run only once (cache hit frame skips it)"
    );

    cached_ui.request_semantics_snapshot();
    cached_ui.layout_all(&mut cached_app, &mut cached_services, bounds, 1.0);
    let (cached_projection, underlay_pos, overlay_pos, hit_underlay, hit_overlay) = {
        let cached_snap = cached_ui.semantics_snapshot().expect("semantics snapshot");

        let underlay_bounds = cached_snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay_btn"))
            .map(|n| n.bounds)
            .expect("expected semantics node for underlay_btn");
        let overlay_bounds = cached_snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("overlay_btn"))
            .map(|n| n.bounds)
            .expect("expected semantics node for overlay_btn");

        let underlay_pos = Point::new(
            Px(underlay_bounds.origin.x.0 + 2.0),
            Px(underlay_bounds.origin.y.0 + 2.0),
        );
        let overlay_pos = Point::new(
            Px(overlay_bounds.origin.x.0 + 2.0),
            Px(overlay_bounds.origin.y.0 + 2.0),
        );

        let hit_underlay = cached_ui
            .debug_hit_test(underlay_pos)
            .hit
            .and_then(|hit| hit_test_id(cached_snap, hit).map(str::to_string));
        let hit_overlay = cached_ui
            .debug_hit_test(overlay_pos)
            .hit
            .and_then(|hit| hit_test_id(cached_snap, hit).map(str::to_string));

        (
            projection(cached_snap),
            underlay_pos,
            overlay_pos,
            hit_underlay,
            hit_overlay,
        )
    };

    assert_eq!(
        cached_ui
            .debug_hit_test(Point::new(Px(0.0), Px(0.0)))
            .barrier_root,
        Some(overlay_root0),
        "expected the modal overlay layer to become the barrier root"
    );
    assert!(
        cached_ui
            .debug_hit_test(Point::new(Px(0.0), Px(0.0)))
            .active_layer_roots
            .contains(&overlay_root0),
        "expected active hit-test roots to include the modal overlay root"
    );

    assert_eq!(
        hit_underlay, None,
        "modal overlay should block underlay hit-testing even when the underlay remains in the semantics snapshot"
    );

    assert_eq!(hit_overlay.as_deref(), Some("overlay_btn"));

    assert_eq!(
        cached_app.models().get_copied(&underlay_activated),
        Some(false)
    );
    assert_eq!(
        cached_app.models().get_copied(&overlay_activated),
        Some(false)
    );

    // Click underlay: should be blocked by the modal barrier.
    cached_ui.dispatch_event(
        &mut cached_app,
        &mut cached_services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: underlay_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    cached_ui.dispatch_event(
        &mut cached_app,
        &mut cached_services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: underlay_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        cached_app.models().get_copied(&underlay_activated),
        Some(false)
    );
    assert_eq!(
        cached_app.models().get_copied(&overlay_activated),
        Some(false)
    );

    // Click overlay: should activate even on a cache-hit frame.
    cached_ui.dispatch_event(
        &mut cached_app,
        &mut cached_services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: overlay_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    cached_ui.dispatch_event(
        &mut cached_app,
        &mut cached_services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: overlay_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        cached_app.models().get_copied(&underlay_activated),
        Some(false)
    );
    assert_eq!(
        cached_app.models().get_copied(&overlay_activated),
        Some(true)
    );

    // Uncached run: single frame should match the same observable semantics + input gating.
    let mut uncached_app = TestHost::new();
    let mut uncached_ui: UiTree<TestHost> = UiTree::new();
    uncached_ui.set_window(window);
    uncached_ui.set_view_cache_enabled(false);
    uncached_ui.set_debug_enabled(true);
    let mut uncached_services = FakeTextService::default();

    let underlay_activated = uncached_app.models_mut().insert(false);
    let overlay_activated = uncached_app.models_mut().insert(false);

    let root = {
        let underlay_activated = underlay_activated.clone();
        render_root(
            &mut uncached_ui,
            &mut uncached_app,
            &mut uncached_services,
            window,
            bounds,
            "view-cache-modal-overlay-eq",
            move |cx| {
                let mut view_cache = crate::element::ViewCacheProps::default();
                view_cache.layout.size.width = Length::Fill;
                view_cache.layout.size.height = Length::Fill;

                vec![cx.view_cache(view_cache, |cx| {
                    let mut props = crate::element::PressableProps::default();
                    underlay_pressable(&mut props);
                    vec![cx.pressable(props, |cx, _state| {
                        let underlay_activated = underlay_activated.clone();
                        cx.pressable_on_activate(Arc::new(move |host, _cx, _reason| {
                            let _ = host
                                .models_mut()
                                .update(&underlay_activated, |v: &mut bool| *v = true);
                        }));
                        Vec::new()
                    })]
                })]
            },
        )
    };
    uncached_ui.set_root(root);

    let overlay_root = {
        let overlay_activated = overlay_activated.clone();
        crate::declarative::render_dismissible_root_with_hooks(
            &mut uncached_ui,
            &mut uncached_app,
            &mut uncached_services,
            window,
            bounds,
            "view-cache-modal-overlay",
            move |cx| {
                let mut props = crate::element::PressableProps::default();
                overlay_pressable(&mut props);
                vec![cx.pressable(props, |cx, _state| {
                    let overlay_activated = overlay_activated.clone();
                    cx.pressable_on_activate(Arc::new(move |host, _cx, _reason| {
                        let _ = host
                            .models_mut()
                            .update(&overlay_activated, |v: &mut bool| *v = true);
                    }));
                    Vec::new()
                })]
            },
        )
    };

    let overlay_layer = uncached_ui.push_overlay_root_ex(overlay_root, true, true);
    uncached_ui.set_layer_visible(overlay_layer, true);

    uncached_ui.request_semantics_snapshot();
    uncached_ui.layout_all(&mut uncached_app, &mut uncached_services, bounds, 1.0);

    let (uncached_projection, underlay_pos, overlay_pos, barrier_root, hit_underlay, hit_overlay) = {
        let snap = uncached_ui
            .semantics_snapshot()
            .expect("semantics snapshot");

        let underlay_bounds = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay_btn"))
            .map(|n| n.bounds)
            .expect("expected semantics node for underlay_btn");
        let overlay_bounds = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("overlay_btn"))
            .map(|n| n.bounds)
            .expect("expected semantics node for overlay_btn");
        let underlay_pos = Point::new(
            Px(underlay_bounds.origin.x.0 + 2.0),
            Px(underlay_bounds.origin.y.0 + 2.0),
        );
        let overlay_pos = Point::new(
            Px(overlay_bounds.origin.x.0 + 2.0),
            Px(overlay_bounds.origin.y.0 + 2.0),
        );

        let hit_underlay = uncached_ui
            .debug_hit_test(underlay_pos)
            .hit
            .and_then(|hit| hit_test_id(snap, hit).map(str::to_string));
        let hit_overlay = uncached_ui
            .debug_hit_test(overlay_pos)
            .hit
            .and_then(|hit| hit_test_id(snap, hit).map(str::to_string));

        (
            projection(snap),
            underlay_pos,
            overlay_pos,
            snap.barrier_root,
            hit_underlay,
            hit_overlay,
        )
    };

    assert_eq!(barrier_root, Some(overlay_root));
    assert_eq!(hit_underlay, None);
    assert_eq!(hit_overlay.as_deref(), Some("overlay_btn"));

    uncached_ui.dispatch_event(
        &mut uncached_app,
        &mut uncached_services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: underlay_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    uncached_ui.dispatch_event(
        &mut uncached_app,
        &mut uncached_services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: underlay_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        uncached_app.models().get_copied(&underlay_activated),
        Some(false)
    );
    assert_eq!(
        uncached_app.models().get_copied(&overlay_activated),
        Some(false)
    );

    uncached_ui.dispatch_event(
        &mut uncached_app,
        &mut uncached_services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: overlay_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    uncached_ui.dispatch_event(
        &mut uncached_app,
        &mut uncached_services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: overlay_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        uncached_app.models().get_copied(&underlay_activated),
        Some(false)
    );
    assert_eq!(
        uncached_app.models().get_copied(&overlay_activated),
        Some(true)
    );

    assert_eq!(
        cached_projection, uncached_projection,
        "cache-hit frames should preserve semantics output under modal overlays"
    );
}
