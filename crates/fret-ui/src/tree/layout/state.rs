use super::*;
use crate::layout_pass::LayoutPassKind;

impl<H: UiHost> UiTree<H> {
    fn virtual_list_scroll_handle_requires_layout(
        app: &mut H,
        window: AppWindowId,
        element: GlobalElementId,
        props: &crate::element::VirtualListProps,
    ) -> bool {
        crate::elements::with_element_state(
            &mut *app,
            window,
            element,
            crate::element::VirtualListState::default,
            |state| {
                state.metrics.ensure_with_mode(
                    props.measure_mode,
                    props.len,
                    props.estimate_row_height,
                    props.gap,
                    props.scroll_margin,
                );

                let viewport = match props.axis {
                    fret_core::Axis::Vertical => Px(state.viewport_h.0.max(0.0)),
                    fret_core::Axis::Horizontal => Px(state.viewport_w.0.max(0.0)),
                };
                if viewport.0 <= 0.0 || props.len == 0 {
                    return false;
                }

                let offset_point = props.scroll_handle.offset();
                let offset_axis = match props.axis {
                    fret_core::Axis::Vertical => offset_point.y,
                    fret_core::Axis::Horizontal => offset_point.x,
                };
                let offset_axis = state.metrics.clamp_offset(offset_axis, viewport);

                let Some(visible) = state.metrics.visible_range(offset_axis, viewport, 0) else {
                    return false;
                };

                let Some(window_range) =
                    state
                        .render_window_range
                        .or(state.window_range)
                        .filter(|r| {
                            r.count == props.len
                                && r.overscan == props.overscan
                                && r.start_index <= r.end_index
                                && r.end_index < r.count
                        })
                else {
                    return false;
                };

                let window_start = window_range
                    .start_index
                    .saturating_sub(window_range.overscan);
                let window_end = (window_range.end_index + window_range.overscan)
                    .min(window_range.count.saturating_sub(1));
                window_start > visible.start_index || window_end < visible.end_index
            },
        )
    }

    pub(crate) fn take_layout_engine(&mut self) -> crate::layout_engine::TaffyLayoutEngine {
        std::mem::take(&mut self.layout_engine)
    }

    pub(crate) fn put_layout_engine(&mut self, engine: crate::layout_engine::TaffyLayoutEngine) {
        self.layout_engine = engine;
    }

    pub(crate) fn register_viewport_root(&mut self, root: NodeId, bounds: Rect) {
        self.viewport_roots.push((root, bounds));
    }

    #[allow(dead_code)]
    pub(crate) fn viewport_roots(&self) -> &[(NodeId, Rect)] {
        &self.viewport_roots
    }

    pub(crate) fn interactive_resize_active(&self) -> bool {
        self.interactive_resize_active
    }

    pub(crate) fn update_interactive_resize_state_for_layout(
        &mut self,
        frame_id: FrameId,
        bounds: Rect,
        scale_factor: f32,
    ) {
        if self.interactive_resize_last_updated_frame == Some(frame_id) {
            return;
        }
        self.interactive_resize_last_updated_frame = Some(frame_id);

        let prev_bounds = self.last_layout_bounds;
        let prev_scale_bits = self.last_layout_scale_factor.map(|v| v.to_bits());
        let scale_bits = scale_factor.to_bits();
        let bounds_changed = prev_bounds.is_some_and(|prev| {
            prev.size.width.0.to_bits() != bounds.size.width.0.to_bits()
                || prev.size.height.0.to_bits() != bounds.size.height.0.to_bits()
        });
        let scale_changed = prev_scale_bits.is_some_and(|prev| prev != scale_bits);
        let changed = bounds_changed || scale_changed;

        if changed {
            self.interactive_resize_last_bounds_delta = if bounds_changed {
                prev_bounds.map(|prev| {
                    let dw = (bounds.size.width.0 - prev.size.width.0).abs();
                    let dh = (bounds.size.height.0 - prev.size.height.0).abs();
                    (fret_core::Px(dw), fret_core::Px(dh))
                })
            } else {
                None
            };
            self.interactive_resize_active = true;
            self.interactive_resize_stable_frames = 0;
            return;
        }

        self.interactive_resize_last_bounds_delta = None;

        if !self.interactive_resize_active {
            return;
        }

        let stable_frames_required = interactive_resize_stable_frames_required();
        if stable_frames_required == 0 {
            self.interactive_resize_active = false;
            self.interactive_resize_stable_frames = 0;
            return;
        }

        self.interactive_resize_stable_frames =
            self.interactive_resize_stable_frames.saturating_add(1);
        if self.interactive_resize_stable_frames >= stable_frames_required {
            self.interactive_resize_active = false;
            self.interactive_resize_stable_frames = 0;
        }
    }

