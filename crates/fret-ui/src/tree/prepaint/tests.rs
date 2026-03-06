use super::*;

struct NoopWidget;
impl Widget<crate::test_host::TestHost> for NoopWidget {}

#[test]
fn prepaint_updates_virtual_list_window_and_marks_cache_root_dirty_on_escape() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let cache_root = ui.create_node(NoopWidget);
    ui.nodes[cache_root].view_cache.enabled = true;
    ui.set_root(cache_root);

    let element = GlobalElementId(1);
    let vlist_node = ui.create_node_for_element(element, NoopWidget);
    ui.add_child(cache_root, vlist_node);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(40.0)),
    );
    ui.nodes[vlist_node].bounds = bounds;

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    crate::declarative::frame::with_window_frame_mut(&mut app, window, |frame| {
        frame.instances.insert(
            vlist_node,
            crate::declarative::frame::ElementRecord {
                element,
                instance: crate::declarative::frame::ElementInstance::VirtualList(
                    crate::element::VirtualListProps {
                        layout: crate::element::LayoutStyle::default(),
                        axis: fret_core::Axis::Vertical,
                        len: 1000,
                        items_revision: 1,
                        estimate_row_height: Px(10.0),
                        measure_mode: crate::element::VirtualListMeasureMode::Fixed,
                        key_cache: crate::element::VirtualListKeyCacheMode::VisibleOnly,
                        overscan: 10,
                        keep_alive: 0,
                        scroll_margin: Px(0.0),
                        gap: Px(0.0),
                        scroll_handle: scroll_handle.clone(),
                        visible_items: Vec::new(),
                    },
                ),
                inherited_foreground: None,
                semantics_decoration: None,
                key_context: None,
            },
        );
    });

    crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::VirtualListState::default,
        |state| {
            state.render_window_range = Some(crate::virtual_list::VirtualRange {
                start_index: 0,
                end_index: 20,
                overscan: 10,
                count: 1000,
            });
            state.viewport_h = bounds.size.height;
        },
    );

    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(220.0)));

    let record = InteractionRecord {
        node: vlist_node,
        bounds,
        render_transform_inv: None,
        children_render_transform_inv: None,
        clips_hit_test: true,
        clip_hit_test_corner_radii: None,
        is_focusable: false,
        focus_traversal_children: true,
        can_scroll_descendant_into_view: true,
    };

    ui.prepaint_virtual_list_window_from_interaction_record(&mut app, &record);
    assert!(
        !ui.nodes[cache_root].view_cache_needs_rerender,
        "expected overscan-contained offset changes to avoid dirtying the cache root"
    );

    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(620.0)));
    ui.prepaint_virtual_list_window_from_interaction_record(&mut app, &record);
    assert!(
        ui.nodes[cache_root].view_cache_needs_rerender,
        "expected prepaint window escape to dirty the nearest cache root"
    );

    let last = ui
        .debug_virtual_list_windows()
        .last()
        .expect("expected a debug virtual list window record");
    assert!(
        matches!(
            last.source,
            crate::tree::UiDebugVirtualListWindowSource::Prepaint
        ),
        "expected the debug window record to be sourced from prepaint"
    );
    assert!(
        last.window_mismatch,
        "expected the last prepaint window update to report a mismatch"
    );
    assert_eq!(
        last.window_shift_kind,
        crate::tree::UiDebugVirtualListWindowShiftKind::Escape,
        "expected the last prepaint window update to record an escape shift"
    );
    assert_eq!(
        last.window_shift_apply_mode,
        Some(crate::tree::UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender),
        "expected the non-retained virtual list path to apply window shifts via rerender"
    );
    assert_eq!(
        last.window_shift_reason,
        Some(crate::tree::UiDebugVirtualListWindowShiftReason::ScrollOffset),
        "expected the escape shift in this test to be attributed to scroll offset"
    );
    assert_eq!(
        last.window_shift_invalidation_detail,
        Some(crate::tree::UiDebugInvalidationDetail::ScrollHandleWindowUpdate),
        "expected the non-retained escape shift to align with the scroll-handle window-update invalidation detail"
    );
}

