use super::*;

use crate::layout_constraints::LayoutSize;
use crate::layout_constraints::{AvailableSpace, LayoutConstraints};
use crate::layout_engine::build_viewport_flow_subtree;
use crate::layout_pass::LayoutPassKind;

impl<H: UiHost> UiTree<H> {
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
        if pass_kind == LayoutPassKind::Final
            && let Some(window) = self.window
        {
            let frame_id = app.frame_id();
            app.with_global_mut_untracked(
                fret_core::WindowFrameClockService::default,
                |svc, _host| svc.record_frame(window, frame_id),
            );
        }

        let profile_layout_all = crate::runtime_config::ui_runtime_config().layout_all_profile
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
            self.debug_stats.layout_observation_record_time = Duration::default();
            self.debug_stats.layout_observation_record_models_items = 0;
            self.debug_stats.layout_observation_record_globals_items = 0;
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

        let roots_len = roots.len();
        let trace_layout = tracing::enabled!(tracing::Level::TRACE);

        let mut viewport_cursor: usize = 0;

        let layout_phase_time_enabled = self.debug_enabled || profile_layout_all;
        let window = self.window;
        let frame_id = app.frame_id();
        let (_, invalidate_elapsed) = fret_perf::measure_span(
            layout_phase_time_enabled,
            trace_layout,
            || {
                tracing::trace_span!(
                    "fret.ui.layout.invalidate_scroll_handle_bindings",
                    window = ?window,
                    frame_id = frame_id.0,
                    pass_kind = ?pass_kind,
                )
            },
            || {
                self.invalidate_scroll_handle_bindings_for_changed_handles(
                    app, pass_kind, true, true,
                )
            },
        );
        if profile_layout_all {
            t_invalidate_scroll_handle_bindings = invalidate_elapsed;
        }
        if self.debug_enabled
            && let Some(invalidate_elapsed) = invalidate_elapsed
        {
            self.debug_stats
                .layout_invalidate_scroll_handle_bindings_time += invalidate_elapsed;
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
            let (_, expand_elapsed) = fret_perf::measure_span(
                layout_phase_time_enabled,
                trace_layout,
                || {
                    tracing::trace_span!(
                        "fret.ui.layout.expand_view_cache_invalidations",
                        window = ?window,
                        frame_id = frame_id.0,
                        pass_kind = ?pass_kind,
                    )
                },
                || self.expand_view_cache_layout_invalidations_if_needed(),
            );
            if profile_layout_all {
                t_expand_view_cache_invalidations = expand_elapsed;
            }
            if self.debug_enabled
                && let Some(expand_elapsed) = expand_elapsed
            {
                self.debug_stats.layout_expand_view_cache_invalidations_time += expand_elapsed;
            }
        }

        // Fast path (ADR 0175): if nothing requires layout this frame, skip the layout engine and
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

        let (_, request_build_elapsed) = fret_perf::measure_span(
            layout_phase_time_enabled,
            trace_layout,
            || {
                tracing::trace_span!(
                    "fret.ui.layout.request_build_roots",
                    window = ?window,
                    frame_id = frame_id.0,
                    pass_kind = ?pass_kind,
                    roots_len,
                )
            },
            || {
                self.request_build_window_roots_if_final(
                    app,
                    services,
                    &roots,
                    bounds,
                    scale_factor,
                    pass_kind,
                );
            },
        );
        if profile_layout_all {
            t_request_build_roots = request_build_elapsed;
        }
        if self.debug_enabled
            && let Some(request_build_elapsed) = request_build_elapsed
        {
            self.debug_stats.layout_request_build_roots_time += request_build_elapsed;
        }

        let (_, roots_elapsed) = fret_perf::measure_span(
            layout_phase_time_enabled,
            trace_layout,
            || {
                tracing::trace_span!(
                    "fret.ui.layout.roots",
                    window = ?window,
                    frame_id = frame_id.0,
                    pass_kind = ?pass_kind,
                    roots_len,
                )
            },
            || {
                for root in roots {
                    let _ = self.layout_in_with_pass_kind(
                        app,
                        services,
                        root,
                        bounds,
                        scale_factor,
                        pass_kind,
                    );

                    self.flush_viewport_roots_after_root(
                        app,
                        services,
                        scale_factor,
                        pass_kind,
                        &mut viewport_cursor,
                    );
                }
            },
        );
        if profile_layout_all {
            t_layout_roots = roots_elapsed;
        }
        if self.debug_enabled
            && let Some(roots_elapsed) = roots_elapsed
        {
            self.debug_stats.layout_roots_time += roots_elapsed;
        }

