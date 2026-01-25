use super::*;
use std::any::TypeId;

use crate::layout_constraints::LayoutSize;
use crate::layout_constraints::{AvailableSpace, LayoutConstraints};
use crate::layout_engine::TaffyLayoutEngine;
use crate::layout_engine::build_viewport_flow_subtree;
use crate::layout_pass::LayoutPassKind;

impl<H: UiHost> UiTree<H> {
    pub(super) fn invalidate_scroll_handle_bindings_for_changed_handles(
        &mut self,
        app: &mut H,
        pass_kind: LayoutPassKind,
    ) {
        if pass_kind != LayoutPassKind::Final {
            return;
        }
        let Some(window) = self.window else {
            return;
        };

        let changed = crate::declarative::frame::take_changed_scroll_handle_keys(app, window);
        if changed.is_empty() {
            return;
        }

        let mut visited = HashMap::<NodeId, u8>::new();
        for change in changed {
            let mut inv = match change.kind {
                crate::declarative::frame::ScrollHandleChangeKind::Layout => Invalidation::Layout,
                crate::declarative::frame::ScrollHandleChangeKind::HitTestOnly => {
                    Invalidation::HitTestOnly
                }
            };
            let handle_key = change.handle_key;
            let bound = crate::declarative::frame::bound_elements_for_scroll_handle(
                &mut *app, window, handle_key,
            );
            if bound.is_empty() {
                continue;
            }

            // If a virtual list requested a scroll-to-item, the scroll handle revision bumps even
            // when offset/viewport/content are unchanged, which makes the change appear as
            // "layout-affecting". For fixed-size virtual lists, we can consume the deferred
            // request up-front (using cached metrics + viewport) and convert it into a simple
            // offset update, avoiding a layout-driven consumption path.
            if inv == Invalidation::Layout {
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
                    let Some(record) =
                        crate::declarative::frame::element_record_for_node(app, window, node)
                    else {
                        continue;
                    };
                    let crate::declarative::frame::ElementInstance::VirtualList(props) =
                        record.instance
                    else {
                        continue;
                    };
                    if props.measure_mode != crate::element::VirtualListMeasureMode::Fixed {
                        continue;
                    }
                    let Some((index, strategy)) = props.scroll_handle.deferred_scroll_to_item()
                    else {
                        continue;
                    };

                    let applied = crate::elements::with_element_state(
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

                            let viewport_size = props.scroll_handle.viewport_size();
                            let viewport = match props.axis {
                                fret_core::Axis::Vertical => Px(viewport_size.height.0.max(0.0)),
                                fret_core::Axis::Horizontal => Px(viewport_size.width.0.max(0.0)),
                            };
                            if viewport.0 <= 0.0 || props.len == 0 {
                                return None;
                            }

                            let current = match props.axis {
                                fret_core::Axis::Vertical => props.scroll_handle.offset().y,
                                fret_core::Axis::Horizontal => props.scroll_handle.offset().x,
                            };
                            let desired = state
                                .metrics
                                .scroll_offset_for_item(index, viewport, current, strategy);
                            let desired = state.metrics.clamp_offset(desired, viewport);

                            match props.axis {
                                fret_core::Axis::Vertical => state.offset_y = desired,
                                fret_core::Axis::Horizontal => state.offset_x = desired,
                            }

                            Some(desired)
                        },
                    );

                    let Some(applied) = applied else {
                        continue;
                    };

                    let prev = props.scroll_handle.offset();
                    match props.axis {
                        fret_core::Axis::Vertical => {
                            props
                                .scroll_handle
                                .set_offset(fret_core::Point::new(prev.x, applied));
                        }
                        fret_core::Axis::Horizontal => {
                            props
                                .scroll_handle
                                .set_offset(fret_core::Point::new(applied, prev.y));
                        }
                    }
                    props.scroll_handle.clear_deferred_scroll_to_item();

                    consumed_scroll_to_item = true;
                    inv = Invalidation::HitTestOnly;
                    self.request_redraw_coalesced(app);
                }
            }

