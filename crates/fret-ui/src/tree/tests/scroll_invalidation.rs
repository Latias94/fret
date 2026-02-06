use super::*;
use fret_core::Modifiers;

#[test]
fn scroll_wheel_invalidation_is_hit_test_only() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeUiServices;

    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "scroll",
        |cx| {
            let handle = scroll_handle.clone();
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(handle),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(crate::element::ColumnProps::default(), |cx| {
                        (0..100)
                            .map(|i| cx.text(format!("row-{i}")))
                            .collect::<Vec<_>>()
                    })]
                },
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let scroll_node = ui.children(root)[0];
    let scroll_bounds = ui.debug_node_bounds(scroll_node).expect("scroll bounds");
    let p = Point::new(
        Px(scroll_bounds.origin.x.0 + 5.0),
        Px(scroll_bounds.origin.y.0 + 5.0),
    );

    let ids: Vec<NodeId> = ui.nodes.keys().collect();
    for id in ids {
        ui.test_clear_node_invalidations(id);
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: PointerId(0),
            position: p,
            delta: Point::new(Px(0.0), Px(-60.0)),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected wheel to update scroll offset"
    );

    let scroll_flags = ui.nodes[scroll_node].invalidation;
    assert!(
        !scroll_flags.layout,
        "expected scroll wheel to avoid layout invalidation"
    );
    assert!(
        scroll_flags.hit_test && scroll_flags.paint,
        "expected scroll wheel to invalidate hit-test + paint"
    );
}