        if pass_kind == LayoutPassKind::Final {
            let (_, barrier_elapsed) = fret_perf::measure_span(
                layout_phase_time_enabled,
                trace_layout,
                || {
                    tracing::trace_span!(
                        "fret.ui.layout.pending_barriers",
                        window = ?window,
                        frame_id = frame_id.0,
                        pass_kind = ?pass_kind,
                    )
                },
                || {
                    self.layout_pending_barrier_relayouts_if_needed(
                        app,
                        services,
                        scale_factor,
                        pass_kind,
                        &mut viewport_cursor,
                    );
                },
            );
            if profile_layout_all {
                t_pending_barriers = barrier_elapsed;
            }
            if self.debug_enabled
                && let Some(barrier_elapsed) = barrier_elapsed
            {
                self.debug_stats.layout_barrier_relayouts_time += barrier_elapsed;
                self.debug_stats.layout_pending_barrier_relayouts_time += barrier_elapsed;
            }
        }

        if pass_kind == LayoutPassKind::Final {
            let (_, view_cache_elapsed) = fret_perf::measure_span(
                layout_phase_time_enabled,
                trace_layout,
                || {
                    tracing::trace_span!(
                        "fret.ui.layout.view_cache",
                        window = ?window,
                        frame_id = frame_id.0,
                        pass_kind = ?pass_kind,
                    )
                },
                || {
                    let (_, repair_elapsed) = fret_perf::measure_span(
                        layout_phase_time_enabled,
                        trace_layout,
                        || {
                            tracing::trace_span!(
                                "fret.ui.layout.view_cache.repair_bounds",
                                window = ?window,
                                frame_id = frame_id.0,
                                pass_kind = ?pass_kind,
                            )
                        },
                        || self.repair_view_cache_root_bounds_from_engine_if_needed(app),
                    );
                    if profile_layout_all {
                        t_repair_view_cache_bounds = repair_elapsed;
                    }
                    if self.debug_enabled
                        && let Some(repair_elapsed) = repair_elapsed
                    {
                        self.debug_stats.layout_repair_view_cache_bounds_time += repair_elapsed;
                    }

                    let (_, contained_elapsed) = fret_perf::measure_span(
                        layout_phase_time_enabled,
                        trace_layout,
                        || {
                            tracing::trace_span!(
                                "fret.ui.layout.view_cache.layout_contained_roots",
                                window = ?window,
                                frame_id = frame_id.0,
                                pass_kind = ?pass_kind,
                            )
                        },
                        || {
                            self.layout_contained_view_cache_roots_if_needed(
                                app,
                                services,
                                scale_factor,
                                pass_kind,
                                &mut viewport_cursor,
                            );
                        },
                    );
                    if profile_layout_all {
                        t_layout_contained_view_cache_roots = contained_elapsed;
                    }
                    if self.debug_enabled
                        && let Some(contained_elapsed) = contained_elapsed
                    {
                        self.debug_stats.layout_contained_view_cache_roots_time +=
                            contained_elapsed;
                    }

                    let (_, collapse_elapsed) = fret_perf::measure_span(
                        layout_phase_time_enabled,
                        trace_layout,
                        || {
                            tracing::trace_span!(
                                "fret.ui.layout.view_cache.collapse_observations",
                                window = ?window,
                                frame_id = frame_id.0,
                                pass_kind = ?pass_kind,
                            )
                        },
                        || self.collapse_layout_observations_to_view_cache_roots_if_needed(),
                    );
                    if profile_layout_all {
                        t_collapse_layout_observations = collapse_elapsed;
                    }
                    if self.debug_enabled
                        && let Some(collapse_elapsed) = collapse_elapsed
                    {
                        self.debug_stats.layout_collapse_layout_observations_time +=
                            collapse_elapsed;
                    }
                },
            );
            if self.debug_enabled
                && let Some(view_cache_elapsed) = view_cache_elapsed
            {
                self.debug_stats.layout_view_cache_time += view_cache_elapsed;
            }
        }

