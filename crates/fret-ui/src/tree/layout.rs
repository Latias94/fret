use super::*;
use std::any::TypeId;

use crate::layout_constraints::LayoutSize;
use crate::layout_constraints::{AvailableSpace, LayoutConstraints};
use crate::layout_engine::TaffyLayoutEngine;
use crate::layout_engine::build_viewport_flow_subtree;
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

            // If a virtual list requested a scroll-to-item, the scroll handle revision bumps even
            // when offset/viewport/content are unchanged, which makes the change appear as
            // "layout-affecting". Consume the deferred request up-front (using cached metrics +
            // viewport) and convert it into a simple offset update, avoiding a layout-driven
            // consumption path in the common case.
            if consume_deferred_scroll_to_item
                && change_kind == crate::declarative::frame::ScrollHandleChangeKind::Layout
            {
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

                            let viewport_size = vlist_scroll_handle.viewport_size();
                            let viewport = match vlist_axis {
                                fret_core::Axis::Vertical => Px(viewport_size.height.0.max(0.0)),
                                fret_core::Axis::Horizontal => Px(viewport_size.width.0.max(0.0)),
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
                    // Windowed paint surfaces (ADR 0190) depend on the scroll offset to determine
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
                            // rerendering the parent cache root (ADR 0192). Schedule a redraw so
                            // `render_root` can reconcile row subtrees in the next frame.
                            self.request_redraw_coalesced(app);
                        } else {
                            // Do not force a layout pass just to discover that the visible window
                            // is outside the previously rendered overscan window. Instead, treat
                            // it as a prepaint-windowed "ephemeral update" signal (ADR 0190):
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

    pub fn layout_all(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        bounds: Rect,
        scale_factor: f32,
    ) {
        self.layout_all_with_pass_kind(app, services, bounds, scale_factor, LayoutPassKind::Final);
    }

    #[stacksafe::stacksafe]
    pub(crate) fn layout_all_with_pass_kind(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        bounds: Rect,
        scale_factor: f32,
        pass_kind: LayoutPassKind,
    ) {
        let profile_layout_all = std::env::var_os("FRET_LAYOUT_ALL_PROFILE")
            .is_some_and(|v| !v.is_empty() && v != "0")
            && pass_kind == LayoutPassKind::Final;
        let profile_started = profile_layout_all.then(Instant::now);
        let mut t_invalidate_scroll_handle_bindings: Option<Duration> = None;
        let mut t_expand_view_cache_invalidations: Option<Duration> = None;
        let mut t_request_build_roots: Option<Duration> = None;
        let mut t_layout_roots: Option<Duration> = None;
        let mut t_pending_barriers: Option<Duration> = None;
        let mut t_repair_view_cache_bounds: Option<Duration> = None;
        let mut t_layout_contained_view_cache_roots: Option<Duration> = None;
        let mut t_collapse_layout_observations: Option<Duration> = None;
        let mut t_refresh_semantics: Option<Duration> = None;
        let mut t_prepaint_after_layout: Option<Duration> = None;
        let mut t_flush_deferred_cleanup: Option<Duration> = None;

        if pass_kind == LayoutPassKind::Final {
            self.layout_node_profile = LayoutNodeProfileConfig::from_env()
                .map(|cfg| LayoutNodeProfileState::new(cfg, app.frame_id()));
            self.measure_node_profile = MeasureNodeProfileConfig::from_env()
                .map(|cfg| MeasureNodeProfileState::new(cfg, app.frame_id()));
        } else {
            self.layout_node_profile = None;
            self.measure_node_profile = None;
        }

        self.measure_cache_this_frame.clear();

        if pass_kind == LayoutPassKind::Final {
            self.update_interactive_resize_state_for_layout(app.frame_id(), bounds, scale_factor);
        }

        let started = self.debug_enabled.then(Instant::now);
        if self.debug_enabled {
            self.begin_debug_frame_if_needed(app.frame_id());
            self.debug_stats.frame_id = app.frame_id();
            self.debug_stats.layout_nodes_visited = 0;
            self.debug_stats.layout_nodes_performed = 0;
            self.debug_stats.layout_engine_solves = 0;
            self.debug_stats.layout_engine_solve_time = Duration::default();
            self.debug_stats.layout_engine_child_rect_queries = 0;
            self.debug_stats.layout_engine_child_rect_time = Duration::default();
            self.debug_stats.layout_engine_widget_fallback_solves = 0;
            self.debug_stats.layout_collect_roots_time = Duration::default();
            self.debug_stats
                .layout_invalidate_scroll_handle_bindings_time = Duration::default();
            self.debug_stats.layout_expand_view_cache_invalidations_time = Duration::default();
            self.debug_stats.layout_request_build_roots_time = Duration::default();
            self.debug_stats.layout_pending_barrier_relayouts_time = Duration::default();
            self.debug_stats.layout_repair_view_cache_bounds_time = Duration::default();
            self.debug_stats.layout_contained_view_cache_roots_time = Duration::default();
            self.debug_stats.layout_collapse_layout_observations_time = Duration::default();
            self.debug_stats.layout_prepaint_after_layout_time = Duration::default();
            self.debug_stats.layout_skipped_engine_frame = false;
            self.debug_stats.layout_fast_path_taken = false;
            self.debug_stats.layout_invalidations_count = self.layout_invalidations_count;
            self.debug_stats.view_cache_active = self.view_cache_active();
            self.debug_stats.focus = self.focus;
            self.debug_stats.captured = self.captured_for(fret_core::PointerId(0));
        }

        let roots_started = self.debug_enabled.then(Instant::now);
        let roots: Vec<NodeId> = self
            .visible_layers_in_paint_order()
            .map(|layer| self.layers[layer].root)
            .collect();
        if let Some(roots_started) = roots_started {
            self.debug_stats.layout_collect_roots_time += roots_started.elapsed();
        }

        let mut viewport_cursor: usize = 0;

        let scroll_started = self.debug_enabled.then(Instant::now);
        let started_phase = profile_layout_all.then(Instant::now);
        self.invalidate_scroll_handle_bindings_for_changed_handles(app, pass_kind, true, true);
        if let Some(started) = started_phase {
            t_invalidate_scroll_handle_bindings = Some(started.elapsed());
        }
        if let Some(scroll_started) = scroll_started {
            self.debug_stats
                .layout_invalidate_scroll_handle_bindings_time += scroll_started.elapsed();
        }

        let any_root_needs_layout_or_bounds = roots.iter().any(|&root| {
            self.nodes
                .get(root)
                .is_some_and(|node| node.invalidation.layout || node.bounds != bounds)
        });
        let any_pending_barrier_needs_layout = self.pending_barrier_relayouts.iter().any(|&root| {
            self.nodes
                .get(root)
                .is_some_and(|node| node.invalidation.layout)
        });
        let any_view_cache_root_needs_layout = self.view_cache_active()
            && self
                .nodes
                .iter()
                .any(|(_, node)| node.view_cache.enabled && node.invalidation.layout);

        if pass_kind == LayoutPassKind::Final
            && !any_root_needs_layout_or_bounds
            && !any_view_cache_root_needs_layout
            && !any_pending_barrier_needs_layout
            && self.invalidated_paint_nodes == 0
            && self.invalidated_hit_test_nodes == 0
        {
            self.pending_barrier_relayouts.retain(|&root| {
                self.nodes
                    .get(root)
                    .is_some_and(|node| node.invalidation.layout)
            });
            if self.semantics_requested {
                let semantics_started = self.debug_enabled.then(Instant::now);
                self.semantics_requested = false;
                self.refresh_semantics_snapshot(app);
                if let Some(semantics_started) = semantics_started {
                    self.debug_stats.layout_semantics_refresh_time += semantics_started.elapsed();
                }
            }
            let prepaint_started = self.debug_enabled.then(Instant::now);
            self.prepaint_after_layout_stable_frame(app);
            if let Some(prepaint_started) = prepaint_started {
                self.debug_stats.layout_prepaint_after_layout_time += prepaint_started.elapsed();
            }
            self.debug_stats.layout_skipped_engine_frame = true;

            let focus_started = self.debug_enabled.then(Instant::now);
            self.repair_focus_node_from_focused_element_if_needed(app);
            if let Some(focus_started) = focus_started {
                self.debug_stats.layout_focus_repair_time += focus_started.elapsed();
            }

            let deferred_cleanup_started = self.debug_enabled.then(Instant::now);
            self.flush_deferred_cleanup(services);
            if let Some(deferred_cleanup_started) = deferred_cleanup_started {
                self.debug_stats.layout_deferred_cleanup_time += deferred_cleanup_started.elapsed();
            }
            if let Some(started) = started {
                self.debug_stats.layout_time = started
                    .elapsed()
                    .saturating_sub(self.debug_stats.layout_prepaint_after_layout_time);
            }
            return;
        }

        if pass_kind == LayoutPassKind::Final {
            let expand_started = self.debug_enabled.then(Instant::now);
            let started_phase = profile_layout_all.then(Instant::now);
            self.expand_view_cache_layout_invalidations_if_needed();
            if let Some(started) = started_phase {
                t_expand_view_cache_invalidations = Some(started.elapsed());
            }
            if let Some(expand_started) = expand_started {
                self.debug_stats.layout_expand_view_cache_invalidations_time +=
                    expand_started.elapsed();
            }
        }

        // Fast path (ADR 0190): if nothing requires layout this frame, skip the layout engine and
        // only run prepaint/semantics. This keeps scroll-only and cache-hit frames cheap while
        // still allowing prepaint-windowed surfaces to update their ephemeral outputs.
        if pass_kind == LayoutPassKind::Final
            && self.pending_barrier_relayouts.is_empty()
            && self.last_layout_bounds == Some(bounds)
            && self.last_layout_scale_factor == Some(scale_factor)
            && self.layout_invalidations_count == 0
        {
            self.debug_stats.layout_fast_path_taken = true;
            self.prepaint_after_layout(app, scale_factor);

            if self.semantics_requested {
                self.semantics_requested = false;
                self.refresh_semantics_snapshot(app);
            }
            self.flush_deferred_cleanup(services);

            self.last_layout_bounds = Some(bounds);
            self.last_layout_scale_factor = Some(scale_factor);

            if let Some(started) = started {
                self.debug_stats.layout_time = started.elapsed();
            }
            return;
        }

        let (layout_engine_solves_start, layout_engine_solve_time_start) = {
            self.begin_layout_engine_frame(app);
            if self.debug_enabled {
                (
                    self.layout_engine.solve_count(),
                    self.layout_engine.last_solve_time(),
                )
            } else {
                (0, Duration::default())
            }
        };

        let started_phase = profile_layout_all.then(Instant::now);
        let request_build_started = self.debug_enabled.then(Instant::now);
        self.request_build_window_roots_if_final(
            app,
            services,
            &roots,
            bounds,
            scale_factor,
            pass_kind,
        );
        if let Some(started) = started_phase {
            t_request_build_roots = Some(started.elapsed());
        }
        if let Some(request_build_started) = request_build_started {
            self.debug_stats.layout_request_build_roots_time += request_build_started.elapsed();
        }

        let started_phase = profile_layout_all.then(Instant::now);
        let roots_started = self.debug_enabled.then(Instant::now);
        for root in roots {
            let _ =
                self.layout_in_with_pass_kind(app, services, root, bounds, scale_factor, pass_kind);

            self.flush_viewport_roots_after_root(
                app,
                services,
                scale_factor,
                pass_kind,
                &mut viewport_cursor,
            );
        }
        if let Some(roots_started) = roots_started {
            self.debug_stats.layout_roots_time += roots_started.elapsed();
        }
        if let Some(started) = started_phase {
            t_layout_roots = Some(started.elapsed());
        }

        if pass_kind == LayoutPassKind::Final {
            let barrier_started = self.debug_enabled.then(Instant::now);
            let started_phase = profile_layout_all.then(Instant::now);
            self.layout_pending_barrier_relayouts_if_needed(
                app,
                services,
                scale_factor,
                pass_kind,
                &mut viewport_cursor,
            );
            if let Some(started) = started_phase {
                t_pending_barriers = Some(started.elapsed());
            }
            if let Some(barrier_started) = barrier_started {
                let elapsed = barrier_started.elapsed();
                self.debug_stats.layout_barrier_relayouts_time += elapsed;
                self.debug_stats.layout_pending_barrier_relayouts_time += elapsed;
            }
        }

        if pass_kind == LayoutPassKind::Final {
            let view_cache_started = self.debug_enabled.then(Instant::now);

            let started_phase = profile_layout_all.then(Instant::now);
            let repair_started = self.debug_enabled.then(Instant::now);
            self.repair_view_cache_root_bounds_from_engine_if_needed(app);
            if let Some(started) = started_phase {
                t_repair_view_cache_bounds = Some(started.elapsed());
            }
            if let Some(repair_started) = repair_started {
                self.debug_stats.layout_repair_view_cache_bounds_time += repair_started.elapsed();
            }

            let started_phase = profile_layout_all.then(Instant::now);
            let contained_started = self.debug_enabled.then(Instant::now);
            self.layout_contained_view_cache_roots_if_needed(
                app,
                services,
                scale_factor,
                pass_kind,
                &mut viewport_cursor,
            );
            if let Some(started) = started_phase {
                t_layout_contained_view_cache_roots = Some(started.elapsed());
            }
            if let Some(contained_started) = contained_started {
                self.debug_stats.layout_contained_view_cache_roots_time +=
                    contained_started.elapsed();
            }

            let started_phase = profile_layout_all.then(Instant::now);
            let collapse_started = self.debug_enabled.then(Instant::now);
            self.collapse_layout_observations_to_view_cache_roots_if_needed();
            if let Some(started) = started_phase {
                t_collapse_layout_observations = Some(started.elapsed());
            }
            if let Some(collapse_started) = collapse_started {
                self.debug_stats.layout_collapse_layout_observations_time +=
                    collapse_started.elapsed();
            }

            if let Some(view_cache_started) = view_cache_started {
                self.debug_stats.layout_view_cache_time += view_cache_started.elapsed();
            }
        }

        if self.semantics_requested {
            let semantics_started = self.debug_enabled.then(Instant::now);
            let started_phase = profile_layout_all.then(Instant::now);
            self.semantics_requested = false;
            self.refresh_semantics_snapshot(app);
            if let Some(started) = started_phase {
                t_refresh_semantics = Some(started.elapsed());
            }
            if let Some(semantics_started) = semantics_started {
                self.debug_stats.layout_semantics_refresh_time += semantics_started.elapsed();
            }
        }
        if pass_kind == LayoutPassKind::Final {
            let prepaint_started = self.debug_enabled.then(Instant::now);
            let started_phase = profile_layout_all.then(Instant::now);
            self.prepaint_after_layout(app, scale_factor);
            if let Some(started) = started_phase {
                t_prepaint_after_layout = Some(started.elapsed());
            }
            if let Some(prepaint_started) = prepaint_started {
                self.debug_stats.layout_prepaint_after_layout_time += prepaint_started.elapsed();
            }
        }
        let started_phase = profile_layout_all.then(Instant::now);
        if pass_kind == LayoutPassKind::Final {
            let focus_started = self.debug_enabled.then(Instant::now);
            self.repair_focus_node_from_focused_element_if_needed(app);
            if let Some(focus_started) = focus_started {
                self.debug_stats.layout_focus_repair_time += focus_started.elapsed();
            }
        }
        let deferred_cleanup_started = self.debug_enabled.then(Instant::now);
        self.flush_deferred_cleanup(services);
        if let Some(started) = started_phase {
            t_flush_deferred_cleanup = Some(started.elapsed());
        }
        if let Some(deferred_cleanup_started) = deferred_cleanup_started {
            self.debug_stats.layout_deferred_cleanup_time += deferred_cleanup_started.elapsed();
        }

        if let Some(started) = started {
            self.debug_stats.layout_time = started
                .elapsed()
                .saturating_sub(self.debug_stats.layout_prepaint_after_layout_time);
        }

        if pass_kind == LayoutPassKind::Final {
            self.layout_engine.end_frame();
            if let Some(window) = self.window {
                let frame_id = app.frame_id();
                crate::elements::with_window_state(app, window, |st| {
                    st.clear_stale_interaction_targets_for_frame(frame_id);
                });
            }
        }

        if pass_kind == LayoutPassKind::Final {
            self.last_layout_bounds = Some(bounds);
            self.last_layout_scale_factor = Some(scale_factor);
        }

        if self.debug_enabled {
            self.debug_stats.layout_engine_solves = self
                .layout_engine
                .solve_count()
                .saturating_sub(layout_engine_solves_start);
            self.debug_stats.layout_engine_solve_time = self
                .layout_engine
                .last_solve_time()
                .saturating_sub(layout_engine_solve_time_start);
        }

        if let Some(started) = profile_started {
            let total = started.elapsed();
            tracing::info!(
                window = ?self.window,
                total_ms = total.as_millis(),
                invalidate_scroll_handle_bindings_ms =
                    t_invalidate_scroll_handle_bindings.map(|d| d.as_millis()),
                expand_view_cache_invalidations_ms =
                    t_expand_view_cache_invalidations.map(|d| d.as_millis()),
                request_build_roots_ms = t_request_build_roots.map(|d| d.as_millis()),
                layout_roots_ms = t_layout_roots.map(|d| d.as_millis()),
                pending_barriers_ms = t_pending_barriers.map(|d| d.as_millis()),
                repair_view_cache_bounds_ms = t_repair_view_cache_bounds.map(|d| d.as_millis()),
                layout_contained_view_cache_roots_ms =
                    t_layout_contained_view_cache_roots.map(|d| d.as_millis()),
                collapse_layout_observations_ms = t_collapse_layout_observations.map(|d| d.as_millis()),
                refresh_semantics_ms = t_refresh_semantics.map(|d| d.as_millis()),
                prepaint_after_layout_ms = t_prepaint_after_layout.map(|d| d.as_millis()),
                flush_deferred_cleanup_ms = t_flush_deferred_cleanup.map(|d| d.as_millis()),
                layout_nodes_performed = self.debug_stats.layout_nodes_performed,
                "layout_all profile"
            );
        }

        if pass_kind == LayoutPassKind::Final {
            self.emit_layout_node_profile(app);
            self.emit_measure_node_profile(app);
        }
    }

    fn emit_layout_node_profile(&mut self, app: &mut H) {
        let Some(profile) = self.layout_node_profile.take() else {
            return;
        };
        if profile.entries.is_empty() {
            return;
        }
        let Some(window) = self.window else {
            return;
        };

        let mut test_id_by_node: HashMap<NodeId, String> = HashMap::new();
        if let Some(snapshot) = self.semantics_snapshot() {
            for node in &snapshot.nodes {
                if let Some(test_id) = node.test_id.as_deref() {
                    test_id_by_node.insert(node.id, test_id.to_string());
                }
            }
        }

        let resolve_test_id = |tree: &UiTree<H>, id: NodeId| -> Option<&str> {
            let mut cur = Some(id);
            while let Some(node) = cur {
                if let Some(test_id) = test_id_by_node.get(&node) {
                    return Some(test_id.as_str());
                }
                cur = tree.nodes.get(node).and_then(|n| n.parent);
            }
            None
        };

        for (rank, entry) in profile.entries.iter().enumerate() {
            let kind = crate::declarative::frame::element_record_for_node(app, window, entry.node)
                .map(|r| r.instance.kind_name());

            let element_path: Option<String> = self
                .nodes
                .get(entry.node)
                .and_then(|n| n.element)
                .and_then(|element| {
                    #[cfg(feature = "diagnostics")]
                    {
                        crate::elements::with_window_state(app, window, |st| {
                            st.debug_path_for_element(element)
                        })
                    }
                    #[cfg(not(feature = "diagnostics"))]
                    {
                        let _ = element;
                        None
                    }
                });

            tracing::info!(
                window = ?self.window,
                frame_id = profile.frame_id.0,
                nodes_profiled = profile.nodes_profiled,
                total_self_ms = profile.total_self_time.as_millis() as u64,
                rank,
                node = ?entry.node,
                pass = ?entry.pass_kind,
                self_us = entry.elapsed_self.as_micros() as u64,
                total_us = entry.elapsed_total.as_micros() as u64,
                kind = kind.unwrap_or("<unknown>"),
                test_id = resolve_test_id(self, entry.node),
                element_path = element_path.as_deref().unwrap_or("<unknown>"),
                bounds_w = entry.bounds.size.width.0,
                bounds_h = entry.bounds.size.height.0,
                "layout_node profile"
            );
        }
    }

    fn emit_measure_node_profile(&mut self, app: &mut H) {
        let Some(profile) = self.measure_node_profile.take() else {
            return;
        };
        if profile.entries.is_empty() {
            return;
        }
        let Some(window) = self.window else {
            return;
        };

        let mut test_id_by_node: HashMap<NodeId, String> = HashMap::new();
        if let Some(snapshot) = self.semantics_snapshot() {
            for node in &snapshot.nodes {
                if let Some(test_id) = node.test_id.as_deref() {
                    test_id_by_node.insert(node.id, test_id.to_string());
                }
            }
        }

        let resolve_test_id = |tree: &UiTree<H>, id: NodeId| -> Option<&str> {
            let mut cur = Some(id);
            while let Some(node) = cur {
                if let Some(test_id) = test_id_by_node.get(&node) {
                    return Some(test_id.as_str());
                }
                cur = tree.nodes.get(node).and_then(|n| n.parent);
            }
            None
        };

        for (rank, entry) in profile.entries.iter().enumerate() {
            let kind = crate::declarative::frame::element_record_for_node(app, window, entry.node)
                .map(|r| r.instance.kind_name());

            let element_path: Option<String> = self
                .nodes
                .get(entry.node)
                .and_then(|n| n.element)
                .and_then(|element| {
                    #[cfg(feature = "diagnostics")]
                    {
                        crate::elements::with_window_state(app, window, |st| {
                            st.debug_path_for_element(element)
                        })
                    }
                    #[cfg(not(feature = "diagnostics"))]
                    {
                        let _ = element;
                        None
                    }
                });

            tracing::info!(
                window = ?self.window,
                frame_id = profile.frame_id.0,
                nodes_profiled = profile.nodes_profiled,
                total_self_ms = profile.total_self_time.as_millis() as u64,
                rank,
                node = ?entry.node,
                self_us = entry.elapsed_self.as_micros() as u64,
                total_us = entry.elapsed_total.as_micros() as u64,
                kind = kind.unwrap_or("<unknown>"),
                test_id = resolve_test_id(self, entry.node),
                element_path = element_path.as_deref().unwrap_or("<unknown>"),
                known_w = entry.constraints.known.width.map(|p| p.0),
                known_h = entry.constraints.known.height.map(|p| p.0),
                avail_w = ?entry.constraints.available.width,
                avail_h = ?entry.constraints.available.height,
                "measure_node profile"
            );
        }
    }

    fn repair_focus_node_from_focused_element_if_needed(&mut self, app: &mut H) {
        let Some(window) = self.window else {
            return;
        };
        let Some(focused) = self.focus() else {
            return;
        };
        let Some(element) = self.node_element(focused) else {
            #[cfg(debug_assertions)]
            if std::env::var_os("FRET_DEBUG_FOCUS_REPAIR").is_some() {
                eprintln!("focus_repair: focused={focused:?} has no element");
            }
            return;
        };
        let Some(canonical) = crate::elements::node_for_element(app, window, element) else {
            #[cfg(debug_assertions)]
            if std::env::var_os("FRET_DEBUG_FOCUS_REPAIR").is_some() {
                eprintln!(
                    "focus_repair: focused={focused:?} element={element:?} has no canonical node",
                );
            }
            return;
        };
        #[cfg(debug_assertions)]
        if std::env::var_os("FRET_DEBUG_FOCUS_REPAIR").is_some() {
            eprintln!(
                "focus_repair: focused={focused:?} element={element:?} canonical={canonical:?} canonical_exists={}",
                self.node_exists(canonical)
            );
        }
        if canonical != focused && self.node_exists(canonical) {
            self.set_focus(Some(canonical));
        }
    }

    fn repair_view_cache_root_bounds_from_engine_if_needed(&mut self, app: &mut H) {
        if !self.view_cache_active() {
            return;
        }
        let Some(window) = self.window else {
            return;
        };

        let mut targets: Vec<(NodeId, Rect, Point)> = Vec::with_capacity(16);
        for (id, node) in self.nodes.iter() {
            if !node.view_cache.enabled {
                continue;
            }
            if node.bounds.size != Size::default() {
                continue;
            }
            let Some(parent) = node.parent else {
                continue;
            };
            let Some(parent_bounds) = self.nodes.get(parent).map(|n| n.bounds) else {
                continue;
            };
            let Some(local) = self.layout_engine_child_local_rect(parent, id) else {
                continue;
            };

            let origin = Point::new(
                Px(parent_bounds.origin.x.0 + local.origin.x.0),
                Px(parent_bounds.origin.y.0 + local.origin.y.0),
            );
            let new_bounds = Rect::new(origin, local.size);
            targets.push((id, new_bounds, node.bounds.origin));
        }

        for (root, new_bounds, old_origin) in targets {
            let delta = Point::new(
                Px(new_bounds.origin.x.0 - old_origin.x.0),
                Px(new_bounds.origin.y.0 - old_origin.y.0),
            );

            if let Some(node) = self.nodes.get_mut(root) {
                node.bounds = new_bounds;
            }
            if let Some(element) = self.nodes.get(root).and_then(|n| n.element) {
                crate::elements::record_bounds_for_element(app, window, element, new_bounds);
            }

            if delta.x.0 == 0.0 && delta.y.0 == 0.0 {
                continue;
            }

            let mut stack: Vec<NodeId> = self
                .nodes
                .get(root)
                .map(|n| n.children.clone())
                .unwrap_or_default();
            while let Some(id) = stack.pop() {
                let Some(n) = self.nodes.get_mut(id) else {
                    continue;
                };
                n.bounds.origin = Point::new(
                    Px(n.bounds.origin.x.0 + delta.x.0),
                    Px(n.bounds.origin.y.0 + delta.y.0),
                );
                if let Some(element) = n.element {
                    crate::elements::record_bounds_for_element(app, window, element, n.bounds);
                }
                stack.extend(n.children.iter().copied());
            }
        }
    }

    fn layout_pending_barrier_relayouts_if_needed(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        scale_factor: f32,
        pass_kind: LayoutPassKind,
        viewport_cursor: &mut usize,
    ) {
        if pass_kind != LayoutPassKind::Final {
            return;
        }

        let pending = self.take_pending_barrier_relayouts();
        if pending.is_empty() {
            return;
        }

        let mut unique = HashSet::<NodeId>::with_capacity(pending.len());
        let mut targets: Vec<NodeId> = Vec::with_capacity(pending.len());
        for node in pending {
            if unique.insert(node) {
                targets.push(node);
            }
        }

        for root in targets {
            let Some(node) = self.nodes.get(root) else {
                continue;
            };
            if !node.invalidation.layout {
                continue;
            }

            // Barrier relayouts intentionally do not invalidate ancestors. Prefer the retained
            // bounds (stable barrier viewport), but fall back to resolving bounds from the parent
            // layout-engine rect when needed (e.g. newly mounted nodes with default bounds).
            let mut bounds = node.bounds;
            if (bounds.size == Size::default() || bounds.origin == Point::default())
                && let Some(parent) = node.parent
                && let Some(parent_bounds) = self.nodes.get(parent).map(|n| n.bounds)
                && let Some(local) = self.layout_engine_child_local_rect(parent, root)
            {
                let resolved = Rect::new(
                    Point::new(
                        Px(parent_bounds.origin.x.0 + local.origin.x.0),
                        Px(parent_bounds.origin.y.0 + local.origin.y.0),
                    ),
                    local.size,
                );
                if resolved.size != Size::default() {
                    bounds = resolved;
                }
            }

            if bounds.size == Size::default() {
                continue;
            }

            let _ =
                self.layout_in_with_pass_kind(app, services, root, bounds, scale_factor, pass_kind);
            if self.debug_enabled {
                self.debug_stats.barrier_relayouts_performed = self
                    .debug_stats
                    .barrier_relayouts_performed
                    .saturating_add(1);
            }
            self.flush_viewport_roots_after_root(
                app,
                services,
                scale_factor,
                pass_kind,
                viewport_cursor,
            );
        }
    }

    pub fn layout(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        available: Size,
        scale_factor: f32,
    ) -> Size {
        let bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            available,
        );

        if self.invalidated_layout_nodes == 0
            && self.invalidated_hit_test_nodes == 0
            && let Some(n) = self.nodes.get(root)
            && !n.invalidation.layout
            && !n.invalidation.hit_test
            && n.bounds == bounds
            && n.measured_size != Size::default()
        {
            return n.measured_size;
        }

        let mut viewport_cursor: usize = 0;
        self.begin_layout_engine_frame(app);
        self.request_build_window_roots_if_final(
            app,
            services,
            std::slice::from_ref(&root),
            bounds,
            scale_factor,
            LayoutPassKind::Final,
        );
        let size = self.layout_in_with_pass_kind(
            app,
            services,
            root,
            bounds,
            scale_factor,
            LayoutPassKind::Final,
        );
        self.flush_viewport_roots_after_root(
            app,
            services,
            scale_factor,
            LayoutPassKind::Final,
            &mut viewport_cursor,
        );

        self.layout_engine.end_frame();
        if let Some(window) = self.window {
            let frame_id = app.frame_id();
            crate::elements::with_window_state(app, window, |st| {
                st.clear_stale_interaction_targets_for_frame(frame_id);
            });
        }
        self.sync_element_bounds_cache_after_layout(app);
        size
    }

    pub fn layout_in(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        bounds: Rect,
        scale_factor: f32,
    ) -> Size {
        if self.invalidated_layout_nodes == 0
            && self.invalidated_hit_test_nodes == 0
            && let Some(n) = self.nodes.get(root)
            && !n.invalidation.layout
            && !n.invalidation.hit_test
            && n.bounds == bounds
            && n.measured_size != Size::default()
        {
            return n.measured_size;
        }

        let mut viewport_cursor: usize = 0;
        self.begin_layout_engine_frame(app);
        self.request_build_window_roots_if_final(
            app,
            services,
            std::slice::from_ref(&root),
            bounds,
            scale_factor,
            LayoutPassKind::Final,
        );
        let size = self.layout_in_with_pass_kind(
            app,
            services,
            root,
            bounds,
            scale_factor,
            LayoutPassKind::Final,
        );
        self.flush_viewport_roots_after_root(
            app,
            services,
            scale_factor,
            LayoutPassKind::Final,
            &mut viewport_cursor,
        );
        self.layout_engine.end_frame();
        if let Some(window) = self.window {
            let frame_id = app.frame_id();
            crate::elements::with_window_state(app, window, |st| {
                st.clear_stale_interaction_targets_for_frame(frame_id);
            });
        }
        self.sync_element_bounds_cache_after_layout(app);
        size
    }

    #[stacksafe::stacksafe]
    pub fn layout_in_with_pass_kind(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        bounds: Rect,
        scale_factor: f32,
        pass_kind: LayoutPassKind,
    ) -> Size {
        self.layout_node(app, services, root, bounds, scale_factor, pass_kind)
    }

    fn sync_element_bounds_cache_after_layout(&mut self, app: &mut H) {
        let Some(window) = self.window else {
            return;
        };

        let element_nodes =
            crate::elements::with_window_state(app, window, |st| st.element_nodes());

        let bounds: Vec<(GlobalElementId, Rect)> = element_nodes
            .into_iter()
            .filter_map(|(element, node)| self.node_bounds(node).map(|rect| (element, rect)))
            .collect();

        crate::elements::with_window_state(app, window, |st| {
            for (element, rect) in bounds {
                st.record_bounds(element, rect);
            }
        })
    }

    pub fn measure_in(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        node: NodeId,
        constraints: LayoutConstraints,
        scale_factor: f32,
    ) -> Size {
        self.measure_node(app, services, node, constraints, scale_factor)
    }

    fn begin_layout_engine_frame(&mut self, app: &mut H) {
        self.layout_engine.begin_frame(app.frame_id());
        self.viewport_roots.clear();
    }

    fn layout_contained_view_cache_roots_if_needed(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        scale_factor: f32,
        pass_kind: LayoutPassKind,
        viewport_cursor: &mut usize,
    ) {
        if !self.view_cache_active() {
            return;
        }

        let mut targets: Vec<(NodeId, Rect)> = Vec::with_capacity(16);
        for (id, node) in self.nodes.iter() {
            if !node.view_cache.enabled || !node.view_cache.contained_layout {
                continue;
            }
            if !node.invalidation.layout {
                continue;
            }

            // Contained relayouts run after the main layout pass. If a cache root was newly
            // mounted (or skipped by an engine-backed parent) its retained bounds can still be
            // the default `Rect::default()`, which would incorrectly relayout the subtree at the
            // origin and desynchronize semantics/hit-testing from the painted output.
            //
            // Prefer the parent's solved layout-engine rect when available so the contained pass
            // runs in the same coordinate space as the parent placement.
            let mut bounds = node.bounds;
            if (bounds.size == Size::default() || bounds.origin == Point::default())
                && let Some(parent) = node.parent
                && let Some(parent_bounds) = self.nodes.get(parent).map(|n| n.bounds)
                && let Some(local) = self.layout_engine_child_local_rect(parent, id)
            {
                let resolved = Rect::new(
                    Point::new(
                        Px(parent_bounds.origin.x.0 + local.origin.x.0),
                        Px(parent_bounds.origin.y.0 + local.origin.y.0),
                    ),
                    local.size,
                );
                if resolved.size != Size::default() {
                    bounds = resolved;
                }
            }

            targets.push((id, bounds));
        }

        for (root, bounds) in targets {
            if self.debug_enabled {
                self.debug_stats.view_cache_contained_relayouts = self
                    .debug_stats
                    .view_cache_contained_relayouts
                    .saturating_add(1);
                self.debug_view_cache_contained_relayout_roots.push(root);
            }
            let _ =
                self.layout_in_with_pass_kind(app, services, root, bounds, scale_factor, pass_kind);
            self.flush_viewport_roots_after_root(
                app,
                services,
                scale_factor,
                pass_kind,
                viewport_cursor,
            );
            if let Some(node) = self.nodes.get_mut(root) {
                node.view_cache_needs_rerender = true;
            }
        }
    }

    fn maybe_dump_taffy_subtree(
        &self,
        app: &mut H,
        window: AppWindowId,
        engine: &TaffyLayoutEngine,
        root: NodeId,
        root_bounds: Rect,
        scale_factor: f32,
    ) {
        use std::sync::atomic::{AtomicU32, Ordering};

        if std::env::var_os("FRET_TAFFY_DUMP").is_none() {
            return;
        }

        static DUMP_COUNT: AtomicU32 = AtomicU32::new(0);
        let dump_max: Option<u32> =
            if std::env::var("FRET_TAFFY_DUMP_ONCE").ok().as_deref() == Some("1") {
                Some(1)
            } else {
                std::env::var("FRET_TAFFY_DUMP_MAX")
                    .ok()
                    .and_then(|s| s.parse().ok())
            };
        if let Some(max) = dump_max {
            let prev = DUMP_COUNT.fetch_add(1, Ordering::SeqCst);
            if prev >= max {
                return;
            }
        }

        if let Ok(filter) = std::env::var("FRET_TAFFY_DUMP_ROOT")
            && !format!("{root:?}").contains(&filter)
        {
            return;
        }

        // When debugging complex demos or golden-gated layouts, it is often easier to filter by a
        // stable element label (e.g. a `SemanticsProps.label`) than by ephemeral `NodeId`s.
        let dump_root = if let Ok(filter) = std::env::var("FRET_TAFFY_DUMP_ROOT_LABEL") {
            let root_label = crate::declarative::frame::element_record_for_node(app, window, root)
                .map(|r| format!("{:?}", r.instance))
                .unwrap_or_default();
            if root_label.contains(&filter) {
                root
            } else {
                let mut stack: Vec<NodeId> = vec![root];
                let mut visited: std::collections::HashSet<NodeId> =
                    std::collections::HashSet::new();
                let mut found: Option<NodeId> = None;
                while let Some(node) = stack.pop() {
                    if !visited.insert(node) {
                        continue;
                    }

                    let label =
                        crate::declarative::frame::element_record_for_node(app, window, node)
                            .map(|r| format!("{:?}", r.instance))
                            .unwrap_or_default();
                    if label.contains(&filter) {
                        found = Some(node);
                        break;
                    }

                    if let Some(node) = self.nodes.get(node) {
                        stack.extend(node.children.iter().copied());
                    }
                }

                let Some(found) = found else {
                    return;
                };

                found
            }
        } else {
            root
        };

        let out_dir = std::env::var("FRET_TAFFY_DUMP_DIR")
            .ok()
            .unwrap_or_else(|| ".fret/taffy-dumps".to_string());

        let frame = app.frame_id().0;
        let root_slug: String = format!("{dump_root:?}")
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
            .collect();
        let filename = format!("taffy_{frame}_{root_slug}.json");

        let dump = engine.debug_dump_subtree_json(dump_root, |node| {
            crate::declarative::frame::element_record_for_node(app, window, node)
                .map(|r| format!("{:?}", r.instance))
        });

        let wrapped = serde_json::json!({
            "meta": {
                "window": format!("{window:?}"),
                "root_bounds": {
                    "x": root_bounds.origin.x.0,
                    "y": root_bounds.origin.y.0,
                    "w": root_bounds.size.width.0,
                    "h": root_bounds.size.height.0,
                },
                "scale_factor": scale_factor,
            },
            "taffy": dump,
        });

        let result = std::fs::create_dir_all(&out_dir)
            .and_then(|_| {
                serde_json::to_vec_pretty(&wrapped)
                    .map_err(|e| std::io::Error::other(format!("serialize: {e}")))
            })
            .and_then(|bytes| {
                std::fs::write(std::path::Path::new(&out_dir).join(&filename), bytes)
            });

        match result {
            Ok(()) => tracing::info!(
                out_dir = %out_dir,
                filename = %filename,
                "wrote taffy debug dump"
            ),
            Err(err) => tracing::warn!(
                error = %err,
                out_dir = %out_dir,
                filename = %filename,
                "failed to write taffy debug dump"
            ),
        }
    }

    /// Write a Taffy layout dump for a subtree rooted at `root`.
    ///
    /// The dump includes both local and absolute rects plus a debug label per node. When
    /// `root_label_filter` is provided, the dump will search for the first node whose debug label
    /// contains the filter string and use that node as the dump root (falling back to `root` when
    /// the filter does not match anything).
    ///
    /// This is a debug-only escape hatch intended for diagnosing layout regressions and scroll /
    /// clipping issues. The output is JSON and is written to `out_dir`.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::too_many_arguments)]
    pub fn debug_write_taffy_subtree_json(
        &self,
        app: &mut H,
        window: AppWindowId,
        root: NodeId,
        root_bounds: Rect,
        scale_factor: f32,
        root_label_filter: Option<&str>,
        out_dir: impl AsRef<std::path::Path>,
        filename_tag: &str,
    ) -> std::io::Result<std::path::PathBuf> {
        fn sanitize_for_filename(s: &str) -> String {
            s.chars()
                .map(|ch| match ch {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
                    _ => '_',
                })
                .collect()
        }

        let dump_root = if let Some(filter) = root_label_filter {
            let root_label = crate::declarative::frame::element_record_for_node(app, window, root)
                .map(|r| format!("{:?}", r.instance))
                .unwrap_or_default();
            if root_label.contains(filter) {
                root
            } else {
                let mut stack: Vec<NodeId> = vec![root];
                let mut visited: std::collections::HashSet<NodeId> =
                    std::collections::HashSet::new();
                let mut found: Option<NodeId> = None;
                while let Some(node) = stack.pop() {
                    if !visited.insert(node) {
                        continue;
                    }

                    let label =
                        crate::declarative::frame::element_record_for_node(app, window, node)
                            .map(|r| format!("{:?}", r.instance))
                            .unwrap_or_default();
                    if label.contains(filter) {
                        found = Some(node);
                        break;
                    }

                    if let Some(node) = self.nodes.get(node) {
                        stack.extend(node.children.iter().copied());
                    }
                }

                found.unwrap_or(root)
            }
        } else {
            root
        };

        let tag = sanitize_for_filename(filename_tag);
        let frame = app.frame_id().0;
        let root_slug = sanitize_for_filename(&format!("{dump_root:?}"));
        let filename = if tag.is_empty() {
            format!("taffy_{frame}_{root_slug}.json")
        } else {
            format!("taffy_{frame}_{tag}_{root_slug}.json")
        };

        let dump = self
            .layout_engine
            .debug_dump_subtree_json(dump_root, |node| {
                crate::declarative::frame::element_record_for_node(app, window, node)
                    .map(|r| format!("{:?}", r.instance))
            });

        let wrapped = serde_json::json!({
            "meta": {
                "window": format!("{window:?}"),
                "root_bounds": {
                    "x": root_bounds.origin.x.0,
                    "y": root_bounds.origin.y.0,
                    "w": root_bounds.size.width.0,
                    "h": root_bounds.size.height.0,
                },
                "scale_factor": scale_factor,
            },
            "taffy": dump,
        });

        let out_dir = out_dir.as_ref();
        std::fs::create_dir_all(out_dir)?;
        let path = out_dir.join(filename);
        let bytes = serde_json::to_vec_pretty(&wrapped)
            .map_err(|e| std::io::Error::other(format!("serialize: {e}")))?;
        std::fs::write(&path, bytes)?;
        Ok(path)
    }

    fn request_build_window_roots_if_final(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        roots: &[NodeId],
        bounds: Rect,
        scale_factor: f32,
        pass_kind: LayoutPassKind,
    ) {
        if pass_kind != LayoutPassKind::Final {
            return;
        }

        let Some(window) = self.window else {
            return;
        };

        let profile_layout =
            std::env::var_os("FRET_LAYOUT_PROFILE").is_some_and(|v| !v.is_empty() && v != "0");
        let total_started = profile_layout.then(Instant::now);

        let sf = scale_factor;
        let available = LayoutSize::new(
            AvailableSpace::Definite(bounds.size.width),
            AvailableSpace::Definite(bounds.size.height),
        );

        let mut engine = self.take_layout_engine();
        engine.set_measure_profiling_enabled(self.debug_enabled);

        let phase1_started = profile_layout.then(Instant::now);
        // Phase 1: request/build for stable identity, even if we later skip compute/apply.
        for &root in roots {
            if self
                .nodes
                .get(root)
                .is_none_or(|node| node.element.is_none())
            {
                continue;
            }

            build_viewport_flow_subtree(&mut engine, app, &*self, window, sf, root, bounds.size);
        }
        let phase1_elapsed = phase1_started.map(|s| s.elapsed());

        let phase2_started = profile_layout.then(Instant::now);
        // Phase 2: compute/apply only when layout is needed.
        for &root in roots {
            let (has_element, needs_layout, is_translation_only) = match self.nodes.get(root) {
                Some(node) => {
                    let has_element = node.element.is_some();
                    let needs_layout = node.invalidation.layout || node.bounds != bounds;
                    let is_translation_only = !node.invalidation.layout
                        && node.bounds.size == bounds.size
                        && node.bounds.origin != bounds.origin
                        && node.measured_size != Size::default();
                    (has_element, needs_layout, is_translation_only)
                }
                None => continue,
            };

            if !has_element || !needs_layout || is_translation_only {
                continue;
            }

            let solves_before = engine.solve_count();
            let solve_time_before = engine.last_solve_time();
            let _ =
                engine.compute_root_for_node_with_measure_if_needed(root, available, sf, |n, c| {
                    self.measure_in(app, services, n, c, sf)
                });
            if self.debug_enabled && engine.solve_count() > solves_before {
                let elapsed = engine.last_solve_time().saturating_sub(solve_time_before);
                let top_measures = engine
                    .last_solve_measure_hotspots()
                    .iter()
                    .map(|h| {
                        let mut element: Option<GlobalElementId> = None;
                        let mut element_kind: Option<&'static str> = None;
                        if let Some(record) =
                            crate::declarative::frame::element_record_for_node(app, window, h.node)
                        {
                            element = Some(record.element);
                            element_kind = Some(record.instance.kind_name());
                        }
                        let top_children = self
                            .debug_take_top_measure_children(h.node, 3)
                            .into_iter()
                            .map(|(child, r)| {
                                let mut child_element: Option<GlobalElementId> = None;
                                let mut child_kind: Option<&'static str> = None;
                                if let Some(record) =
                                    crate::declarative::frame::element_record_for_node(
                                        app, window, child,
                                    )
                                {
                                    child_element = Some(record.element);
                                    child_kind = Some(record.instance.kind_name());
                                }
                                super::UiDebugLayoutEngineMeasureChildHotspot {
                                    child,
                                    measure_time: r.total_time,
                                    calls: r.calls,
                                    element: child_element,
                                    element_kind: child_kind,
                                }
                            })
                            .collect();
                        super::UiDebugLayoutEngineMeasureHotspot {
                            node: h.node,
                            measure_time: h.total_time,
                            calls: h.calls,
                            cache_hits: h.cache_hits,
                            element,
                            element_kind,
                            top_children,
                        }
                    })
                    .collect();
                self.debug_record_layout_engine_solve(
                    engine.last_solve_root().unwrap_or(root),
                    elapsed,
                    engine.last_solve_measure_calls(),
                    engine.last_solve_measure_cache_hits(),
                    engine.last_solve_measure_time(),
                    top_measures,
                );
                self.debug_measure_children.clear();
            }

            self.maybe_dump_taffy_subtree(app, window, &engine, root, bounds, sf);
        }
        let phase2_elapsed = phase2_started.map(|s| s.elapsed());

        self.put_layout_engine(engine);

        if let Some(started) = total_started {
            let total = started.elapsed();
            tracing::info!(
                window = ?window,
                roots = roots.len(),
                total_ms = total.as_millis(),
                phase1_ms = phase1_elapsed.map(|d| d.as_millis()),
                phase2_ms = phase2_elapsed.map(|d| d.as_millis()),
                "layout root request/build profile"
            );
        }
    }

    fn flush_viewport_roots_after_root(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        scale_factor: f32,
        pass_kind: LayoutPassKind,
        viewport_cursor: &mut usize,
    ) {
        let sf = scale_factor;
        let window = self.window;

        while *viewport_cursor < self.viewport_roots.len() {
            let batch_start = *viewport_cursor;
            let batch_end = self.viewport_roots.len();

            struct ViewportWorkItem {
                root: NodeId,
                bounds: Rect,
                needs_layout: bool,
                is_translation_only: bool,
            }

            let mut batch: Vec<ViewportWorkItem> = Vec::with_capacity(batch_end - batch_start);
            for &(root, bounds) in &self.viewport_roots[batch_start..batch_end] {
                let Some((prev_bounds, invalidated, measured)) = self
                    .nodes
                    .get(root)
                    .map(|n| (n.bounds, n.invalidation.layout, n.measured_size))
                else {
                    continue;
                };

                let needs_layout = invalidated || prev_bounds != bounds;
                let is_translation_only = !invalidated
                    && prev_bounds.size == bounds.size
                    && prev_bounds.origin != bounds.origin
                    && measured != Size::default();

                batch.push(ViewportWorkItem {
                    root,
                    bounds,
                    needs_layout,
                    is_translation_only,
                });
            }

            if pass_kind == LayoutPassKind::Final
                && let Some(window) = window
            {
                let mut engine = self.take_layout_engine();
                engine.set_measure_profiling_enabled(self.debug_enabled);

                // Phase 1: request/build newly registered viewport roots for stable identity,
                // regardless of whether they will be computed this frame.
                for item in &batch {
                    if self
                        .nodes
                        .get(item.root)
                        .is_none_or(|node| node.element.is_none())
                    {
                        continue;
                    }

                    build_viewport_flow_subtree(
                        &mut engine,
                        app,
                        &*self,
                        window,
                        sf,
                        item.root,
                        item.bounds.size,
                    );
                }

                // Phase 2: compute/apply only for roots that need layout and are not translation-only.
                for item in &batch {
                    if !item.needs_layout || item.is_translation_only {
                        continue;
                    }

                    let available = LayoutSize::new(
                        AvailableSpace::Definite(item.bounds.size.width),
                        AvailableSpace::Definite(item.bounds.size.height),
                    );

                    let solves_before = engine.solve_count();
                    let solve_time_before = engine.last_solve_time();
                    let _ = engine.compute_root_for_node_with_measure_if_needed(
                        item.root,
                        available,
                        sf,
                        |n, c| self.measure_in(app, services, n, c, sf),
                    );
                    if self.debug_enabled && engine.solve_count() > solves_before {
                        let elapsed = engine.last_solve_time().saturating_sub(solve_time_before);
                        let top_measures = engine
                            .last_solve_measure_hotspots()
                            .iter()
                            .map(|h| {
                                let mut element: Option<GlobalElementId> = None;
                                let mut element_kind: Option<&'static str> = None;
                                if let Some(record) =
                                    crate::declarative::frame::element_record_for_node(
                                        app, window, h.node,
                                    )
                                {
                                    element = Some(record.element);
                                    element_kind = Some(record.instance.kind_name());
                                }
                                let top_children = self
                                    .debug_take_top_measure_children(h.node, 3)
                                    .into_iter()
                                    .map(|(child, r)| {
                                        let mut child_element: Option<GlobalElementId> = None;
                                        let mut child_kind: Option<&'static str> = None;
                                        if let Some(record) =
                                            crate::declarative::frame::element_record_for_node(
                                                app, window, child,
                                            )
                                        {
                                            child_element = Some(record.element);
                                            child_kind = Some(record.instance.kind_name());
                                        }
                                        super::UiDebugLayoutEngineMeasureChildHotspot {
                                            child,
                                            measure_time: r.total_time,
                                            calls: r.calls,
                                            element: child_element,
                                            element_kind: child_kind,
                                        }
                                    })
                                    .collect();
                                super::UiDebugLayoutEngineMeasureHotspot {
                                    node: h.node,
                                    measure_time: h.total_time,
                                    calls: h.calls,
                                    cache_hits: h.cache_hits,
                                    element,
                                    element_kind,
                                    top_children,
                                }
                            })
                            .collect();
                        self.debug_record_layout_engine_solve(
                            engine.last_solve_root().unwrap_or(item.root),
                            elapsed,
                            engine.last_solve_measure_calls(),
                            engine.last_solve_measure_cache_hits(),
                            engine.last_solve_measure_time(),
                            top_measures,
                        );
                        self.debug_measure_children.clear();
                    }

                    self.maybe_dump_taffy_subtree(app, window, &engine, item.root, item.bounds, sf);
                }

                self.put_layout_engine(engine);
            }

            // Apply the viewport root bounds by running the regular layout pass. Even when a root
            // is translation-only (so we skip compute), the translation-only fast path needs to
            // update the retained bounds for the subtree.
            for item in &batch {
                if !item.needs_layout {
                    continue;
                }

                let _ = self.layout_in_with_pass_kind(
                    app,
                    services,
                    item.root,
                    item.bounds,
                    scale_factor,
                    LayoutPassKind::Final,
                );
            }

            *viewport_cursor = batch_end;
        }
    }

    pub(crate) fn record_layout_engine_widget_fallback_solve(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        widget_kind: &'static str,
        missing_child: Option<NodeId>,
    ) {
        if self.debug_enabled {
            self.debug_stats.layout_engine_widget_fallback_solves = self
                .debug_stats
                .layout_engine_widget_fallback_solves
                .saturating_add(1);
        }

        let forbid_fallback_solves =
            std::env::var_os("FRET_LAYOUT_FORBID_WIDGET_FALLBACK_SOLVES").is_some();
        let trace_fallback_solves =
            std::env::var_os("FRET_LAYOUT_TRACE_WIDGET_FALLBACK_SOLVES").is_some();

        if trace_fallback_solves {
            let label = crate::declarative::frame::element_record_for_node(app, window, node)
                .map(|r| r.instance);
            let missing_label = missing_child.and_then(|child| {
                crate::declarative::frame::element_record_for_node(app, window, child)
                    .map(|r| r.instance)
            });

            tracing::warn!(
                window = ?self.window,
                node = ?node,
                widget_kind,
                label = ?label,
                missing_child = ?missing_child,
                missing_label = ?missing_label,
                path = ?self.debug_node_path(node),
                "layout engine child rects missing; falling back to widget-local solve"
            );
        }

        if forbid_fallback_solves {
            let label = crate::declarative::frame::element_record_for_node(app, window, node)
                .map(|r| format!("{:?}", r.instance))
                .unwrap_or_default();
            let missing_label = missing_child
                .and_then(|child| {
                    crate::declarative::frame::element_record_for_node(app, window, child)
                })
                .map(|r| format!("{:?}", r.instance));
            let path = self.debug_node_path(node);
            panic!(
                "layout engine fallback solve ({widget_kind}) for {node:?} {label} missing_child={missing_child:?} missing_label={missing_label:?} path={path:?}"
            );
        }
    }

    /// Internal barrier bridge: prefer calling via `LayoutCx::solve_barrier_child_root(...)`.
    ///
    /// Only explicit layout barriers (scroll, virtualization, resizable splits, etc.) should solve
    /// child roots "out of band" like this, and only during `LayoutPassKind::Final`.
    pub(crate) fn solve_barrier_flow_root(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        root_bounds: Rect,
        scale_factor: f32,
    ) {
        let Some(window) = self.window else {
            return;
        };

        let mut engine = self.take_layout_engine();
        engine.set_measure_profiling_enabled(self.debug_enabled);
        crate::layout_engine::build_viewport_flow_subtree(
            &mut engine,
            app,
            &*self,
            window,
            scale_factor,
            root,
            root_bounds.size,
        );
        let available = LayoutSize::new(
            AvailableSpace::Definite(root_bounds.size.width),
            AvailableSpace::Definite(root_bounds.size.height),
        );

        let sf = scale_factor;
        let solves_before = engine.solve_count();
        let solve_time_before = engine.last_solve_time();
        let _ = engine.compute_root_for_node_with_measure_if_needed(
            root,
            available,
            sf,
            |node, constraints| self.measure_in(app, services, node, constraints, sf),
        );
        if self.debug_enabled && engine.solve_count() > solves_before {
            let elapsed = engine.last_solve_time().saturating_sub(solve_time_before);
            let top_measures = engine
                .last_solve_measure_hotspots()
                .iter()
                .map(|h| {
                    let mut element: Option<GlobalElementId> = None;
                    let mut element_kind: Option<&'static str> = None;
                    if let Some(record) =
                        crate::declarative::frame::element_record_for_node(app, window, h.node)
                    {
                        element = Some(record.element);
                        element_kind = Some(record.instance.kind_name());
                    }
                    let top_children =
                        self.debug_take_top_measure_children(h.node, 3)
                            .into_iter()
                            .map(|(child, r)| {
                                let mut child_element: Option<GlobalElementId> = None;
                                let mut child_kind: Option<&'static str> = None;
                                if let Some(record) =
                                    crate::declarative::frame::element_record_for_node(
                                        app, window, child,
                                    )
                                {
                                    child_element = Some(record.element);
                                    child_kind = Some(record.instance.kind_name());
                                }
                                super::UiDebugLayoutEngineMeasureChildHotspot {
                                    child,
                                    measure_time: r.total_time,
                                    calls: r.calls,
                                    element: child_element,
                                    element_kind: child_kind,
                                }
                            })
                            .collect();
                    super::UiDebugLayoutEngineMeasureHotspot {
                        node: h.node,
                        measure_time: h.total_time,
                        calls: h.calls,
                        cache_hits: h.cache_hits,
                        element,
                        element_kind,
                        top_children,
                    }
                })
                .collect();
            self.debug_record_layout_engine_solve(
                engine.last_solve_root().unwrap_or(root),
                elapsed,
                engine.last_solve_measure_calls(),
                engine.last_solve_measure_cache_hits(),
                engine.last_solve_measure_time(),
                top_measures,
            );
            self.debug_measure_children.clear();
        }

        self.maybe_dump_taffy_subtree(app, window, &engine, root, root_bounds, scale_factor);
        self.put_layout_engine(engine);
    }

    /// Internal barrier bridge: prefer calling via `LayoutCx::solve_barrier_child_root_if_needed(...)`.
    pub(crate) fn solve_barrier_flow_root_if_needed(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        root_bounds: Rect,
        scale_factor: f32,
    ) {
        let Some(node) = self.nodes.get(root) else {
            return;
        };

        let needs_layout = node.invalidation.layout || node.bounds != root_bounds;
        if !needs_layout {
            return;
        }

        let is_translation_only = !node.invalidation.layout
            && node.bounds.size == root_bounds.size
            && node.bounds.origin != root_bounds.origin
            && node.measured_size != Size::default();
        if is_translation_only {
            return;
        }

        self.solve_barrier_flow_root(app, services, root, root_bounds, scale_factor);
    }

    /// Internal barrier bridge: batch variant of `solve_barrier_flow_root_if_needed`.
    ///
    /// This exists to reduce `take_layout_engine()` / `put_layout_engine()` churn when a barrier
    /// needs to solve many child roots (e.g. virtualized lists).
    pub(crate) fn solve_barrier_flow_roots_if_needed(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        roots: &[(NodeId, Rect)],
        scale_factor: f32,
    ) {
        let Some(window) = self.window else {
            return;
        };
        if roots.is_empty() {
            return;
        }

        let mut batch: Vec<(NodeId, Rect)> = Vec::with_capacity(roots.len());
        for &(root, root_bounds) in roots {
            let Some(node) = self.nodes.get(root) else {
                continue;
            };

            let needs_layout = node.invalidation.layout || node.bounds != root_bounds;
            if !needs_layout {
                continue;
            }

            let is_translation_only = !node.invalidation.layout
                && node.bounds.size == root_bounds.size
                && node.bounds.origin != root_bounds.origin
                && node.measured_size != Size::default();
            if is_translation_only {
                continue;
            }

            batch.push((root, root_bounds));
        }

        if batch.is_empty() {
            return;
        }

        let mut engine = self.take_layout_engine();
        engine.set_measure_profiling_enabled(self.debug_enabled);
        for &(root, root_bounds) in &batch {
            crate::layout_engine::build_viewport_flow_subtree(
                &mut engine,
                app,
                &*self,
                window,
                scale_factor,
                root,
                root_bounds.size,
            );
        }

        let sf = scale_factor;
        for &(root, root_bounds) in &batch {
            let available = LayoutSize::new(
                AvailableSpace::Definite(root_bounds.size.width),
                AvailableSpace::Definite(root_bounds.size.height),
            );

            let solves_before = engine.solve_count();
            let solve_time_before = engine.last_solve_time();
            let _ =
                engine.compute_root_for_node_with_measure_if_needed(root, available, sf, |n, c| {
                    self.measure_in(app, services, n, c, sf)
                });
            if self.debug_enabled && engine.solve_count() > solves_before {
                let elapsed = engine.last_solve_time().saturating_sub(solve_time_before);
                let top_measures = engine
                    .last_solve_measure_hotspots()
                    .iter()
                    .map(|h| {
                        let mut element: Option<GlobalElementId> = None;
                        let mut element_kind: Option<&'static str> = None;
                        if let Some(record) =
                            crate::declarative::frame::element_record_for_node(app, window, h.node)
                        {
                            element = Some(record.element);
                            element_kind = Some(record.instance.kind_name());
                        }
                        let top_children = self
                            .debug_take_top_measure_children(h.node, 3)
                            .into_iter()
                            .map(|(child, r)| {
                                let mut child_element: Option<GlobalElementId> = None;
                                let mut child_kind: Option<&'static str> = None;
                                if let Some(record) =
                                    crate::declarative::frame::element_record_for_node(
                                        app, window, child,
                                    )
                                {
                                    child_element = Some(record.element);
                                    child_kind = Some(record.instance.kind_name());
                                }
                                super::UiDebugLayoutEngineMeasureChildHotspot {
                                    child,
                                    measure_time: r.total_time,
                                    calls: r.calls,
                                    element: child_element,
                                    element_kind: child_kind,
                                }
                            })
                            .collect();
                        super::UiDebugLayoutEngineMeasureHotspot {
                            node: h.node,
                            measure_time: h.total_time,
                            calls: h.calls,
                            cache_hits: h.cache_hits,
                            element,
                            element_kind,
                            top_children,
                        }
                    })
                    .collect();
                self.debug_record_layout_engine_solve(
                    engine.last_solve_root().unwrap_or(root),
                    elapsed,
                    engine.last_solve_measure_calls(),
                    engine.last_solve_measure_cache_hits(),
                    engine.last_solve_measure_time(),
                    top_measures,
                );
                self.debug_measure_children.clear();
            }

            self.maybe_dump_taffy_subtree(app, window, &engine, root, root_bounds, sf);
        }

        self.put_layout_engine(engine);
    }

    fn layout_node(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        node: NodeId,
        bounds: Rect,
        scale_factor: f32,
        pass_kind: LayoutPassKind,
    ) -> Size {
        let is_probe = pass_kind == LayoutPassKind::Probe;
        if self.debug_enabled {
            self.debug_stats.layout_nodes_visited =
                self.debug_stats.layout_nodes_visited.saturating_add(1);
        }

        let (prev_bounds, measured, invalidated) = match self.nodes.get(node) {
            Some(n) => (n.bounds, n.measured_size, n.invalidation.layout),
            None => return Size::default(),
        };
        let invalidated_for_pass = invalidated || is_probe;

        let view_cache = self
            .nodes
            .get(node)
            .map(|n| n.view_cache)
            .unwrap_or_default();
        let span = if view_cache.enabled && tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace_span!(
                "ui.cache_root.layout",
                node = ?node,
                pass = ?pass_kind,
                view_cache_active = self.view_cache_active(),
                contained_layout = view_cache.contained_layout,
                invalidated = invalidated_for_pass,
                frame_id = app.frame_id().0,
            )
        } else {
            tracing::Span::none()
        };
        let _span_guard = span.enter();

        if let Some(n) = self.nodes.get_mut(node) {
            n.bounds = bounds;
        }

        if !invalidated_for_pass
            && prev_bounds.size == bounds.size
            && prev_bounds.origin != bounds.origin
            && measured != Size::default()
        {
            let delta = Point::new(
                bounds.origin.x - prev_bounds.origin.x,
                bounds.origin.y - prev_bounds.origin.y,
            );
            if delta.x.0 != 0.0 || delta.y.0 != 0.0 {
                self.layout_engine.mark_seen_if_present(node);

                let window = self.window;
                let mut stack: Vec<NodeId> = Vec::new();
                let mut i = 0usize;
                loop {
                    let child = self
                        .nodes
                        .get(node)
                        .and_then(|n| n.children.get(i))
                        .copied();
                    let Some(child) = child else {
                        break;
                    };
                    stack.push(child);
                    i += 1;
                }

                while let Some(id) = stack.pop() {
                    self.layout_engine.mark_seen_if_present(id);

                    let Some(n) = self.nodes.get_mut(id) else {
                        continue;
                    };
                    n.bounds.origin =
                        Point::new(n.bounds.origin.x + delta.x, n.bounds.origin.y + delta.y);
                    if !is_probe && let (Some(window), Some(element)) = (window, n.element) {
                        crate::elements::record_bounds_for_element(app, window, element, n.bounds);
                    }
                    for &child in &n.children {
                        stack.push(child);
                    }
                }
            }
            if !is_probe
                && let (Some(window), Some(element)) =
                    (self.window, self.nodes.get(node).and_then(|n| n.element))
            {
                crate::elements::record_bounds_for_element(app, window, element, bounds);
            }
            return measured;
        }

        let needs_layout = invalidated_for_pass || prev_bounds != bounds;
        if !needs_layout {
            return measured;
        }
        if self.debug_enabled {
            self.debug_stats.layout_nodes_performed =
                self.debug_stats.layout_nodes_performed.saturating_add(1);
        }
        let sf = scale_factor;

        let mut observations = SmallCopyList::<(ModelId, Invalidation), 8>::default();
        let mut observe_model = |model: ModelId, inv: Invalidation| {
            observations.push((model, inv));
        };

        let mut global_observations = SmallCopyList::<(TypeId, Invalidation), 8>::default();
        let mut observe_global = |id: TypeId, inv: Invalidation| {
            global_observations.push((id, inv));
        };
        // Theme changes can affect layout metrics across most of the tree; treat it as a default
        // dependency to ensure layout re-runs when the global theme is updated.
        observe_global(TypeId::of::<Theme>(), Invalidation::Layout);
        // Text shaping/metrics depend on the effective font stack. Track a single stable key so
        // changing font configuration or loading new fonts forces a relayout without directly
        // depending on backend configuration globals.
        observe_global(
            TypeId::of::<fret_runtime::TextFontStackKey>(),
            Invalidation::Layout,
        );

        if let Some(profile) = self.layout_node_profile.as_mut() {
            profile.enter(node, pass_kind, bounds);
        }
        let widget_started = self.debug_enabled.then(Instant::now);
        let mut widget_type: &'static str = "<unknown>";
        if self.debug_enabled {
            self.debug_layout_stack.push(super::DebugLayoutStackFrame {
                child_inclusive_time: Duration::default(),
            });
        }
        let size = self.with_widget_mut(node, |widget, tree| {
            if tree.debug_enabled {
                widget_type = widget.debug_type_name();
            }
            let mut children_buf = SmallNodeList::<32>::default();
            if let Some(children) = tree.nodes.get(node).map(|n| n.children.as_slice()) {
                children_buf.set(children);
            }
            let mut cx = LayoutCx {
                app,
                node,
                window: tree.window,
                focus: tree.focus,
                children: children_buf.as_slice(),
                bounds,
                available: bounds.size,
                pass_kind,
                scale_factor: sf,
                services: &mut *services,
                observe_model: &mut observe_model,
                observe_global: &mut observe_global,
                tree,
            };
            widget.layout(&mut cx)
        });
        if let Some(profile) = self.layout_node_profile.as_mut() {
            profile.exit(node);
        }
        if let Some(widget_started) = widget_started {
            const MAX_LAYOUT_HOTSPOTS: usize = 16;
            let inclusive_time = widget_started.elapsed();
            let child_inclusive_time = self
                .debug_layout_stack
                .pop()
                .map(|f| f.child_inclusive_time)
                .unwrap_or_default();
            let exclusive_time = inclusive_time.saturating_sub(child_inclusive_time);
            if let Some(parent) = self.debug_layout_stack.last_mut() {
                parent.child_inclusive_time += inclusive_time;
            }
            let element = self.nodes.get(node).and_then(|n| n.element);
            let element_kind = self.window.and_then(|window| {
                crate::declarative::frame::element_record_for_node(app, window, node)
                    .map(|record| record.instance.kind_name())
            });
            let element_path = if self.debug_enabled {
                #[cfg(feature = "diagnostics")]
                {
                    self.window.and_then(|window| {
                        element.and_then(|element| {
                            crate::elements::with_window_state(app, window, |st| {
                                st.debug_path_for_element(element)
                            })
                        })
                    })
                }
                #[cfg(not(feature = "diagnostics"))]
                {
                    None
                }
            } else {
                None
            };
            let record = super::UiDebugLayoutHotspot {
                node,
                element,
                element_kind,
                element_path,
                widget_type,
                inclusive_time,
                exclusive_time,
            };
            let idx = self
                .debug_layout_hotspots
                .iter()
                .position(|h| h.exclusive_time < record.exclusive_time)
                .unwrap_or(self.debug_layout_hotspots.len());
            self.debug_layout_hotspots.insert(idx, record.clone());
            if self.debug_layout_hotspots.len() > MAX_LAYOUT_HOTSPOTS {
                self.debug_layout_hotspots.truncate(MAX_LAYOUT_HOTSPOTS);
            }

            let idx = self
                .debug_layout_inclusive_hotspots
                .iter()
                .position(|h| h.inclusive_time < record.inclusive_time)
                .unwrap_or(self.debug_layout_inclusive_hotspots.len());
            self.debug_layout_inclusive_hotspots.insert(idx, record);
            if self.debug_layout_inclusive_hotspots.len() > MAX_LAYOUT_HOTSPOTS {
                self.debug_layout_inclusive_hotspots
                    .truncate(MAX_LAYOUT_HOTSPOTS);
            }
        }

        if !is_probe {
            self.observed_in_layout
                .record(node, observations.as_slice());
            self.observed_globals_in_layout
                .record(node, global_observations.as_slice());
            if let Some((prev, next)) = self.nodes.get_mut(node).map(|n| {
                n.measured_size = size;
                let prev = n.invalidation;
                if n.invalidation.layout {
                    debug_assert!(self.layout_invalidations_count > 0);
                    self.layout_invalidations_count =
                        self.layout_invalidations_count.saturating_sub(1);
                }
                n.invalidation.layout = false;
                (prev, n.invalidation)
            }) {
                self.update_invalidation_counters(prev, next);
            }
        }

        size
    }

    fn measure_node(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        node: NodeId,
        constraints: LayoutConstraints,
        scale_factor: f32,
    ) -> Size {
        let avail_w = available_space_key(constraints.available.width);
        let avail_h = available_space_key(constraints.available.height);
        let cache_key = NodeMeasureCacheKey {
            known_w_bits: constraints.known.width.map(|px| px.0.to_bits()),
            known_h_bits: constraints.known.height.map(|px| px.0.to_bits()),
            avail_w,
            avail_h,
            scale_bits: scale_factor.to_bits(),
        };

        let key = MeasureStackKey {
            node,
            known_w_bits: cache_key.known_w_bits,
            known_h_bits: cache_key.known_h_bits,
            avail_w,
            avail_h,
            scale_bits: cache_key.scale_bits,
        };

        if let Some(size) = self.measure_cache_this_frame.get(&key) {
            return *size;
        }

        if let Some(n) = self.nodes.get(node)
            && !n.invalidation.layout
            && let Some(cache) = n.measure_cache
            && cache.key == cache_key
        {
            return cache.size;
        }

        if self.measure_stack.contains(&key) {
            if cfg!(debug_assertions) {
                panic!("measure_in re-entered for {node:?} under {constraints:?}");
            }
            if let Some(suppressed) = self.measure_reentrancy_diagnostics.record(app.frame_id()) {
                tracing::warn!(
                    window = ?self.window,
                    node = ?node,
                    constraints = ?constraints,
                    suppressed,
                    "measure_in re-entered; returning Size::default()"
                );
            }
            return Size::default();
        }
        self.measure_stack.push(key);

        let sf = scale_factor;

        let mut observations = SmallCopyList::<(ModelId, Invalidation), 8>::default();
        let mut observe_model = |model: ModelId, inv: Invalidation| {
            observations.push((model, inv));
        };

        let mut global_observations = SmallCopyList::<(TypeId, Invalidation), 8>::default();
        let mut observe_global = |id: TypeId, inv: Invalidation| {
            global_observations.push((id, inv));
        };
        observe_global(TypeId::of::<Theme>(), Invalidation::Layout);
        observe_global(
            TypeId::of::<fret_runtime::TextFontStackKey>(),
            Invalidation::Layout,
        );

        if let Some(profile) = self.measure_node_profile.as_mut() {
            profile.enter(node, constraints);
        }

        let measure_started = self.debug_enabled.then(Instant::now);
        let mut widget_type: &'static str = "<unknown>";
        if self.debug_enabled {
            self.debug_widget_measure_stack
                .push(super::DebugWidgetMeasureStackFrame {
                    child_inclusive_time: Duration::default(),
                });
        }
        let size = self.with_widget_mut(node, |widget, tree| {
            if tree.debug_enabled {
                widget_type = widget.debug_type_name();
            }
            let mut children_buf = SmallNodeList::<32>::default();
            if let Some(children) = tree.nodes.get(node).map(|n| n.children.as_slice()) {
                children_buf.set(children);
            }
            let mut cx = crate::widget::MeasureCx {
                app,
                node,
                window: tree.window,
                focus: tree.focus,
                children: children_buf.as_slice(),
                constraints,
                scale_factor: sf,
                services: &mut *services,
                observe_model: &mut observe_model,
                observe_global: &mut observe_global,
                tree,
            };
            widget.measure(&mut cx)
        });
        if let Some(measure_started) = measure_started {
            const MAX_MEASURE_HOTSPOTS: usize = 16;
            let inclusive_time = measure_started.elapsed();
            let child_inclusive_time = self
                .debug_widget_measure_stack
                .pop()
                .map(|f| f.child_inclusive_time)
                .unwrap_or_default();
            let exclusive_time = inclusive_time.saturating_sub(child_inclusive_time);
            if let Some(parent) = self.debug_widget_measure_stack.last_mut() {
                parent.child_inclusive_time += inclusive_time;
            }
            let element = self.nodes.get(node).and_then(|n| n.element);
            let record = super::UiDebugWidgetMeasureHotspot {
                node,
                element,
                widget_type,
                inclusive_time,
                exclusive_time,
            };
            let idx = self
                .debug_widget_measure_hotspots
                .iter()
                .position(|h| h.inclusive_time < record.inclusive_time)
                .unwrap_or(self.debug_widget_measure_hotspots.len());
            self.debug_widget_measure_hotspots.insert(idx, record);
            if self.debug_widget_measure_hotspots.len() > MAX_MEASURE_HOTSPOTS {
                self.debug_widget_measure_hotspots
                    .truncate(MAX_MEASURE_HOTSPOTS);
            }
        }

        if let Some(profile) = self.measure_node_profile.as_mut() {
            profile.exit(node);
        }

        self.measure_cache_this_frame.insert(key, size);

        if let Some(n) = self.nodes.get_mut(node) {
            n.measure_cache = Some(NodeMeasureCache {
                key: cache_key,
                size,
            });
        }

        self.observed_in_layout
            .record(node, observations.as_slice());
        self.observed_globals_in_layout
            .record(node, global_observations.as_slice());

        let popped = self.measure_stack.pop();
        debug_assert_eq!(popped, Some(key));
        size
    }
}

fn available_space_key(avail: AvailableSpace) -> (u8, u32) {
    match avail {
        AvailableSpace::Definite(px) => (0, px.0.to_bits()),
        AvailableSpace::MinContent => (1, 0),
        AvailableSpace::MaxContent => (2, 0),
    }
}