#[test]
fn virtual_list_wheel_scroll_is_hit_test_only_within_overscan_window() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeUiServices;

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "virtual-list",
        |cx| {
            let mut layout = crate::element::LayoutStyle::default();
            layout.size.width = crate::element::Length::Fill;
            layout.size.height = crate::element::Length::Fill;
            layout.overflow = crate::element::Overflow::Clip;

            vec![cx.virtual_list_keyed_with_layout(
                layout,
                10_000,
                crate::element::VirtualListOptions::fixed(Px(28.0), 10),
                &scroll_handle,
                |i| i as crate::ItemKey,
                |cx, _i| cx.spacer(crate::element::SpacerProps::default()),
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let list_node = ui.children(root)[0];
    let list_bounds = ui.debug_node_bounds(list_node).expect("list bounds");
    let p = Point::new(
        Px(list_bounds.origin.x.0 + 5.0),
        Px(list_bounds.origin.y.0 + 5.0),
    );

    let ids: Vec<NodeId> = ui.nodes.keys().collect();
    for id in ids {
        ui.test_clear_node_invalidations(id);
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: PointerId(0),
            position: p,
            delta: Point::new(Px(0.0), Px(-56.0)),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected wheel to update scroll offset"
    );

    let flags = ui.nodes[list_node].invalidation;
    assert!(
        !flags.layout,
        "expected virtual list wheel to avoid layout invalidation while still inside the overscan window"
    );
    assert!(
        flags.hit_test && flags.paint,
        "expected virtual list wheel to invalidate hit-test + paint"
    );
}

#[test]
fn virtual_list_out_of_band_scroll_avoids_layout_after_overscan_window() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeUiServices;

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let mut scene = Scene::default();
    let render_frame = |ui: &mut UiTree<crate::test_host::TestHost>,
                        app: &mut crate::test_host::TestHost,
                        services: &mut FakeUiServices|
     -> NodeId {
        declarative::render_root(ui, app, services, window, bounds, "virtual-list", |cx| {
            let mut layout = crate::element::LayoutStyle::default();
            layout.size.width = crate::element::Length::Fill;
            layout.size.height = crate::element::Length::Fill;
            layout.overflow = crate::element::Overflow::Clip;

            vec![cx.virtual_list_keyed_with_layout(
                layout,
                10_000,
                crate::element::VirtualListOptions::fixed(Px(28.0), 10),
                &scroll_handle,
                |i| i as crate::ItemKey,
                |cx, _i| cx.spacer(crate::element::SpacerProps::default()),
            )]
        })
    };

    // Frame 0: first mount establishes the viewport.
    let root = render_frame(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    // Frame 1: rerender after the list has a final viewport so `render_window_range` is populated.
    app.advance_frame();
    let root = render_frame(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    let list_node = ui.children(root)[0];
    let list_element = ui.nodes[list_node]
        .element
        .expect("expected virtual list node to have an element id");
    let has_render_window = crate::elements::with_element_state(
        &mut app,
        window,
        list_element,
        crate::element::VirtualListState::default,
        |state| state.render_window_range.is_some(),
    );
    assert!(
        has_render_window,
        "expected frame-1 rerender to populate VirtualListState.render_window_range"
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    // Small jump: still within the overscan window, should remain HitTestOnly.
    scroll_handle.set_offset(Point::new(Px(0.0), Px(28.0 * 5.0)));
    app.advance_frame();
    let root = render_frame(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    let saw_layout = ui.debug_invalidation_walks().iter().any(|w| {
        w.inv == Invalidation::Layout && w.detail == UiDebugInvalidationDetail::ScrollHandleLayout
    });
    let saw_hit_test_only = ui.debug_invalidation_walks().iter().any(|w| {
        w.inv == Invalidation::HitTestOnly
            && w.detail == UiDebugInvalidationDetail::ScrollHandleHitTestOnly
    });
    assert!(
        saw_hit_test_only,
        "expected out-of-band scroll within overscan to record a hit-test-only invalidation walk"
    );
    assert!(
        !saw_layout,
        "expected out-of-band scroll within overscan to avoid layout invalidation walks"
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    // Larger jump: leaves the overscan window. This should remain HitTestOnly: without view-cache
    // reuse, rerender will rebuild the visible items anyway, so we should not pay for a layout
    // invalidation walk just to detect a window mismatch.
    scroll_handle.set_offset(Point::new(Px(0.0), Px(28.0 * 25.0)));
    app.advance_frame();
    let root = render_frame(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    let saw_layout = ui.debug_invalidation_walks().iter().any(|w| {
        w.inv == Invalidation::Layout && w.detail == UiDebugInvalidationDetail::ScrollHandleLayout
    });
    let saw_hit_test_only = ui.debug_invalidation_walks().iter().any(|w| {
        w.inv == Invalidation::HitTestOnly
            && w.detail == UiDebugInvalidationDetail::ScrollHandleHitTestOnly
    });
    assert!(
        saw_hit_test_only,
        "expected out-of-band scroll beyond overscan to record a hit-test-only invalidation walk"
    );
    assert!(
        !saw_layout,
        "expected out-of-band scroll beyond overscan to avoid layout invalidation walks"
    );
}

#[test]
fn virtual_list_window_jump_rerender_uses_latest_handle_offset() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeUiServices;

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let mut scene = Scene::default();
    let render_frame = |ui: &mut UiTree<crate::test_host::TestHost>,
                        app: &mut crate::test_host::TestHost,
                        services: &mut FakeUiServices|
     -> NodeId {
        declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "virtual-list-jump",
            |cx| {
                let mut layout = crate::element::LayoutStyle::default();
                layout.size.width = crate::element::Length::Fill;
                layout.size.height = crate::element::Length::Fill;
                layout.overflow = crate::element::Overflow::Clip;

                vec![cx.virtual_list_keyed_with_layout(
                    layout,
                    10_000,
                    crate::element::VirtualListOptions::fixed(Px(28.0), 10),
                    &scroll_handle,
                    |i| i as crate::ItemKey,
                    |cx, _i| cx.text("row"),
                )]
            },
        )
    };

    // Frame 0: establish viewport and initial window.
    let root = render_frame(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    // Frame 1: rerender once to ensure `render_window_range` is populated.
    app.advance_frame();
    let root = render_frame(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    let list_node = ui.children(root)[0];
    let list_element = ui.nodes[list_node]
        .element
        .expect("expected virtual list node to have an element id");
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let state = crate::elements::with_element_state(
        &mut app,
        window,
        list_element,
        crate::element::VirtualListState::default,
        |s| s.clone(),
    );
    assert!(
        state.render_window_range.is_some(),
        "expected render_window_range to be populated after frame-1 rerender"
    );
    assert!(
        state.has_final_viewport && state.viewport_h.0 > 0.0,
        "expected a final viewport before validating window jumps"
    );

    // Frame 2: jump far enough to leave the previous overscan window so the runtime upgrades to
    // Layout and rerenders. The rerender should compute the visible window against the latest
    // handle offset (not the stale state offset from last layout).
    scroll_handle.set_offset(Point::new(Px(0.0), Px(28.0 * 25.0)));
    app.advance_frame();
    let root = render_frame(&mut ui, &mut app, &mut services);
    ui.set_root(root);

    let jumped_state = crate::elements::with_element_state(
        &mut app,
        window,
        list_element,
        crate::element::VirtualListState::default,
        |s| s.clone(),
    );

    let viewport = jumped_state.viewport_h;
    let offset = jumped_state
        .metrics
        .clamp_offset(scroll_handle.offset().y, viewport);
    let expected = jumped_state.metrics.visible_range(offset, viewport, 10);
    assert_eq!(
        jumped_state.render_window_range, expected,
        "expected render_window_range to match the latest handle offset on a window jump rerender"
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let last = ui
        .debug_virtual_list_windows()
        .last()
        .expect("expected a virtual list window debug record");
    assert!(
        !last.window_mismatch,
        "expected a window jump rerender to avoid a follow-up window_mismatch frame"
    );
}

#[test]
fn virtual_list_window_shift_detail_classifies_viewport_resize() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let mut services = FakeUiServices;
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    let mut scene = Scene::default();

    let bounds_small = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let bounds_large = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(480.0)),
    );

    let render_frame = |ui: &mut UiTree<crate::test_host::TestHost>,
                        app: &mut crate::test_host::TestHost,
                        services: &mut FakeUiServices,
                        bounds: Rect,
                        items_revision: u64|
     -> NodeId {
        let handle = scroll_handle.clone();
        declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "vlist-viewport-resize",
            |cx| {
                let mut cache = crate::element::ViewCacheProps::default();
                cache.layout.size.width = crate::element::Length::Fill;
                cache.layout.size.height = crate::element::Length::Fill;
                cache.cache_key = 1;

                vec![cx.view_cache(cache, move |cx| {
                    let mut layout = crate::element::LayoutStyle::default();
                    layout.size.width = crate::element::Length::Fill;
                    layout.size.height = crate::element::Length::Fill;
                    layout.overflow = crate::element::Overflow::Clip;

                    let mut options = crate::element::VirtualListOptions::fixed(Px(28.0), 10);
                    options.items_revision = items_revision;
                    vec![cx.virtual_list_keyed_with_layout(
                        layout,
                        10_000,
                        options,
                        &handle,
                        |i| i as crate::ItemKey,
                        |cx, _i| cx.spacer(crate::element::SpacerProps::default()),
                    )]
                })]
            },
        )
    };

    // Frame 0: establish viewport.
    let root = render_frame(&mut ui, &mut app, &mut services, bounds_small, 1);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds_small, 1.0);
    ui.paint_all(&mut app, &mut services, bounds_small, &mut scene, 1.0);
    app.advance_frame();

    // Frame 1: rerender so `render_window_range` is populated for mismatch checks.
    let root = render_frame(&mut ui, &mut app, &mut services, bounds_small, 1);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds_small, 1.0);
    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds_small, &mut scene, 1.0);
    app.advance_frame();

    // Frame 2: enlarge viewport so the current visible range escapes the previously rendered
    // overscan window.
    let root = render_frame(&mut ui, &mut app, &mut services, bounds_large, 1);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds_large, 1.0);

    let record = ui
        .debug_virtual_list_windows()
        .iter()
        .rev()
        .find(|r| {
            r.window_shift_reason
                == Some(crate::tree::UiDebugVirtualListWindowShiftReason::ViewportResize)
                && r.window_shift_apply_mode
                    == Some(
                        crate::tree::UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender,
                    )
        })
        .expect("expected a viewport-resize window shift record");
    assert_eq!(
        record.window_shift_invalidation_detail,
        Some(crate::tree::UiDebugInvalidationDetail::ScrollHandleViewportResizeWindowUpdate)
    );
}