        if self.semantics_requested {
            let (_, semantics_elapsed) = fret_perf::measure_span(
                layout_phase_time_enabled,
                trace_layout,
                || {
                    tracing::trace_span!(
                        "fret.ui.layout.refresh_semantics",
                        window = ?window,
                        frame_id = frame_id.0,
                        pass_kind = ?pass_kind,
                    )
                },
                || {
                    self.semantics_requested = false;
                    self.refresh_semantics_snapshot(app);
                },
            );
            if profile_layout_all {
                t_refresh_semantics = semantics_elapsed;
            }
            if self.debug_enabled
                && let Some(semantics_elapsed) = semantics_elapsed
            {
                self.debug_stats.layout_semantics_refresh_time += semantics_elapsed;
            }
        }
        if pass_kind == LayoutPassKind::Final {
            let (_, prepaint_elapsed) = fret_perf::measure_span(
                layout_phase_time_enabled,
                trace_layout,
                || {
                    tracing::trace_span!(
                        "fret.ui.layout.prepaint_after_layout",
                        window = ?window,
                        frame_id = frame_id.0,
                        pass_kind = ?pass_kind,
                    )
                },
                || self.prepaint_after_layout(app, scale_factor),
            );
            if profile_layout_all {
                t_prepaint_after_layout = prepaint_elapsed;
            }
            if self.debug_enabled
                && let Some(prepaint_elapsed) = prepaint_elapsed
            {
                self.debug_stats.layout_prepaint_after_layout_time += prepaint_elapsed;
            }
        }
        if pass_kind == LayoutPassKind::Final {
            let (_, focus_elapsed) = fret_perf::measure_span(
                self.debug_enabled,
                trace_layout,
                || {
                    tracing::trace_span!(
                        "fret.ui.layout.focus_repair",
                        window = ?window,
                        frame_id = frame_id.0,
                        pass_kind = ?pass_kind,
                    )
                },
                || self.repair_focus_node_from_focused_element_if_needed(app),
            );
            if let Some(focus_elapsed) = focus_elapsed {
                self.debug_stats.layout_focus_repair_time += focus_elapsed;
            }
        }
        let (_, deferred_cleanup_elapsed) = fret_perf::measure_span(
            layout_phase_time_enabled,
            trace_layout,
            || {
                tracing::trace_span!(
                    "fret.ui.layout.flush_deferred_cleanup",
                    window = ?window,
                    frame_id = frame_id.0,
                    pass_kind = ?pass_kind,
                )
            },
            || self.flush_deferred_cleanup(services),
        );
        if profile_layout_all {
            t_flush_deferred_cleanup = deferred_cleanup_elapsed;
        }
        if self.debug_enabled
            && let Some(deferred_cleanup_elapsed) = deferred_cleanup_elapsed
        {
            self.debug_stats.layout_deferred_cleanup_time += deferred_cleanup_elapsed;
        }