#[test]
fn prepaint_detects_render_window_insufficient_for_overscan_policy() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let cache_root = ui.create_node(NoopWidget);
    ui.nodes[cache_root].view_cache.enabled = true;
    ui.set_root(cache_root);

    let element = GlobalElementId(1);
    let vlist_node = ui.create_node_for_element(element, NoopWidget);
    ui.add_child(cache_root, vlist_node);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(200.0)),
    );
    ui.nodes[vlist_node].bounds = bounds;

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    crate::declarative::frame::with_window_frame_mut(&mut app, window, |frame| {
        frame.instances.insert(
            vlist_node,
            crate::declarative::frame::ElementRecord {
                element,
                instance: crate::declarative::frame::ElementInstance::VirtualList(
                    crate::element::VirtualListProps {
                        layout: crate::element::LayoutStyle::default(),
                        axis: fret_core::Axis::Vertical,
                        len: 1000,
                        items_revision: 1,
                        estimate_row_height: Px(10.0),
                        measure_mode: crate::element::VirtualListMeasureMode::Fixed,
                        key_cache: crate::element::VirtualListKeyCacheMode::VisibleOnly,
                        overscan: 10,
                        keep_alive: 0,
                        scroll_margin: Px(0.0),
                        gap: Px(0.0),
                        scroll_handle: scroll_handle.clone(),
                        visible_items: Vec::new(),
                    },
                ),
                inherited_foreground: None,
                semantics_decoration: None,
                key_context: None,
            },
        );
    });

    // Simulate a render-derived window computed under a smaller viewport:
    // - rendered visible range: 0..9 (10 items)
    // - overscan: 10 => rendered expanded window: 0..19
    //
    // Under the final viewport (200px @ 10px rows) the desired visible range is 0..19, so the
    // visible range is still within the rendered expanded envelope. However, the desired
    // overscan policy needs the expanded window to cover 0..29. Prepaint should treat this as a
    // mismatch and schedule a one-shot rerender/reconcile.
    crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::VirtualListState::default,
        |state| {
            state.render_window_range = Some(crate::virtual_list::VirtualRange {
                start_index: 0,
                end_index: 9,
                overscan: 10,
                count: 1000,
            });
            state.items_revision = 1;
            state.items_len = 1000;
            state.viewport_h = Px(100.0);
            state.offset_y = Px(0.0);
        },
    );

    let record = InteractionRecord {
        node: vlist_node,
        bounds,
        render_transform_inv: None,
        children_render_transform_inv: None,
        clips_hit_test: true,
        clip_hit_test_corner_radii: None,
        is_focusable: false,
        focus_traversal_children: true,
        can_scroll_descendant_into_view: true,
    };

    ui.prepaint_virtual_list_window_from_interaction_record(&mut app, &record);

    assert!(
        ui.nodes[cache_root].view_cache_needs_rerender,
        "expected prepaint to dirty the nearest cache root when the rendered window is insufficient for the desired overscan policy"
    );

    let last = ui
        .debug_virtual_list_windows()
        .last()
        .expect("expected a debug virtual list window record");
    assert!(
        !last.window_mismatch,
        "expected the prepaint update to prefetch while still within the rendered window"
    );
    assert_eq!(
        last.window_shift_kind,
        crate::tree::UiDebugVirtualListWindowShiftKind::Prefetch,
        "expected the prepaint update to record a prefetch shift for the one-shot correction"
    );
    assert_eq!(
        last.window_shift_reason,
        Some(crate::tree::UiDebugVirtualListWindowShiftReason::ViewportResize),
        "expected the correction to be attributed to the viewport change"
    );
    assert_eq!(
        last.window_shift_invalidation_detail,
        Some(crate::tree::UiDebugInvalidationDetail::ScrollHandleViewportResizeWindowUpdate),
        "expected viewport-driven window updates to have a distinct invalidation detail"
    );
}

#[test]
fn prepaint_marks_scroll_to_item_window_updates_with_distinct_invalidation_detail() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let cache_root = ui.create_node(NoopWidget);
    ui.nodes[cache_root].view_cache.enabled = true;
    ui.set_root(cache_root);

    let element = GlobalElementId(1);
    let vlist_node = ui.create_node_for_element(element, NoopWidget);
    ui.add_child(cache_root, vlist_node);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(40.0)),
    );
    ui.nodes[vlist_node].bounds = bounds;

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    crate::declarative::frame::with_window_frame_mut(&mut app, window, |frame| {
        frame.instances.insert(
            vlist_node,
            crate::declarative::frame::ElementRecord {
                element,
                instance: crate::declarative::frame::ElementInstance::VirtualList(
                    crate::element::VirtualListProps {
                        layout: crate::element::LayoutStyle::default(),
                        axis: fret_core::Axis::Vertical,
                        len: 1000,
                        items_revision: 1,
                        estimate_row_height: Px(10.0),
                        measure_mode: crate::element::VirtualListMeasureMode::Fixed,
                        key_cache: crate::element::VirtualListKeyCacheMode::VisibleOnly,
                        overscan: 10,
                        keep_alive: 0,
                        scroll_margin: Px(0.0),
                        gap: Px(0.0),
                        scroll_handle: scroll_handle.clone(),
                        visible_items: Vec::new(),
                    },
                ),
                inherited_foreground: None,
                semantics_decoration: None,
                key_context: None,
            },
        );
    });

    crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::VirtualListState::default,
        |state| {
            state.render_window_range = Some(crate::virtual_list::VirtualRange {
                start_index: 0,
                end_index: 20,
                overscan: 10,
                count: 1000,
            });
            state.viewport_h = bounds.size.height;
        },
    );

    // Simulate a pending scroll-to-item request. This should classify subsequent window updates
    // distinctly from scroll-offset-driven updates.
    scroll_handle.scroll_to_item(900, crate::scroll::ScrollStrategy::Nearest);
    // Force an escape mismatch by jumping the offset far beyond the rendered overscan.
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(620.0)));

    let record = InteractionRecord {
        node: vlist_node,
        bounds,
        render_transform_inv: None,
        children_render_transform_inv: None,
        clips_hit_test: true,
        clip_hit_test_corner_radii: None,
        is_focusable: false,
        focus_traversal_children: true,
        can_scroll_descendant_into_view: true,
    };

    ui.prepaint_virtual_list_window_from_interaction_record(&mut app, &record);

    let last = ui
        .debug_virtual_list_windows()
        .last()
        .expect("expected a debug virtual list window record");
    assert_eq!(
        last.window_shift_kind,
        crate::tree::UiDebugVirtualListWindowShiftKind::Escape,
        "expected scroll-to-item to trigger an escape shift in this setup"
    );
    assert_eq!(
        last.window_shift_reason,
        Some(crate::tree::UiDebugVirtualListWindowShiftReason::ScrollToItem),
        "expected the shift to be attributed to a deferred scroll-to-item request"
    );
    assert_eq!(
        last.window_shift_invalidation_detail,
        Some(crate::tree::UiDebugInvalidationDetail::ScrollHandleScrollToItemWindowUpdate),
        "expected scroll-to-item window updates to have a distinct invalidation detail"
    );
}