#[test]
fn virtual_list_window_shift_detail_classifies_items_revision() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let cache_root = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![cache_root]);
    ui.set_children(cache_root, vec![child]);
    ui.set_node_view_cache_flags(cache_root, true, false, false);

    ui.mark_nearest_view_cache_root_needs_rerender(
        child,
        UiDebugInvalidationSource::Other,
        crate::tree::UiDebugInvalidationDetail::ScrollHandleItemsRevisionWindowUpdate,
    );

    assert!(
        ui.nodes[cache_root].view_cache_needs_rerender,
        "expected items revision window update detail to mark the nearest cache root dirty"
    );
}

#[test]
fn scroll_offset_changes_do_not_replay_paint_cache() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    ui.set_paint_cache_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeUiServices;

    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "scroll",
        |cx| {
            let handle = scroll_handle.clone();
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(handle),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(crate::element::ColumnProps::default(), |cx| {
                        (0..100)
                            .map(|i| cx.text(format!("row-{i}")))
                            .collect::<Vec<_>>()
                    })]
                },
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let bindings = crate::declarative::frame::bound_elements_for_scroll_handle(
        &mut app,
        window,
        scroll_handle.binding_key(),
    );
    assert!(
        !bindings.is_empty(),
        "expected scroll handle bindings to be registered for the scroll node"
    );
    assert!(
        bindings.iter().any(
            |&element| crate::declarative::node_for_element_in_window_frame(
                &mut app, window, element
            ) == Some(scroll_node)
        ),
        "expected scroll handle binding to resolve to the scroll node"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let transforms1: Vec<Transform2D> = scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            fret_core::SceneOp::PushTransform { transform } => Some(*transform),
            _ => None,
        })
        .collect();
    let fp1 = scene.fingerprint();
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    let ids: Vec<NodeId> = ui.nodes.keys().collect();
    for id in ids {
        ui.test_clear_node_invalidations(id);
    }

    let prev_offset = scroll_handle.offset();
    let prev_children_transform = ui
        .node_children_render_transform(scroll_node)
        .unwrap_or(Transform2D::IDENTITY);

    let prev_revision = scroll_handle.revision();
    scroll_handle.set_offset(Point::new(Px(0.0), Px(100.0)));
    assert!(
        scroll_handle.revision() > prev_revision,
        "expected programmatic scroll to bump scroll handle revision"
    );
    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected programmatic scroll to update scroll offset"
    );
    assert!(
        (scroll_handle.offset().y.0 - prev_offset.y.0).abs() > 0.01,
        "expected programmatic scroll to change scroll offset"
    );
    let children_transform = ui
        .node_children_render_transform(scroll_node)
        .unwrap_or(Transform2D::IDENTITY);
    assert_ne!(
        children_transform,
        Transform2D::IDENTITY,
        "expected a non-identity children render transform after scrolling"
    );
    assert_ne!(
        children_transform, prev_children_transform,
        "expected scroll offset changes to update children render transform"
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let transforms2: Vec<Transform2D> = scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            fret_core::SceneOp::PushTransform { transform } => Some(*transform),
            _ => None,
        })
        .collect();
    let fp2 = scene.fingerprint();

    assert!(
        !ui.debug_paint_cache_replays.contains_key(&root)
            && !ui.debug_paint_cache_replays.contains_key(&scroll_node),
        "expected scroll offset changes to prevent paint-cache replay for scroll ancestors, got: {:?} (root={:?}, scroll_node={:?})",
        ui.debug_paint_cache_replays,
        root,
        scroll_node,
    );
    assert_ne!(
        transforms1, transforms2,
        "expected scroll offset changes to update emitted transform ops"
    );
    assert_ne!(
        fp1, fp2,
        "expected scroll offset changes to update scene output"
    );
}