        // layout_time is computed below, and should exclude prepaint_after_layout time (since that
        // work is accounted separately and runs even on "layout fast path" frames).

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
                collapse_layout_observations_ms =
                    t_collapse_layout_observations.map(|d| d.as_millis()),
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
            if crate::runtime_config::ui_runtime_config().debug_focus_repair {
                eprintln!("focus_repair: focused={focused:?} has no element");
            }
            return;
        };
        let Some(canonical) = crate::elements::node_for_element(app, window, element) else {
            #[cfg(debug_assertions)]
            if crate::runtime_config::ui_runtime_config().debug_focus_repair {
                eprintln!(
                    "focus_repair: focused={focused:?} element={element:?} has no canonical node",
                );
            }
            return;
        };
        #[cfg(debug_assertions)]
        if crate::runtime_config::ui_runtime_config().debug_focus_repair {
            eprintln!(
                "focus_repair: focused={focused:?} element={element:?} canonical={canonical:?} canonical_exists={}",
                self.node_exists(canonical)
            );
        }
        if canonical != focused && self.node_exists(canonical) {
            self.set_focus(Some(canonical));
        }
    }

    fn repair_view_cache_root_bounds_from_engine_if_needed(&mut self, _app: &mut H) {
        if !self.view_cache_active() {
            return;
        }

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

        let nodes = &self.nodes;
        let scratch_element_nodes = &mut self.scratch_element_nodes;

        crate::elements::with_window_state(app, window, |st| {
            st.element_nodes_copy_into(scratch_element_nodes);
            for &(element, node) in scratch_element_nodes.iter() {
                let Some(rect) = nodes.get(node).map(|n| n.bounds) else {
                    continue;
                };
                st.record_bounds(element, rect);
            }
        });
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

        // If both an ancestor and a descendant cache root are invalidated in the same frame, only
        // relayout the ancestor; it will already relayout the subtree.
        //
        // Hot path: avoid scanning the whole node store. Cache-root invalidations are tracked in
        // `dirty_cache_roots`, so we can restrict this pass to the subset that actually changed.
        let mut candidates: Vec<NodeId> = Vec::with_capacity(16);
        for &id in &self.dirty_cache_roots {
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            if !node.view_cache.enabled || !node.view_cache.contained_layout {
                continue;
            }
            if !node.invalidation.layout {
                continue;
            }
            candidates.push(id);
        }

        if candidates.is_empty() {
            return;
        }

        let candidate_set: std::collections::HashSet<NodeId> = candidates.iter().copied().collect();

        let mut targets: Vec<(NodeId, Rect)> = Vec::with_capacity(candidates.len());
        for id in candidates {
            let mut skip = false;
            let mut parent = self.nodes.get(id).and_then(|n| n.parent);
            while let Some(p) = parent {
                if candidate_set.contains(&p) {
                    skip = true;
                    break;
                }
                parent = self.nodes.get(p).and_then(|n| n.parent);
            }
            if skip {
                continue;
            }

            let Some(node) = self.nodes.get(id) else {
                continue;
            };

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

        let runtime_cfg = crate::runtime_config::ui_runtime_config();
        let profile_layout = runtime_cfg.layout_profile;
        let total_started = profile_layout.then(Instant::now);

        let sf = scale_factor;
        let available = LayoutSize::new(
            AvailableSpace::Definite(bounds.size.width),
            AvailableSpace::Definite(bounds.size.height),
        );

        let mut engine = self.take_layout_engine();
        engine.set_measure_profiling_enabled(self.debug_enabled && profile_layout);

        let phase1_started = profile_layout.then(Instant::now);
        let reuse_cached_flow = self.interactive_resize_active();
        let allow_translation_only_skip = runtime_cfg.layout_skip_request_build_translation_only;
        // Phase 1: request/build for stable identity, even if we later skip compute/apply.
        for &root in roots {
            let Some((has_element, layout_invalidated, prev_bounds, measured)) =
                self.nodes.get(root).map(|node| {
                    (
                        node.element.is_some(),
                        node.invalidation.layout,
                        node.bounds,
                        node.measured_size,
                    )
                })
            else {
                continue;
            };
            if !has_element {
                continue;
            }

            let needs_layout = layout_invalidated || prev_bounds != bounds;
            let is_translation_only = allow_translation_only_skip
                && !layout_invalidated
                && prev_bounds.size == bounds.size
                && prev_bounds.origin != bounds.origin
                && measured != Size::default();

            if engine.layout_id_for_node(root).is_some() && (!needs_layout || is_translation_only) {
                engine.mark_seen_subtree_from_cached_children(root);
                continue;
            }
            if reuse_cached_flow && engine.layout_id_for_node(root).is_some() && !layout_invalidated
            {
                engine.set_viewport_root_override_size(root, bounds.size, sf);
                engine.mark_seen_subtree_from_cached_children(root);
            } else {
                build_viewport_flow_subtree(
                    &mut engine,
                    app,
                    &*self,
                    window,
                    sf,
                    root,
                    bounds.size,
                );
            }
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
                let solve_root = engine.last_solve_root().unwrap_or(root);
                let root_element = self.nodes.get(solve_root).and_then(|n| n.element);
                let root_element_kind =
                    crate::declarative::frame::element_record_for_node(app, window, solve_root)
                        .map(|record| record.instance.kind_name());
                let root_element_path: Option<String> = root_element.and_then(|element| {
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

                self.debug_record_layout_engine_solve(
                    solve_root,
                    root_element,
                    root_element_kind,
                    root_element_path,
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
                layout_invalidated: bool,
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
                    layout_invalidated: invalidated,
                });
            }

            if pass_kind == LayoutPassKind::Final
                && let Some(window) = window
            {
                let mut engine = self.take_layout_engine();
                engine.set_measure_profiling_enabled(
                    self.debug_enabled && crate::runtime_config::ui_runtime_config().layout_profile,
                );

                let reuse_cached_flow = self.interactive_resize_active();

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
                    if engine.layout_id_for_node(item.root).is_some()
                        && (!item.needs_layout || item.is_translation_only)
                    {
                        engine.mark_seen_subtree_from_cached_children(item.root);
                        continue;
                    }
                    if reuse_cached_flow
                        && engine.layout_id_for_node(item.root).is_some()
                        && !item.layout_invalidated
                    {
                        engine.set_viewport_root_override_size(item.root, item.bounds.size, sf);
                        engine.mark_seen_subtree_from_cached_children(item.root);
                    } else {
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
                        let solve_root = engine.last_solve_root().unwrap_or(item.root);
                        let root_element = self.nodes.get(solve_root).and_then(|n| n.element);
                        let root_element_kind = crate::declarative::frame::element_record_for_node(
                            app, window, solve_root,
                        )
                        .map(|record| record.instance.kind_name());
                        let root_element_path: Option<String> = root_element.and_then(|element| {
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

                        self.debug_record_layout_engine_solve(
                            solve_root,
                            root_element,
                            root_element_kind,
                            root_element_path,
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
}
