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
    let mut scene = Scene::default();

    let renders = Arc::new(AtomicUsize::new(0));
    let leaf_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));

    for frame in 0..6 {
        let renders = renders.clone();
        let leaf_id = leaf_id.clone();
        let root_node = render_root_for_frame(
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

                        cx.state_for(leaf.id, || 123u32, |_| {});

                        vec![leaf]
                    }),
                ]
            },
        );

        assert!(
            ui.base_root().is_some(),
            "expected render_root_for_frame to install the root layer for base roots"
        );
        if frame == 0 {
            assert_eq!(ui.base_root(), Some(root_node));
        }

        layout_frame(&mut ui, &mut app, &mut services, bounds);
        paint_frame(&mut ui, &mut app, &mut services, bounds, &mut scene);

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
fn view_cache_reuse_preserves_scope_only_authoring_identity_liveness() {
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
    let owner_identity = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));

    for _frame in 0..4 {
        let renders = renders.clone();
        let owner_identity = owner_identity.clone();
        let owner_identity_for_render = owner_identity.clone();
        let root_node = render_root_for_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "view-cache-scope-authoring-identity-liveness",
            move |cx| {
                vec![
                    cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);
                        vec![cx.keyed("owner", |cx| {
                            *owner_identity_for_render.lock().unwrap() = Some(cx.root_id());
                            cx.text("owner")
                        })]
                    }),
                ]
            },
        );

        ui.set_root(root_node);
        let owner_identity = owner_identity
            .lock()
            .unwrap()
            .expect("owner identity should be recorded");
        assert_eq!(
            crate::elements::live_node_for_element(&mut app, window, owner_identity),
            None,
            "scope-only identity should not require a mounted node"
        );
        assert!(
            crate::elements::element_identity_is_live_in_current_frame(
                &mut app,
                window,
                owner_identity,
            ),
            "view-cache reuse should keep scope-only authoring identities live for the current frame"
        );

        layout_frame(&mut ui, &mut app, &mut services, bounds);
        app.advance_frame();
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "clean view-cache reuse should avoid rerendering the scoped owner subtree"
    );
}

