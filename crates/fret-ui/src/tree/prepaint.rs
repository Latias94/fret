use super::*;

#[derive(Clone)]
struct VirtualListPrepaintInputs {
    element: GlobalElementId,
    axis: fret_core::Axis,
    len: usize,
    items_revision: u64,
    measure_mode: crate::element::VirtualListMeasureMode,
    overscan: usize,
    estimate_row_height: Px,
    gap: Px,
    scroll_margin: Px,
    scroll_handle: crate::scroll::VirtualListScrollHandle,
}

#[derive(Debug, Clone, Copy)]
struct VirtualListPrepaintWindowUpdate {
    prev_items_revision: u64,
    prev_viewport: Px,
    prev_offset: Px,
    prev_window_range: Option<crate::virtual_list::VirtualRange>,
    render_window_range: Option<crate::virtual_list::VirtualRange>,
    window_range: Option<crate::virtual_list::VirtualRange>,
    viewport: Px,
    offset: Px,
    deferred_scroll_to_item: bool,
    window_mismatch: bool,
    content_extent: Px,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct InteractionCacheEntry {
    pub(super) generation: u64,
    pub(super) key: PaintCacheKey,
    pub(super) start: u32,
    pub(super) end: u32,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(super) struct InteractionRecord {
    pub(super) node: NodeId,
    pub(super) bounds: Rect,
    pub(super) render_transform_inv: Option<Transform2D>,
    pub(super) children_render_transform_inv: Option<Transform2D>,
    pub(super) clips_hit_test: bool,
    pub(super) clip_hit_test_corner_radii: Option<Corners>,
    pub(super) is_focusable: bool,
    pub(super) focus_traversal_children: bool,
    pub(super) can_scroll_descendant_into_view: bool,
}

#[derive(Debug, Default)]
pub(super) struct InteractionCacheState {
    generation: u64,
    pub(super) prev_records: Vec<InteractionRecord>,
    pub(super) records: Vec<InteractionRecord>,
    replay_scratch: Vec<InteractionRecord>,
    pub(super) source_generation: u64,
    pub(super) target_generation: u64,
    pub(super) hits: u32,
    pub(super) misses: u32,
    pub(super) replayed_records: u32,
}

impl InteractionCacheState {
    pub(super) fn begin_frame(&mut self) {
        self.source_generation = self.generation;
        self.target_generation = self.generation.saturating_add(1);
        self.hits = 0;
        self.misses = 0;
        self.replayed_records = 0;

        std::mem::swap(&mut self.prev_records, &mut self.records);
        self.records.clear();
    }

    pub(super) fn finish_frame(&mut self) {
        self.generation = self.target_generation;
    }

    pub(super) fn invalidate_recording(&mut self) {
        self.prev_records.clear();
        self.records.clear();
        self.replay_scratch.clear();
        self.generation = self.generation.saturating_add(1);
    }
}

impl<H: UiHost> UiTree<H> {
    fn prepaint_virtual_list_window_from_interaction_record(
        &mut self,
        app: &mut H,
        record: &InteractionRecord,
    ) {
        let Some(window) = self.window else {
            return;
        };
        if !record.can_scroll_descendant_into_view {
            return;
        }

        let Some(inputs) =
            crate::declarative::frame::element_record_for_node(&mut *app, window, record.node)
                .and_then(|element_record| match &element_record.instance {
                    crate::declarative::frame::ElementInstance::VirtualList(props) => {
                        Some(VirtualListPrepaintInputs {
                            element: element_record.element,
                            axis: props.axis,
                            len: props.len,
                            items_revision: props.items_revision,
                            measure_mode: props.measure_mode,
                            overscan: props.overscan,
                            estimate_row_height: props.estimate_row_height,
                            gap: props.gap,
                            scroll_margin: props.scroll_margin,
                            scroll_handle: props.scroll_handle.clone(),
                        })
                    }
                    _ => None,
                })
        else {
            return;
        };

        let viewport = match inputs.axis {
            fret_core::Axis::Vertical => Px(record.bounds.size.height.0.max(0.0)),
            fret_core::Axis::Horizontal => Px(record.bounds.size.width.0.max(0.0)),
        };
        if viewport.0 <= 0.0 || inputs.len == 0 {
            return;
        }

        let offset_point = inputs.scroll_handle.offset();
        let offset_axis = match inputs.axis {
            fret_core::Axis::Vertical => offset_point.y,
            fret_core::Axis::Horizontal => offset_point.x,
        };
        let deferred_scroll_to_item = inputs.scroll_handle.deferred_scroll_to_item().is_some();

        let update = crate::elements::with_element_state(
            &mut *app,
            window,
            inputs.element,
            crate::element::VirtualListState::default,
            |state| {
                let prev_items_revision = state.items_revision;
                let prev_viewport = match inputs.axis {
                    fret_core::Axis::Vertical => state.viewport_h,
                    fret_core::Axis::Horizontal => state.viewport_w,
                };
                let prev_offset = match inputs.axis {
                    fret_core::Axis::Vertical => state.offset_y,
                    fret_core::Axis::Horizontal => state.offset_x,
                };
                let prev_window_range = state.window_range;
                let render_window_range = state.render_window_range;

                state.metrics.ensure_with_mode(
                    inputs.measure_mode,
                    inputs.len,
                    inputs.estimate_row_height,
                    inputs.gap,
                    inputs.scroll_margin,
                );
                state.items_revision = inputs.items_revision;

                let content_extent = state.metrics.total_height();
                let offset_axis = state.metrics.clamp_offset(offset_axis, viewport);
                match inputs.axis {
                    fret_core::Axis::Vertical => {
                        state.offset_y = offset_axis;
                        state.viewport_h = viewport;
                    }
                    fret_core::Axis::Horizontal => {
                        state.offset_x = offset_axis;
                        state.viewport_w = viewport;
                    }
                }
                if viewport.0 > 0.0 {
                    state.has_final_viewport = true;
                }

                let window_range =
                    state
                        .metrics
                        .visible_range(offset_axis, viewport, inputs.overscan);
                state.window_range = window_range;

                let window_mismatch = if let Some(visible) =
                    state.metrics.visible_range(offset_axis, viewport, 0)
                {
                    match render_window_range.or(window_range).filter(|r| {
                        r.count == inputs.len
                            && r.overscan == inputs.overscan
                            && r.start_index <= r.end_index
                            && r.end_index < r.count
                    }) {
                        None => false,
                        Some(rendered) => {
                            let rendered_start =
                                rendered.start_index.saturating_sub(rendered.overscan);
                            let rendered_end = (rendered.end_index + rendered.overscan)
                                .min(rendered.count.saturating_sub(1));
                            visible.start_index < rendered_start || visible.end_index > rendered_end
                        }
                    }
                } else {
                    false
                };

                VirtualListPrepaintWindowUpdate {
                    prev_items_revision,
                    prev_viewport,
                    prev_offset,
                    prev_window_range,
                    render_window_range,
                    window_range,
                    viewport,
                    offset: offset_axis,
                    deferred_scroll_to_item,
                    window_mismatch,
                    content_extent,
                }
            },
        );

        inputs
            .scroll_handle
            .set_viewport_size_internal(record.bounds.size);
        let content_size = match inputs.axis {
            fret_core::Axis::Vertical => Size::new(record.bounds.size.width, update.content_extent),
            fret_core::Axis::Horizontal => {
                Size::new(update.content_extent, record.bounds.size.height)
            }
        };
        inputs.scroll_handle.set_content_size_internal(content_size);
        let prev_offset_point = inputs.scroll_handle.offset();
        match inputs.axis {
            fret_core::Axis::Vertical => {
                inputs
                    .scroll_handle
                    .set_offset_internal(fret_core::Point::new(prev_offset_point.x, update.offset));
            }
            fret_core::Axis::Horizontal => {
                inputs
                    .scroll_handle
                    .set_offset_internal(fret_core::Point::new(update.offset, prev_offset_point.y));
            }
        }

        if self.debug_enabled {
            self.debug_record_virtual_list_window(crate::tree::UiDebugVirtualListWindow {
                source: crate::tree::UiDebugVirtualListWindowSource::Prepaint,
                node: record.node,
                element: inputs.element,
                axis: inputs.axis,
                is_probe_layout: false,
                items_len: inputs.len,
                items_revision: inputs.items_revision,
                prev_items_revision: update.prev_items_revision,
                measure_mode: inputs.measure_mode,
                overscan: inputs.overscan,
                viewport: update.viewport,
                prev_viewport: update.prev_viewport,
                offset: update.offset,
                prev_offset: update.prev_offset,
                window_range: update.window_range,
                prev_window_range: update.prev_window_range,
                render_window_range: update.render_window_range,
                deferred_scroll_to_item: update.deferred_scroll_to_item,
                deferred_scroll_consumed: false,
                window_mismatch: update.window_mismatch,
            });
        }

        if self.view_cache_active() && update.window_mismatch {
            let retained_host =
                crate::elements::with_window_state(&mut *app, window, |window_state| {
                    let retained = window_state
                        .has_state::<crate::windowed_surface_host::RetainedVirtualListHostMarker>(
                        inputs.element,
                    );
                    if retained {
                        window_state.mark_retained_virtual_list_needs_reconcile(inputs.element);
                    }
                    retained
                });

            if !retained_host {
                self.mark_nearest_view_cache_root_needs_rerender(
                    record.node,
                    UiDebugInvalidationSource::Other,
                    UiDebugInvalidationDetail::ScrollHandleWindowUpdate,
                );
            }
            self.request_redraw_coalesced(app);
        }
    }

    fn apply_interaction_record(&mut self, record: &InteractionRecord) {
        let (prev, next) = {
            let Some(n) = self.nodes.get_mut(record.node) else {
                return;
            };
            let prev = n.invalidation;
            n.prepaint_hit_test = Some(super::PrepaintHitTestCache {
                render_transform_inv: record.render_transform_inv,
                children_render_transform_inv: record.children_render_transform_inv,
                clips_hit_test: record.clips_hit_test,
                clip_hit_test_corner_radii: record.clip_hit_test_corner_radii,
                is_focusable: record.is_focusable,
                focus_traversal_children: record.focus_traversal_children,
                can_scroll_descendant_into_view: record.can_scroll_descendant_into_view,
            });
            n.invalidation.hit_test = false;
            (prev, n.invalidation)
        };
        self.update_invalidation_counters(prev, next);
    }

    pub(super) fn prepaint_after_layout(&mut self, app: &mut H, scale_factor: f32) {
        if self.inspection_active {
            self.interaction_cache.invalidate_recording();
            self.hit_test_bounds_trees.clear();
            return;
        }

        let started = self.debug_enabled.then(Instant::now);
        if self.debug_enabled {
            self.begin_debug_frame_if_needed(app.frame_id());
            self.debug_stats.prepaint_time = Duration::default();
            self.debug_stats.prepaint_nodes_visited = 0;
            self.debug_stats.interaction_cache_hits = 0;
            self.debug_stats.interaction_cache_misses = 0;
            self.debug_stats.interaction_cache_replayed_records = 0;
            self.debug_stats.interaction_records = 0;
        }

        self.interaction_cache.begin_frame();
        self.hit_test_bounds_trees.begin_frame(app.frame_id());

        let theme_revision = Theme::global(&*app).revision();
        let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
        for layer_id in layers {
            let root = self.layers[layer_id].root;
            let hit_testable = self.layers[layer_id].hit_testable;

            let start = self.interaction_cache.records.len();
            self.prepaint_interaction_node(app, root, scale_factor, theme_revision);
            let end = self.interaction_cache.records.len();

            if hit_testable {
                let records = &self.interaction_cache.records[start..end];
                let nodes = &self.nodes;
                self.hit_test_bounds_trees
                    .rebuild_for_layer_from_records(root, records, nodes);
            }
        }

        self.interaction_cache.finish_frame();
        if self.debug_enabled {
            self.debug_stats.interaction_cache_hits = self.interaction_cache.hits;
            self.debug_stats.interaction_cache_misses = self.interaction_cache.misses;
            self.debug_stats.interaction_cache_replayed_records =
                self.interaction_cache.replayed_records;
            self.debug_stats.interaction_records = self.interaction_cache.records.len() as u32;
        }
        if let Some(started) = started {
            self.debug_stats.prepaint_time = started.elapsed();
        }
    }

    fn prepaint_interaction_node(
        &mut self,
        app: &mut H,
        node: NodeId,
        scale_factor: f32,
        theme_revision: u64,
    ) {
        if self.debug_enabled {
            self.debug_stats.prepaint_nodes_visited =
                self.debug_stats.prepaint_nodes_visited.saturating_add(1);
        }

        let (bounds, invalidation, is_view_cache_root, prev_cache, is_manual_cache_root) =
            match self.nodes.get(node) {
                Some(n) => (
                    n.bounds,
                    n.invalidation,
                    self.view_cache_active() && n.view_cache.enabled,
                    n.interaction_cache,
                    n.view_cache.enabled && n.element.is_none(),
                ),
                None => return,
            };

        let child_transform = self
            .node_children_render_transform(node)
            .unwrap_or(Transform2D::IDENTITY);
        let key = PaintCacheKey::new(bounds, scale_factor, theme_revision, child_transform);

        if is_view_cache_root && is_manual_cache_root {
            let contained_layout = self
                .nodes
                .get(node)
                .map(|n| n.view_cache.contained_layout)
                .unwrap_or(false);
            self.debug_record_view_cache_root(
                node,
                self.should_reuse_view_cache_node(node),
                contained_layout,
                crate::tree::UiDebugCacheRootReuseReason::ManualCacheRoot,
            );
        }

        if is_view_cache_root {
            let window = self.window;
            let sf = scale_factor;
            self.with_widget_mut(node, |widget, tree| {
                let mut cx = crate::widget::PrepaintCx {
                    app,
                    tree,
                    node,
                    window,
                    bounds,
                    scale_factor: sf,
                };
                widget.prepaint(&mut cx);
            });
        }

        let can_reuse =
            is_view_cache_root && self.should_reuse_view_cache_node(node) && !invalidation.hit_test;
        if can_reuse
            && let Some(prev) = prev_cache
            && prev.generation == self.interaction_cache.source_generation
            && prev.key == key
        {
            let range = prev.start as usize..prev.end as usize;
            if range.start <= range.end && range.end <= self.interaction_cache.prev_records.len() {
                let start = self.interaction_cache.records.len();
                self.interaction_cache.replay_scratch.clear();
                self.interaction_cache
                    .replay_scratch
                    .extend_from_slice(&self.interaction_cache.prev_records[range]);
                for i in 0..self.interaction_cache.replay_scratch.len() {
                    let record = self.interaction_cache.replay_scratch[i];
                    self.interaction_cache.records.push(record);
                    self.apply_interaction_record(&record);
                    self.prepaint_virtual_list_window_from_interaction_record(app, &record);
                }
                let end = self.interaction_cache.records.len();

                if let Some(n) = self.nodes.get_mut(node) {
                    n.interaction_cache = Some(InteractionCacheEntry {
                        generation: self.interaction_cache.target_generation,
                        key,
                        start: start as u32,
                        end: end as u32,
                    });
                }

                self.interaction_cache.hits = self.interaction_cache.hits.saturating_add(1);
                self.interaction_cache.replayed_records = self
                    .interaction_cache
                    .replayed_records
                    .saturating_add((end - start) as u32);
                return;
            }
        }

        if can_reuse {
            self.interaction_cache.misses = self.interaction_cache.misses.saturating_add(1);
        }

        let start = self.interaction_cache.records.len();
        let (render_transform, children_render_transform, clips_hit_test, corner_radii) =
            match self.nodes.get(node).and_then(|n| n.widget.as_ref()) {
                Some(widget) => {
                    let render_transform_inv =
                        widget.render_transform(bounds).and_then(|t| t.inverse());
                    let children_render_transform_inv = widget
                        .children_render_transform(bounds)
                        .and_then(|t| t.inverse());
                    (
                        render_transform_inv,
                        children_render_transform_inv,
                        widget.clips_hit_test(bounds),
                        widget.clip_hit_test_corner_radii(bounds),
                    )
                }
                None => (None, None, true, None),
            };
        let (is_focusable, focus_traversal_children, can_scroll_descendant_into_view) = self
            .nodes
            .get(node)
            .and_then(|n| n.widget.as_ref())
            .map(|w| {
                (
                    w.is_focusable(),
                    w.focus_traversal_children(),
                    w.can_scroll_descendant_into_view(),
                )
            })
            .unwrap_or((false, true, false));

        let record = InteractionRecord {
            node,
            bounds,
            render_transform_inv: render_transform,
            children_render_transform_inv: children_render_transform,
            clips_hit_test,
            clip_hit_test_corner_radii: corner_radii,
            is_focusable,
            focus_traversal_children,
            can_scroll_descendant_into_view,
        };
        self.interaction_cache.records.push(record);
        self.apply_interaction_record(&record);
        self.prepaint_virtual_list_window_from_interaction_record(app, &record);

        let mut children_buf = SmallNodeList::<32>::default();
        if let Some(children) = self.nodes.get(node).map(|n| n.children.as_slice()) {
            children_buf.set(children);
        }
        for &child in children_buf.as_slice() {
            self.prepaint_interaction_node(app, child, scale_factor, theme_revision);
        }

        let end = self.interaction_cache.records.len();
        if is_view_cache_root {
            if let Some(n) = self.nodes.get_mut(node) {
                n.interaction_cache = Some(InteractionCacheEntry {
                    generation: self.interaction_cache.target_generation,
                    key,
                    start: start as u32,
                    end: end as u32,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
                            scroll_margin: Px(0.0),
                            gap: Px(0.0),
                            scroll_handle: scroll_handle.clone(),
                            visible_items: Vec::new(),
                        },
                    ),
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
    }
}