    pub(crate) fn maybe_bucket_text_wrap_max_width(
        &self,
        wrap: fret_core::TextWrap,
        max_width: Option<fret_core::Px>,
    ) -> Option<fret_core::Px> {
        let max_width = max_width?;
        Some(self.maybe_bucket_text_wrap_width(wrap, max_width))
    }

    pub(crate) fn maybe_bucket_text_wrap_width(
        &self,
        wrap: fret_core::TextWrap,
        width: fret_core::Px,
    ) -> fret_core::Px {
        if !self.interactive_resize_active() {
            return width;
        }
        let mut bucket_px = text_wrap_width_bucket_px();
        if bucket_px <= 1 {
            // Default interactive-resize behavior: for small width jitters, quantize wrap widths
            // so we don't churn wrapped text layout every frame while the user is live-resizing.
            //
            // This intentionally does not apply to "stress" resizes that jump hundreds of pixels,
            // where we want accurate layout and can accept the one-off cost.
            //
            // The env knob still takes precedence; this is only a default for the common
            // "drag jitter" class. Treat small-step as symmetric (back-and-forth resizes should
            // keep the same policy/caches enabled).
            if !self.interactive_resize_is_small_step() {
                return width;
            }
            bucket_px = text_wrap_width_small_step_bucket_px();
            if bucket_px <= 1 {
                return width;
            }
        }
        match wrap {
            fret_core::TextWrap::Word
            | fret_core::TextWrap::WordBreak
            | fret_core::TextWrap::Grapheme => {
                let quantum = bucket_px as f32;
                if quantum <= 0.0 {
                    return width;
                }
                // Use a nearest-bucket snap (round) rather than `floor` so bucketing does not
                // systematically reduce wrap widths (which would create more lines and increase
                // layout/paint work under steady drag resizes).
                let snapped = (width.0 / quantum).round() * quantum;
                fret_core::Px(snapped.max(0.0))
            }
            fret_core::TextWrap::None => width,
        }
    }

    pub(crate) fn interactive_resize_is_small_step(&self) -> bool {
        self.interactive_resize_active()
            && self
                .interactive_resize_last_bounds_delta
                .is_some_and(|(dw, dh)| {
                    dw.0.abs() <= f32::from(text_wrap_width_small_step_max_dw_px())
                        && (dw.0 != 0.0 || dh.0 != 0.0)
                })
    }

    pub(crate) fn layout_engine_child_local_rect(
        &self,
        parent: NodeId,
        child: NodeId,
    ) -> Option<Rect> {
        self.layout_engine
            .child_layout_rect_if_solved(parent, child)
    }

    pub(crate) fn layout_engine_child_local_rect_profiled(
        &mut self,
        parent: NodeId,
        child: NodeId,
    ) -> Option<Rect> {
        let started = self.debug_enabled.then(Instant::now);
        let rect = self
            .layout_engine
            .child_layout_rect_if_solved(parent, child);
        if let Some(started) = started {
            self.debug_stats.layout_engine_child_rect_queries = self
                .debug_stats
                .layout_engine_child_rect_queries
                .saturating_add(1);
            self.debug_stats.layout_engine_child_rect_time += started.elapsed();
        }
        rect
    }

