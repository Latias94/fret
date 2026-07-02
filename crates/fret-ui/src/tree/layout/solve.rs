use super::*;

use crate::layout_constraints::AvailableSpace;
use crate::layout_constraints::LayoutSize;

impl<H: UiHost> UiTree<H> {
    fn build_or_reuse_barrier_flow_subtree_for_resize(
        &mut self,
        engine: &mut crate::layout_engine::TaffyLayoutEngine,
        app: &mut H,
        window: AppWindowId,
        scale_factor: f32,
        root: NodeId,
        root_size: Size,
        available: LayoutSize<AvailableSpace>,
    ) {
        let reuse_cached_flow = self.interactive_resize_active();
        let root_layout_invalidated = self
            .nodes
            .get(root)
            .is_some_and(|node| node.invalidation.layout);
        let root_subtree_layout_dirty = self.node_subtree_layout_dirty(root);
        if reuse_cached_flow
            && engine.layout_id_for_node(root).is_some()
            && !root_layout_invalidated
            && !root_subtree_layout_dirty
        {
            // Barrier child roots are already explicit, authoritative layout roots. Reusing their
            // cached flow during resize only updates the local containing block; dirty descendants
            // still rebuild through normal invalidation. Do not arm the global post-resize rebuild
            // flag here, because that flag invalidates the whole window root and turns steady
            // resize samples into full-tree settle spikes.
            engine.set_viewport_root_override(
                root,
                crate::layout_engine::ViewportRootOverride::from_available(root_size, available),
                scale_factor,
            );
            self.mark_layout_engine_seen_subtree_from_ui_children(engine, root);
        } else {
            crate::layout_engine::build_viewport_flow_subtree_for_available(
                engine,
                app,
                &*self,
                window,
                scale_factor,
                root,
                root_size,
                available,
            );
        }
    }