#[test]
fn view_cache_preserves_selectable_text_interactive_span_bounds() {
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
    let mut scene = Scene::default();

    let leaf_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));

    for frame in 0..6 {
        let leaf_id = leaf_id.clone();
        let root_node = render_root_for_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "view-cache-selectable-text-span-bounds",
            move |cx| {
                vec![
                    cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                        let text = Arc::<str>::from("hello link world");
                        let mut props = crate::element::SelectableTextProps::new(
                            fret_core::AttributedText::new(
                                text.clone(),
                                Arc::from([fret_core::TextSpan::new(text.len())]),
                            ),
                        );
                        props.interactive_spans =
                            Arc::from([crate::element::SelectableTextInteractiveSpan {
                                range: 6..10,
                                tag: Arc::<str>::from("link"),
                            }]);

                        let leaf = cx.selectable_text_props(props);
                        *leaf_id.lock().unwrap() = Some(leaf.id);
                        vec![leaf]
                    }),
                ]
            },
        );

        assert!(
            ui.base_root().is_some(),
            "expected render_root_for_frame to install the root layer for base roots"
        );
        if frame == 0 {
            assert_eq!(ui.base_root(), Some(root_node));
        }

        layout_frame(&mut ui, &mut app, &mut services, bounds);
        paint_frame(&mut ui, &mut app, &mut services, bounds, &mut scene);

        app.advance_frame();
    }

    let leaf = leaf_id.lock().unwrap().expect("leaf id should be recorded");
    let spans = crate::elements::with_element_state(
        &mut app,
        window,
        leaf,
        crate::element::SelectableTextState::default,
        |state| state.interactive_span_bounds.clone(),
    );
    assert_eq!(
        spans.len(),
        1,
        "expected cached selectable text state to survive"
    );
    assert_eq!(spans[0].range, 6..10);
    assert_eq!(spans[0].tag.as_ref(), "link");
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
    let mut scene = Scene::default();

    let renders = Arc::new(AtomicUsize::new(0));

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
        let _root_node = render_root_for_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "view-cache-unrelated-model",
            |cx| vec![build_cached(cx, &renders, &observed)],
        );

        layout_frame(&mut ui, &mut app, &mut services, bounds);
        paint_frame(&mut ui, &mut app, &mut services, bounds, &mut scene);
        app.advance_frame();

        if frame == 1 {
            let _ = unrelated.update(&mut app, |v, _cx| *v += 1);
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
    let mut scene = Scene::default();

    let renders = Arc::new(AtomicUsize::new(0));

    for _frame in 0..3 {
        let _root_node = {
            let renders = renders.clone();
            render_root_for_frame(
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

        layout_frame(&mut ui, &mut app, &mut services, bounds);
        paint_frame(&mut ui, &mut app, &mut services, bounds, &mut scene);
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

#[test]
fn view_cache_keep_alive_revalidates_recorded_membership_before_touching_stale_detached_elements() {
    use crate::elements::NodeEntry;

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
    let cache_root_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));
    let live_leaf_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));
    let root_name = "view-cache-keep-alive-revalidates-membership";
    let owner_root = crate::elements::global_root(window, root_name);
    let frame0 = app.frame_id();

    let stale = crate::elements::GlobalElementId(0xCACE_5001);
    let stale_node = ui.create_node(FillStack);
    ui.set_node_element(stale_node, Some(stale));
    assert!(
        ui.resolve_live_attached_node_for_element_seeded(stale, Some(stale_node))
            .is_none(),
        "reproducer requires a stale detached node entry"
    );

    let mut root0: Option<NodeId> = None;
    let mut cache_root: Option<crate::elements::GlobalElementId> = None;
    let mut live_leaf: Option<crate::elements::GlobalElementId> = None;

    for frame in 0..2 {
        let keep_alive = frame == 1;
        let root = render_root_for_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            root_name,
            {
                let renders = renders.clone();
                let cache_root_id = cache_root_id.clone();
                let live_leaf_id = live_leaf_id.clone();
                move |cx| {
                    let subtree = cx.view_cache_keep_alive(
                        crate::element::ViewCacheProps::default(),
                        keep_alive,
                        {
                            let renders = renders.clone();
                            let live_leaf_id = live_leaf_id.clone();
                            move |cx| {
                                renders.fetch_add(1, Ordering::SeqCst);
                                let leaf = cx.text("live");
                                *live_leaf_id.lock().unwrap() = Some(leaf.id);
                                vec![leaf]
                            }
                        },
                    );
                    *cache_root_id.lock().unwrap() = Some(subtree.id);
                    vec![subtree]
                }
            },
        );

        if frame == 0 {
            root0 = Some(root);
            cache_root = Some(cache_root_id.lock().unwrap().expect("cache root id"));
            live_leaf = Some(live_leaf_id.lock().unwrap().expect("live leaf id"));

            app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _app| {
                let window_state = runtime.for_window_mut(window);
                window_state.set_node_entry(
                    stale,
                    NodeEntry {
                        node: stale_node,
                        last_seen_frame: frame0,
                        root: owner_root,
                    },
                );
                window_state.record_view_cache_subtree_elements(
                    cache_root.expect("cache root set on frame 0"),
                    vec![
                        cache_root.expect("cache root set on frame 0"),
                        live_leaf.expect("live leaf set on frame 0"),
                        stale,
                    ],
                );
            });

            app.advance_frame();
            continue;
        }

        assert_eq!(
            Some(root),
            root0,
            "expected stable root identity across keep-alive reuse"
        );
    }

    let cache_root = cache_root.expect("cache root");
    let live_leaf = live_leaf.expect("live leaf");

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "keep-alive reuse should skip the child render closure"
    );

    app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _app| {
        let window_state = runtime.for_window_mut(window);
        let elements = window_state
            .view_cache_elements_for_root(cache_root)
            .expect("expected authoritative membership after keep-alive reuse");
        assert!(
            elements.contains(&cache_root),
            "cache root should remain in its own membership list"
        );
        assert!(
            elements.contains(&live_leaf),
            "live retained descendants should remain in the authoritative membership list"
        );
        assert!(
            !elements.contains(&stale),
            "stale detached recorded members must be dropped when keep-alive reuse revalidates membership"
        );

        let stale_entry = window_state
            .node_entry(stale)
            .expect("stale entry should remain until GC lag expires");
        assert_eq!(
            stale_entry.last_seen_frame, frame0,
            "stale detached node entries must not be touched by invalid recorded membership"
        );
    });
}