    #[allow(dead_code)]
    pub(crate) fn flow_subtree_is_engine_backed(&self, root: NodeId) -> bool {
        let Some(&child) = self.children(root).first() else {
            return false;
        };
        self.layout_engine_child_local_rect(root, child).is_some()
    }

    #[cfg(test)]
    pub(crate) fn layout_engine_has_node(&self, node: NodeId) -> bool {
        self.layout_engine.layout_id_for_node(node).is_some()
    }

    pub(crate) fn invalidate_scroll_handle_bindings_for_changed_handles(
        &mut self,
        app: &mut H,
        pass_kind: LayoutPassKind,
        consume_deferred_scroll_to_item: bool,
        commit_scroll_handle_baselines: bool,
    ) {
        if pass_kind != LayoutPassKind::Final {
            return;
        }
        let Some(window) = self.window else {
            return;
        };

        let consume_deferred_scroll_to_item =
            consume_deferred_scroll_to_item && commit_scroll_handle_baselines;
        let changed = if commit_scroll_handle_baselines {
            crate::declarative::frame::take_changed_scroll_handle_keys(app, window)
        } else {
            crate::declarative::frame::peek_changed_scroll_handle_keys(app, window)
        };
        if changed.is_empty() {
            return;
        }

        let mut visited = HashMap::<NodeId, u8>::new();
        let mut request_followup_redraw = false;
        for change in changed {
            let handle_key = change.handle_key;
            let bound = crate::declarative::frame::bound_elements_for_scroll_handle(
                &mut *app, window, handle_key,
            );
            if bound.is_empty() {
                continue;
            }

            // Scroll offset/viewport/content updates are classified as "HitTestOnly" by design to
            // avoid re-solving the whole layout engine for transform-only changes. However, many
            // higher-level behaviors (anchored overlays, poppers, etc.) rely on last-frame bounds
            // caches. To keep these overlays in sync after a scroll, schedule exactly one
            // follow-up redraw when scroll geometry changes.
            if change.offset_changed || change.viewport_changed || change.content_changed {
                request_followup_redraw = true;
            }

            let mut change_kind = change.kind;

            // If a virtual list requested a scroll-to-item, the scroll handle revision can bump
            // even when offset/viewport/content are unchanged, which makes the change appear as
            // "layout-affecting". Consume the deferred request up-front (using cached metrics +
            // viewport) and convert it into a simple offset update, avoiding a layout-driven
            // consumption path in the common case.
            //
            // Note: the diagnostics pipeline may classify these revisions as either `Layout` or
            // `HitTestOnly` depending on which stage observed the bump. Prefer consuming the
            // deferred request whenever we can.
            if consume_deferred_scroll_to_item {
                let mut consumed_scroll_to_item = false;
                for element in &bound {
                    if consumed_scroll_to_item {
                        break;
                    }
                    let Some(node) = crate::declarative::node_for_element_in_window_frame(
                        &mut *app, window, *element,
                    ) else {
                        continue;
                    };
                    let Some((
                        vlist_element,
                        vlist_axis,
                        vlist_len,
                        vlist_items_revision,
                        vlist_measure_mode,
                        _vlist_overscan,
                        vlist_estimate_row_height,
                        vlist_gap,
                        vlist_scroll_margin,
                        vlist_scroll_handle,
                    )) = crate::declarative::frame::with_element_record_for_node(
                        app,
                        window,
                        node,
                        |record| match &record.instance {
                            crate::declarative::frame::ElementInstance::VirtualList(props) => {
                                Some((
                                    record.element,
                                    props.axis,
                                    props.len,
                                    props.items_revision,
                                    props.measure_mode,
                                    props.overscan,
                                    props.estimate_row_height,
                                    props.gap,
                                    props.scroll_margin,
                                    props.scroll_handle.clone(),
                                ))
                            }
                            _ => None,
                        },
                    )
                    .flatten()
                    else {
                        continue;
                    };
                    let Some((index, strategy)) = vlist_scroll_handle.deferred_scroll_to_item()
                    else {
                        continue;
                    };

                    let applied = crate::elements::with_element_state(
                        &mut *app,
                        window,
                        vlist_element,
                        crate::element::VirtualListState::default,
                        |state| {
                            state.metrics.ensure_with_mode(
                                vlist_measure_mode,
                                vlist_len,
                                vlist_estimate_row_height,
                                vlist_gap,
                                vlist_scroll_margin,
                            );
                            state.metrics.sync_keys(&state.keys, vlist_items_revision);
                            state.items_revision = vlist_items_revision;

                            let viewport_from_state = match vlist_axis {
                                fret_core::Axis::Vertical => Px(state.viewport_h.0.max(0.0)),
                                fret_core::Axis::Horizontal => Px(state.viewport_w.0.max(0.0)),
                            };
                            let viewport_size = vlist_scroll_handle.viewport_size();
                            let viewport_from_handle = match vlist_axis {
                                fret_core::Axis::Vertical => Px(viewport_size.height.0.max(0.0)),
                                fret_core::Axis::Horizontal => Px(viewport_size.width.0.max(0.0)),
                            };
                            let viewport = if viewport_from_handle.0 > 0.0 {
                                viewport_from_handle
                            } else {
                                viewport_from_state
                            };
                            if viewport.0 <= 0.0 || vlist_len == 0 {
                                return None;
                            }

                            let current = match vlist_axis {
                                fret_core::Axis::Vertical => vlist_scroll_handle.offset().y,
                                fret_core::Axis::Horizontal => vlist_scroll_handle.offset().x,
                            };
                            let desired = state
                                .metrics
                                .scroll_offset_for_item(index, viewport, current, strategy);
                            let desired = state.metrics.clamp_offset(desired, viewport);

                            match vlist_axis {
                                fret_core::Axis::Vertical => state.offset_y = desired,
                                fret_core::Axis::Horizontal => state.offset_x = desired,
                            }

                            Some(desired)
                        },
                    );

                    let Some(applied) = applied else {
                        continue;
                    };

                    let prev = vlist_scroll_handle.offset();
                    match vlist_axis {
                        fret_core::Axis::Vertical => {
                            vlist_scroll_handle.set_offset(fret_core::Point::new(prev.x, applied));
                        }
                        fret_core::Axis::Horizontal => {
                            vlist_scroll_handle.set_offset(fret_core::Point::new(applied, prev.y));
                        }
                    }
                    vlist_scroll_handle.clear_deferred_scroll_to_item(app.frame_id());

                    consumed_scroll_to_item = true;
                    change_kind = crate::declarative::frame::ScrollHandleChangeKind::HitTestOnly;
                    self.request_redraw_coalesced(app);
                }
            }

            if self.debug_enabled && self.debug_scroll_handle_changes.len() < 256 {
                let mut upgraded_to_layout_bindings = 0u32;
                let mut bound_nodes_sample = Vec::new();
                for element in &bound {
                    if let Some(node) = crate::declarative::node_for_element_in_window_frame(
                        &mut *app, window, *element,
                    ) {
                        if bound_nodes_sample.len() < 8 {
                            bound_nodes_sample.push(node);
                        }

                        if change_kind
                            == crate::declarative::frame::ScrollHandleChangeKind::HitTestOnly
                            && let Some(record) = crate::declarative::frame::element_record_for_node(
                                &mut *app, window, node,
                            )
                            && let crate::declarative::frame::ElementInstance::VirtualList(props) =
                                &record.instance
                            && Self::virtual_list_scroll_handle_requires_layout(
                                &mut *app,
                                window,
                                record.element,
                                props,
                            )
                        {
                            upgraded_to_layout_bindings =
                                upgraded_to_layout_bindings.saturating_add(1);
                        }
                    }
                }

                self.debug_scroll_handle_changes
                    .push(crate::tree::UiDebugScrollHandleChange {
                        handle_key,
                        kind: match change_kind {
                            crate::declarative::frame::ScrollHandleChangeKind::Layout => {
                                crate::tree::UiDebugScrollHandleChangeKind::Layout
                            }
                            crate::declarative::frame::ScrollHandleChangeKind::HitTestOnly => {
                                crate::tree::UiDebugScrollHandleChangeKind::HitTestOnly
                            }
                        },
                        revision: change.revision,
                        prev_revision: change.prev_revision,
                        offset: change.offset,
                        prev_offset: change.prev_offset,
                        viewport: change.viewport,
                        prev_viewport: change.prev_viewport,
                        content: change.content,
                        prev_content: change.prev_content,
                        offset_changed: change.offset_changed,
                        viewport_changed: change.viewport_changed,
                        content_changed: change.content_changed,
                        bound_elements: bound.len() as u32,
                        bound_nodes_sample,
                        upgraded_to_layout_bindings,
                    });
            }

            for element in bound {
                let Some(node) = crate::declarative::node_for_element_in_window_frame(
                    &mut *app, window, element,
                ) else {
                    continue;
                };

                let mut inv = match change_kind {
                    crate::declarative::frame::ScrollHandleChangeKind::Layout => {
                        Invalidation::Layout
                    }
                    crate::declarative::frame::ScrollHandleChangeKind::HitTestOnly => {
                        Invalidation::HitTestOnly
                    }
                };
                let mut detail = match change_kind {
                    crate::declarative::frame::ScrollHandleChangeKind::Layout => {
                        UiDebugInvalidationDetail::ScrollHandleLayout
                    }
                    crate::declarative::frame::ScrollHandleChangeKind::HitTestOnly => {
                        UiDebugInvalidationDetail::ScrollHandleHitTestOnly
                    }
                };

                // A scroll handle can see multiple updates during a single layout pass when the
                // same handle is (incorrectly) shared across multiple scroll surfaces (e.g. a
                // horizontal scroll handle reused per-row in a table). This can bump the handle
                // revision even when the final observed offset/viewport/content are unchanged.
                //
                // Treat these "revision-only" changes as HitTestOnly by default. Upgrade to
                // Layout only for VirtualList cases where we must consume a deferred scroll
                // request, or when the visible window leaves the last rendered overscan window.
                if inv == Invalidation::Layout
                    && !change.offset_changed
                    && !change.viewport_changed
                    && !change.content_changed
                {
                    inv = Invalidation::HitTestOnly;
                    detail = UiDebugInvalidationDetail::ScrollHandleHitTestOnly;
                }

                if inv == Invalidation::HitTestOnly
                    && let Some(record) =
                        crate::declarative::frame::element_record_for_node(&mut *app, window, node)
                    && let crate::declarative::frame::ElementInstance::Scroll(scroll_props) =
                        &record.instance
                    && scroll_props.windowed_paint
                {
                    // Windowed paint surfaces (ADR 0175) depend on the scroll offset to determine
                    // which content is painted into the scrollable space. When view-cache reuse is
                    // enabled, a scroll transform update alone is insufficient: the cached subtree
                    // must be allowed to rerender so its paint handlers can run for the new visible
                    // window. Without this, scroll can appear to show stale content.
                    if self.view_cache_enabled() && change.offset_changed {
                        self.mark_nearest_view_cache_root_needs_rerender(
                            node,
                            UiDebugInvalidationSource::Other,
                            UiDebugInvalidationDetail::ScrollHandleWindowUpdate,
                        );
                        self.request_redraw_coalesced(app);
                    }
                } else if inv == Invalidation::HitTestOnly
                    && let Some(record) =
                        crate::declarative::frame::element_record_for_node(&mut *app, window, node)
                    && let crate::declarative::frame::ElementInstance::VirtualList(props) =
                        &record.instance
                {
                    let requires_deferred_consumption =
                        props.scroll_handle.deferred_scroll_to_item().is_some();
                    let requires_window_update = Self::virtual_list_scroll_handle_requires_layout(
                        &mut *app,
                        window,
                        record.element,
                        props,
                    );

                    // Keep element-local scroll state in sync for scroll-handle changes that are
                    // treated as HitTestOnly (wheel/inertial/transform-only updates). This avoids
                    // a "layout-or-nothing" coupling for consumers that observe `VirtualListState`.
                    crate::elements::with_element_state(
                        &mut *app,
                        window,
                        record.element,
                        crate::element::VirtualListState::default,
                        |state| {
                            state.metrics.ensure_with_mode(
                                props.measure_mode,
                                props.len,
                                props.estimate_row_height,
                                props.gap,
                                props.scroll_margin,
                            );

                            let viewport = match props.axis {
                                fret_core::Axis::Vertical => Px(state.viewport_h.0.max(0.0)),
                                fret_core::Axis::Horizontal => Px(state.viewport_w.0.max(0.0)),
                            };
                            if viewport.0 <= 0.0 || props.len == 0 {
                                return;
                            }

                            let offset_point = props.scroll_handle.offset();
                            let offset_axis = match props.axis {
                                fret_core::Axis::Vertical => offset_point.y,
                                fret_core::Axis::Horizontal => offset_point.x,
                            };
                            let offset_axis = state.metrics.clamp_offset(offset_axis, viewport);
                            match props.axis {
                                fret_core::Axis::Vertical => state.offset_y = offset_axis,
                                fret_core::Axis::Horizontal => state.offset_x = offset_axis,
                            }

                            state.window_range =
                                state
                                    .metrics
                                    .visible_range(offset_axis, viewport, props.overscan);
                        },
                    );

                    if requires_deferred_consumption {
                        inv = Invalidation::Layout;
                        detail = UiDebugInvalidationDetail::ScrollHandleLayout;
                    } else if requires_window_update {
                        let retained_host = crate::elements::with_window_state(
                            &mut *app,
                            window,
                            |window_state| {
                                let retained = window_state.has_state::<
                                    crate::windowed_surface_host::RetainedVirtualListHostMarker,
                                >(record.element);
                                if retained {
                                    window_state
                                        .mark_retained_virtual_list_needs_reconcile(
                                            record.element,
                                            crate::tree::UiDebugRetainedVirtualListReconcileKind::Escape,
                                        );
                                }
                                retained
                            },
                        );

                        if retained_host {
                            // Retained-host virtual surfaces can update row membership without
                            // rerendering the parent cache root (ADR 0177). Schedule a redraw so
                            // `render_root` can reconcile row subtrees in the next frame.
                            self.request_redraw_coalesced(app);
                        } else {
                            // Do not force a layout pass just to discover that the visible window
                            // is outside the previously rendered overscan window. Instead, treat
                            // it as a prepaint-windowed "ephemeral update" signal (ADR 0175):
                            // mark the nearest view-cache root dirty and request a redraw so the
                            // next frame rerenders the virtual surface children.
                            self.mark_nearest_view_cache_root_needs_rerender(
                                node,
                                UiDebugInvalidationSource::Other,
                                UiDebugInvalidationDetail::ScrollHandleWindowUpdate,
                            );
                            self.request_redraw_coalesced(app);
                        }
                    }
                }

                self.mark_invalidation_dedup_with_detail(
                    node,
                    inv,
                    &mut visited,
                    UiDebugInvalidationSource::Other,
                    detail,
                );
            }
        }

        if request_followup_redraw {
            self.request_redraw_coalesced(app);
        }
    }
}