    fn definite_available_for_bounds(root_bounds: Rect) -> LayoutSize<AvailableSpace> {
        LayoutSize::new(
            AvailableSpace::Definite(root_bounds.size.width),
            AvailableSpace::Definite(root_bounds.size.height),
        )
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
            crate::runtime_config::ui_runtime_config().layout_forbid_widget_fallback_solves;
        let trace_fallback_solves =
            crate::runtime_config::ui_runtime_config().layout_trace_widget_fallback_solves;

        if trace_fallback_solves {
            let label = crate::declarative::frame::element_record_for_node(app, window, node)
                .map(|r| r.instance);
            let missing_label = missing_child.and_then(|child| {
                crate::declarative::frame::element_record_for_node(app, window, child)
                    .map(|r| r.instance)
            });
            let miss_debug = missing_child
                .map(|child| self.layout_engine.debug_child_layout_rect_miss(node, child));

            tracing::warn!(
                window = ?self.window,
                node = ?node,
                widget_kind,
                label = ?label,
                missing_child = ?missing_child,
                missing_label = ?missing_label,
                child_rect_miss = ?miss_debug,
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
        self.solve_barrier_flow_root_with_available(
            app,
            services,
            root,
            root_bounds,
            Self::definite_available_for_bounds(root_bounds),
            scale_factor,
        );
    }

    pub(crate) fn solve_barrier_flow_root_with_available(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        root_bounds: Rect,
        available: LayoutSize<AvailableSpace>,
        scale_factor: f32,
    ) {
        let Some(window) = self.window else {
            return;
        };

        let both_axes_definite = available.width.is_definite() && available.height.is_definite();
        if both_axes_definite
            && self
                .layout_engine
                .root_is_solved_for(root, available, scale_factor)
        {
            return;
        }

        let mut engine = self.take_layout_engine();
        engine.set_measure_profiling_enabled(
            self.debug_enabled && crate::runtime_config::ui_runtime_config().layout_profile,
        );
        self.build_or_reuse_barrier_flow_subtree_for_resize(
            &mut engine,
            app,
            window,
            scale_factor,
            root,
            root_bounds.size,
            available,
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
            let solve_root = engine.last_solve_root().unwrap_or(root);
            let (root_element, root_element_kind, root_element_path) =
                self.debug_resolve_layout_solve_root_label(app, window, solve_root);

            self.debug_record_layout_engine_solve(
                solve_root,
                root_element,
                root_element_kind,
                root_element_path,
                elapsed,
                engine.last_solve_profile(),
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
        self.solve_barrier_flow_root_if_needed_with_available(
            app,
            services,
            root,
            root_bounds,
            Self::definite_available_for_bounds(root_bounds),
            scale_factor,
        );
    }

    pub(crate) fn solve_barrier_flow_root_if_needed_with_available(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        root_bounds: Rect,
        available: LayoutSize<AvailableSpace>,
        scale_factor: f32,
    ) {
        let Some(node) = self.nodes.get(root) else {
            return;
        };

        let both_axes_definite = available.width.is_definite() && available.height.is_definite();
        let solved_for_available =
            self.layout_engine
                .root_is_solved_for(root, available, scale_factor);
        let needs_layout =
            node.invalidation.layout || node.bounds != root_bounds || !solved_for_available;
        if !needs_layout {
            return;
        }

        let is_translation_only = !node.invalidation.layout
            && node.bounds.size == root_bounds.size
            && node.bounds.origin != root_bounds.origin
            && node.measured_size != Size::default();
        if is_translation_only && solved_for_available {
            return;
        }
        let prev_bounds = node.bounds;
        let engine_backed = self.layout_engine.layout_id_for_node(root).is_some();
        if engine_backed
            && both_axes_definite
            && !node.invalidation.layout
            && self.can_skip_clean_geometry_engine_solve_for_resize(
                app,
                root,
                root_bounds,
                prev_bounds,
                scale_factor,
            )
        {
            self.note_interactive_resize_cached_flow_reuse();
            return;
        }

        self.solve_barrier_flow_root_with_available(
            app,
            services,
            root,
            root_bounds,
            available,
            scale_factor,
        );
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
        let roots_with_available: Vec<_> = roots
            .iter()
            .map(|&(root, bounds)| (root, bounds, Self::definite_available_for_bounds(bounds)))
            .collect();
        self.solve_barrier_flow_roots_if_needed_with_available(
            app,
            services,
            &roots_with_available,
            scale_factor,
        );
    }

    pub(crate) fn solve_barrier_flow_roots_if_needed_with_available(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        roots: &[(NodeId, Rect, LayoutSize<AvailableSpace>)],
        scale_factor: f32,
    ) {
        let Some(window) = self.window else {
            return;
        };
        if roots.is_empty() {
            return;
        }

        let mut batch: Vec<(NodeId, Rect, LayoutSize<AvailableSpace>)> =
            Vec::with_capacity(roots.len());
        for &(root, root_bounds, available) in roots {
            let Some(node) = self.nodes.get(root) else {
                continue;
            };

            let both_axes_definite =
                available.width.is_definite() && available.height.is_definite();
            let solved_for_available =
                self.layout_engine
                    .root_is_solved_for(root, available, scale_factor);
            let needs_layout =
                node.invalidation.layout || node.bounds != root_bounds || !solved_for_available;
            if !needs_layout {
                continue;
            }

            let is_translation_only = !node.invalidation.layout
                && node.bounds.size == root_bounds.size
                && node.bounds.origin != root_bounds.origin
                && node.measured_size != Size::default();
            if is_translation_only && solved_for_available {
                continue;
            }
            let engine_backed = self.layout_engine.layout_id_for_node(root).is_some();
            if engine_backed
                && both_axes_definite
                && !node.invalidation.layout
                && self.can_skip_clean_geometry_engine_solve_for_resize(
                    app,
                    root,
                    root_bounds,
                    node.bounds,
                    scale_factor,
                )
            {
                self.note_interactive_resize_cached_flow_reuse();
                continue;
            }

            batch.push((root, root_bounds, available));
        }

        if batch.is_empty() {
            return;
        }

        let mut engine = self.take_layout_engine();
        engine.set_measure_profiling_enabled(
            self.debug_enabled && crate::runtime_config::ui_runtime_config().layout_profile,
        );
        for &(root, root_bounds, available) in &batch {
            self.build_or_reuse_barrier_flow_subtree_for_resize(
                &mut engine,
                app,
                window,
                scale_factor,
                root,
                root_bounds.size,
                available,
            );
        }

        let sf = scale_factor;
        let mut pending_solves: Vec<(NodeId, LayoutSize<AvailableSpace>)> =
            Vec::with_capacity(batch.len());
        for &(root, _root_bounds, available) in &batch {
            pending_solves.push((root, available));
        }

        let solves_before = engine.solve_count();
        let solve_time_before = engine.last_solve_time();
        engine.compute_independent_roots_with_measure_if_needed(&pending_solves, sf, |n, c| {
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
            let solve_root = engine
                .last_solve_root()
                .unwrap_or_else(|| pending_solves[0].0);
            let (root_element, root_element_kind, root_element_path) =
                self.debug_resolve_layout_solve_root_label(app, window, solve_root);

            self.debug_record_layout_engine_solve(
                solve_root,
                root_element,
                root_element_kind,
                root_element_path,
                elapsed,
                engine.last_solve_profile(),
                engine.last_solve_measure_calls(),
                engine.last_solve_measure_cache_hits(),
                engine.last_solve_measure_time(),
                top_measures,
            );
            self.debug_measure_children.clear();
        }

        for &(root, root_bounds, _available) in &batch {
            self.maybe_dump_taffy_subtree(app, window, &engine, root, root_bounds, sf);
        }

        self.put_layout_engine(engine);
    }
}