            let mut virtual_list_needs_refresh: Vec<NodeId> = Vec::new();
            for element in bound {
                let Some(node) = crate::declarative::node_for_element_in_window_frame(
                    &mut *app, window, element,
                ) else {
                    continue;
                };
                self.mark_invalidation_dedup_with_detail(
                    node,
                    inv,
                    &mut visited,
                    UiDebugInvalidationSource::Other,
                    UiDebugInvalidationDetail::ScrollHandle,
                );

                // Range escape detection: when scrolling only invalidates hit-testing/paint
                // (`HitTestOnly`), a view-cache root can remain a cache hit and skip rebuilding the
                // virtual list visible range. If the desired range escapes the mounted range,
                // schedule a one-shot rerender of the nearest cache root.
                if inv == Invalidation::HitTestOnly
                    && self.view_cache_enabled()
                    && let Some(record) =
                        crate::declarative::frame::element_record_for_node(app, window, node)
                    && let crate::declarative::frame::ElementInstance::VirtualList(props) =
                        record.instance
                {
                    let requested_refresh = crate::elements::with_element_state(
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

                            let viewport_size = props.scroll_handle.viewport_size();
                            let viewport = match props.axis {
                                fret_core::Axis::Vertical => Px(viewport_size.height.0.max(0.0)),
                                fret_core::Axis::Horizontal => Px(viewport_size.width.0.max(0.0)),
                            };
                            if viewport.0 <= 0.0 || props.len == 0 {
                                return false;
                            }

                            let handle_offset = match props.axis {
                                fret_core::Axis::Vertical => props.scroll_handle.offset().y,
                                fret_core::Axis::Horizontal => props.scroll_handle.offset().x,
                            };
                            let offset = state.metrics.clamp_offset(handle_offset, viewport);
                            let Some(range) =
                                state
                                    .metrics
                                    .visible_range(offset, viewport, props.overscan)
                            else {
                                return false;
                            };
                            crate::virtual_list::virtual_list_needs_visible_range_refresh(
                                &props.visible_items,
                                range,
                            )
                        },
                    );
                    self.debug_record_virtual_list_visible_range_check(requested_refresh);
                    if requested_refresh {
                        virtual_list_needs_refresh.push(node);
                    }
                }
            }

            for node in virtual_list_needs_refresh {
                self.invalidate_with_source_and_detail(
                    node,
                    Invalidation::Paint,
                    UiDebugInvalidationSource::Notify,
                    UiDebugInvalidationDetail::ScrollHandle,
                );
                self.request_redraw_coalesced(app);
            }
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
        let started = self.debug_enabled.then(Instant::now);
        if self.debug_enabled {
            self.begin_debug_frame_if_needed(app.frame_id());
            self.debug_stats.frame_id = app.frame_id();
            self.debug_stats.layout_nodes_visited = 0;
            self.debug_stats.layout_nodes_performed = 0;
            self.debug_stats.layout_engine_solves = 0;
            self.debug_stats.layout_engine_solve_time = Duration::default();
            self.debug_stats.layout_engine_widget_fallback_solves = 0;
            self.debug_stats.view_cache_active = self.view_cache_active();
            self.debug_stats.focus = self.focus;
            self.debug_stats.captured = self.captured_for(fret_core::PointerId(0));
        }

        let roots: Vec<NodeId> = self
            .visible_layers_in_paint_order()
            .map(|layer| self.layers[layer].root)
            .collect();

        self.invalidate_scroll_handle_bindings_for_changed_handles(app, pass_kind);

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

        let mut viewport_cursor: usize = 0;
        if pass_kind == LayoutPassKind::Final {
            self.expand_view_cache_layout_invalidations_if_needed();
        }

        self.request_build_window_roots_if_final(
            app,
            services,
            &roots,
            bounds,
            scale_factor,
            pass_kind,
        );

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

        if pass_kind == LayoutPassKind::Final {
            self.layout_pending_barrier_relayouts_if_needed(
                app,
                services,
                scale_factor,
                pass_kind,
                &mut viewport_cursor,
            );
        }

        if pass_kind == LayoutPassKind::Final {
            self.repair_view_cache_root_bounds_from_engine_if_needed(app);
        }

        if pass_kind == LayoutPassKind::Final {
            self.layout_contained_view_cache_roots_if_needed(
                app,
                services,
                scale_factor,
                pass_kind,
                &mut viewport_cursor,
            );
        }

        if pass_kind == LayoutPassKind::Final {
            self.collapse_layout_observations_to_view_cache_roots_if_needed();
        }

        if self.semantics_requested {
            self.semantics_requested = false;
            self.refresh_semantics_snapshot(app);
        }

        if pass_kind == LayoutPassKind::Final {
            self.prepaint_after_layout(app, scale_factor);
        }

        self.flush_deferred_cleanup(services);

        if let Some(started) = started {
            self.debug_stats.layout_time = started.elapsed();
        }

        if pass_kind == LayoutPassKind::Final {
            self.layout_engine.end_frame();
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
    }

    fn repair_view_cache_root_bounds_from_engine_if_needed(&mut self, app: &mut H) {
        if !self.view_cache_active() {
            return;
        }
        let Some(window) = self.window else {
            return;
        };

        let mut targets: Vec<(NodeId, Rect, Point)> = Vec::new();
        targets.reserve(16);
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

        let mut targets: Vec<(NodeId, Rect)> = Vec::new();
        targets.reserve(16);
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

        if let Ok(filter) = std::env::var("FRET_TAFFY_DUMP_ROOT") {
            if !format!("{root:?}").contains(&filter) {
                return;
            }
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
                serde_json::to_vec_pretty(&wrapped).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::Other, format!("serialize: {e}"))
                })
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

        let sf = scale_factor;
        let available = LayoutSize::new(
            AvailableSpace::Definite(bounds.size.width),
            AvailableSpace::Definite(bounds.size.height),
        );

        let mut engine = self.take_layout_engine();
        engine.set_measure_profiling_enabled(self.debug_enabled);

        // Phase 1: request/build for stable identity, even if we later skip compute/apply.
        for &root in roots {
            if !self
                .nodes
                .get(root)
                .is_some_and(|node| node.element.is_some())
            {
                continue;
            }

            build_viewport_flow_subtree(&mut engine, app, &*self, window, sf, root, bounds.size);
        }

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

        self.put_layout_engine(engine);
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
                    if !self
                        .nodes
                        .get(item.root)
                        .is_some_and(|node| node.element.is_some())
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
        let _ = engine.compute_root_for_node_with_measure_if_needed(
            root,
            available,
            sf,
            |node, constraints| self.measure_in(app, services, node, constraints, sf),
        );

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

        let mut batch: Vec<(NodeId, Rect)> = Vec::new();
        batch.reserve(roots.len());
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

            let _ =
                engine.compute_root_for_node_with_measure_if_needed(root, available, sf, |n, c| {
                    self.measure_in(app, services, n, c, sf)
                });

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

        let size = self.with_widget_mut(node, |widget, tree| {
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

        if !is_probe {
            self.observed_in_layout
                .record(node, observations.as_slice());
            self.observed_globals_in_layout
                .record(node, global_observations.as_slice());
            if let Some(n) = self.nodes.get_mut(node) {
                n.measured_size = size;
                n.invalidation.layout = false;
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
        let key = MeasureStackKey {
            node,
            known_w_bits: constraints.known.width.map(|px| px.0.to_bits()),
            known_h_bits: constraints.known.height.map(|px| px.0.to_bits()),
            avail_w: available_space_key(constraints.available.width),
            avail_h: available_space_key(constraints.available.height),
            scale_bits: scale_factor.to_bits(),
        };
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

        let size = self.with_widget_mut(node, |widget, tree| {
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
