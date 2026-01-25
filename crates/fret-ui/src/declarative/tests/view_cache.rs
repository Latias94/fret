use super::*;

use fret_runtime::GlobalsHost;
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
fn view_cache_subtree_membership_keeps_detached_children_alive_under_cache_hit() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
        runtime.set_gc_lag_frames(0);
    });

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let renders_outer = Arc::new(AtomicUsize::new(0));
    let renders_inner = Arc::new(AtomicUsize::new(0));
    let outer_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));
    let leaf_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));

    let mut root: Option<NodeId> = None;

    for frame in 0..2 {
        let renders_outer = renders_outer.clone();
        let renders_inner = renders_inner.clone();
        let outer_id_for_render = outer_id.clone();
        let leaf_id_for_render = leaf_id.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "view-cache-membership-under-detach",
            move |cx| {
                let outer = cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                    renders_outer.fetch_add(1, Ordering::SeqCst);
                    let inner = cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                        renders_inner.fetch_add(1, Ordering::SeqCst);
                        let leaf = cx.text("leaf");
                        *leaf_id_for_render.lock().unwrap() = Some(leaf.id);
                        cx.with_state_for(leaf.id, || 123u32, |_| {});
                        vec![leaf]
                    });
                    vec![inner]
                });
                *outer_id_for_render.lock().unwrap() = Some(outer.id);
                vec![outer]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        if frame == 0 {
            let outer_element = outer_id
                .lock()
                .unwrap()
                .expect("outer cache root should be set");
            let outer_node =
                app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
                    runtime
                        .for_window_mut(window)
                        .node_entry(outer_element)
                        .expect("outer cache root must have a node entry")
                        .node
                });

            let detached_child = ui
                .children(outer_node)
                .first()
                .copied()
                .expect("outer cache root should have a child node");

            // Simulate bookkeeping drift: sever the child edge without triggering invalidations,
            // then remove the cached frame-child entry so the next cache-hit frame cannot reach
            // the subtree via child edges. Liveness must still be preserved via the ViewCache
            // subtree membership list (ADR 0191).
            ui.debug_sever_child_edge_without_invalidation(outer_node, detached_child);
            app.with_global_mut_untracked(
                crate::declarative::frame::ElementFrame::default,
                |frame, _| {
                    let window_frame = frame
                        .windows
                        .get_mut(&window)
                        .expect("window frame should exist");
                    window_frame.children.remove(&outer_node);
                },
            );
        }

        app.advance_frame();
    }

    assert_eq!(
        renders_outer.load(Ordering::SeqCst),
        1,
        "outer view cache should hit (child render closure should not rerun)"
    );
    assert_eq!(
        renders_inner.load(Ordering::SeqCst),
        1,
        "inner view cache should only render on the initial cache-miss frame"
    );

    let leaf = leaf_id.lock().unwrap().expect("leaf id should be recorded");
    let value = crate::elements::with_element_state(&mut app, window, leaf, || 0u32, |v| *v);
    assert_eq!(
        value, 123,
        "leaf element state should survive cache-hit frames even if child edges drift"
    );

    let leaf_node_entry_exists = app
        .with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
            runtime.for_window_mut(window).node_entry(leaf).is_some()
        });
    assert!(
        leaf_node_entry_exists,
        "leaf node entry should remain alive under cache-hit liveness bookkeeping"
    );
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
fn view_cache_rerenders_on_virtual_list_scroll_to_item() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(40.0)),
    );
    let mut services = FakeTextService::default();

    let renders = Arc::new(AtomicUsize::new(0));
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let mut root: Option<NodeId> = None;
    let mut scene = Scene::default();

    let render_frame =
        |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut FakeTextService| {
            let renders = renders.clone();
            let scroll_handle = scroll_handle.clone();
            render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "view-cache-virtual-list-scroll-to-item",
                move |cx| {
                    vec![
                        cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                            renders.fetch_add(1, Ordering::SeqCst);
                            vec![cx.virtual_list_keyed(
                                500,
                                crate::element::VirtualListOptions::fixed(Px(10.0), 0),
                                &scroll_handle,
                                |i| i as u64,
                                |cx, _i| cx.text("row"),
                            )]
                        }),
                    ]
                },
            )
        };

    // Frame 0: mount.
    let root0 = render_frame(&mut ui, &mut app, &mut services);
    root.get_or_insert(root0);
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    app.advance_frame();

    // Advance until we observe a cache-hit frame (no child render closure call).
    let mut renders_before_scroll: Option<usize> = None;
    for _ in 0..8 {
        let before = renders.load(Ordering::SeqCst);
        let root_n = render_frame(&mut ui, &mut app, &mut services);
        root.get_or_insert(root_n);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        scene.clear();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        let after = renders.load(Ordering::SeqCst);

        let cache_hit = after == before;
        let mismatch = ui
            .debug_virtual_list_windows()
            .last()
            .is_some_and(|w| w.window_mismatch);

        app.advance_frame();

        if cache_hit && !mismatch {
            renders_before_scroll = Some(after);
            break;
        }
    }

    let renders_before_scroll = renders_before_scroll
        .unwrap_or_else(|| panic!("expected a stable cache-hit frame before scroll_to_item"));

    // Out-of-band scroll-to-item should force rerender (disable reuse) on the next frame so the
    // visible rows are rebuilt in the same tick (avoids a one-frame stale list).
    scroll_handle.scroll_to_item(80, crate::scroll::ScrollStrategy::Start);
    assert!(scroll_handle.deferred_scroll_to_item().is_some());

    // Next frame: render should rerun due to the layout-affecting scroll handle change.
    let before = renders.load(Ordering::SeqCst);
    let root1 = render_frame(&mut ui, &mut app, &mut services);
    root.get_or_insert(root1);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert!(
        scroll_handle.deferred_scroll_to_item().is_none(),
        "expected final layout to consume deferred scroll request"
    );
    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected scroll offset to change"
    );
    assert_eq!(
        renders.load(Ordering::SeqCst),
        before + 1,
        "scroll_to_item should disable view-cache reuse for the affected root"
    );
    assert_eq!(before, renders_before_scroll);

    app.advance_frame();

    // Ensure subsequent out-of-band scroll-to-item requests remain layout-affecting even when the
    // offset was previously updated internally during layout (the registry must not misclassify
    // it as hit-test-only due to stale last_offset bookkeeping).
    let mut stable_after_first_scroll: Option<usize> = None;
    for _ in 0..8 {
        let before = renders.load(Ordering::SeqCst);
        let root_n = render_frame(&mut ui, &mut app, &mut services);
        root.get_or_insert(root_n);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        scene.clear();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        let after = renders.load(Ordering::SeqCst);

        let cache_hit = after == before;
        let mismatch = ui
            .debug_virtual_list_windows()
            .last()
            .is_some_and(|w| w.window_mismatch);

        app.advance_frame();

        if cache_hit && !mismatch {
            stable_after_first_scroll = Some(after);
            break;
        }
    }

    let renders_before_second_scroll = stable_after_first_scroll.unwrap_or_else(|| {
        panic!("expected a stable cache-hit frame after the first scroll_to_item")
    });

    let offset_before_second_scroll = scroll_handle.offset().y;
    scroll_handle.scroll_to_item(120, crate::scroll::ScrollStrategy::Start);
    assert!(scroll_handle.deferred_scroll_to_item().is_some());

    let before = renders.load(Ordering::SeqCst);
    let root2 = render_frame(&mut ui, &mut app, &mut services);
    root.get_or_insert(root2);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert!(
        scroll_handle.deferred_scroll_to_item().is_none(),
        "expected final layout to consume deferred scroll request"
    );
    assert!(
        scroll_handle.offset().y.0 > offset_before_second_scroll.0 + 0.01,
        "expected a subsequent scroll_to_item request to update the scroll offset"
    );
    assert_eq!(
        renders.load(Ordering::SeqCst),
        before + 1,
        "subsequent scroll_to_item should still disable view-cache reuse for the affected root"
    );
    assert_eq!(before, renders_before_second_scroll);
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