#[test]
fn prepaint_caps_retained_host_prefetch_step_to_bound_attach_detach_delta() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let cache_root = ui.create_node(NoopWidget);
    ui.nodes[cache_root].view_cache.enabled = true;
    ui.set_root(cache_root);

    let element = GlobalElementId(1);
    let vlist_node = ui.create_node_for_element(element, NoopWidget);
    ui.add_child(cache_root, vlist_node);

    // Mark this VirtualList as a retained host (ADR 0177) so prepaint applies window shifts via
    // reconcile rather than rerender.
    crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::windowed_surface_host::RetainedVirtualListHostMarker::default,
        |_| {},
    );

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(100.0)),
    );
    ui.nodes[vlist_node].bounds = bounds;

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    crate::declarative::frame::with_window_frame_mut(&mut app, window, |frame| {
        frame.instances.insert(
            vlist_node,
            crate::declarative::frame::ElementRecord {
                element,
                instance: crate::declarative::frame::ElementInstance::VirtualList(
                    crate::element::VirtualListProps {
                        layout: crate::element::LayoutStyle::default(),
                        axis: fret_core::Axis::Vertical,
                        len: 1000,
                        items_revision: 1,
                        estimate_row_height: Px(10.0),
                        measure_mode: crate::element::VirtualListMeasureMode::Fixed,
                        key_cache: crate::element::VirtualListKeyCacheMode::VisibleOnly,
                        overscan: 40,
                        keep_alive: 0,
                        scroll_margin: Px(0.0),
                        gap: Px(0.0),
                        scroll_handle: scroll_handle.clone(),
                        visible_items: Vec::new(),
                    },
                ),
                inherited_foreground: None,
                semantics_decoration: None,
                key_context: None,
            },
        );
    });

    // Start from a render-derived window and then scroll near the prefetch edge while still
    // remaining within the rendered overscan envelope. This should trigger a prefetch shift.
    crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::VirtualListState::default,
        |state| {
            state.render_window_range = Some(crate::virtual_list::VirtualRange {
                start_index: 100,
                end_index: 119,
                overscan: 40,
                count: 1000,
            });
            state.viewport_h = bounds.size.height;
            state.offset_y = Px(0.0);
        },
    );

    // Visible range: start=150, end=159 (viewport 100px @ 10px rows), which is within the
    // rendered expanded window (60..159) and near the end edge.
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(1500.0)));

    let record = InteractionRecord {
        node: vlist_node,
        bounds,
        render_transform_inv: None,
        children_render_transform_inv: None,
        clips_hit_test: true,
        clip_hit_test_corner_radii: None,
        is_focusable: false,
        focus_traversal_children: true,
        can_scroll_descendant_into_view: true,
    };

    ui.prepaint_virtual_list_window_from_interaction_record(&mut app, &record);

    let last = ui
        .debug_virtual_list_windows()
        .last()
        .expect("expected a debug virtual list window record");
    assert_eq!(
        last.window_shift_kind,
        crate::tree::UiDebugVirtualListWindowShiftKind::Prefetch,
        "expected this setup to trigger a prefetch shift"
    );
    assert_eq!(
        last.window_shift_apply_mode,
        Some(crate::tree::UiDebugVirtualListWindowShiftApplyMode::RetainedReconcile),
        "expected retained hosts to apply shifts via reconcile"
    );

    let prev = last
        .prev_window_range
        .or(last.render_window_range)
        .expect("expected a previous window range");
    let next = last.window_range.expect("expected a next window range");
    let delta = next.start_index.saturating_sub(prev.start_index);
    assert!(
        delta <= RETAINED_HOST_PREFETCH_STEP_MAX,
        "expected retained-host prefetch to cap shift delta (delta={delta} max={})",
        RETAINED_HOST_PREFETCH_STEP_MAX
    );
}
