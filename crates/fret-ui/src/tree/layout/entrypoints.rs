use super::*;

use crate::layout_constraints::LayoutSize;
use crate::layout_constraints::{AvailableSpace, LayoutConstraints};
use crate::layout_engine::build_viewport_flow_subtree;
use crate::layout_pass::LayoutPassKind;

#[derive(Default)]
struct LayoutAllProfileTimings {
    collect_roots: Option<Duration>,
    invalidate_scroll_handle_bindings: Option<Duration>,
    expand_view_cache_invalidations: Option<Duration>,
    request_build_roots: Option<Duration>,
    layout_roots: Option<Duration>,
    pending_barriers: Option<Duration>,
    repair_view_cache_bounds: Option<Duration>,
    layout_contained_view_cache_roots: Option<Duration>,
    collapse_layout_observations: Option<Duration>,
    refresh_semantics: Option<Duration>,
    prepaint_after_layout: Option<Duration>,
    focus_repair: Option<Duration>,
    flush_deferred_cleanup: Option<Duration>,
}

#[derive(Clone, Copy)]
enum LayoutPrepaintPhase {
    StableFrame,
    AfterLayout { scale_factor: f32 },
}

#[derive(Clone, Copy)]
struct LayoutPostLayoutPhaseOptions {
    prepaint: Option<LayoutPrepaintPhase>,
    repair_focus: bool,
    resolve_pending_focus_target: bool,
    time_enabled: bool,
    trace_enabled: bool,
    window: Option<AppWindowId>,
    frame_id: FrameId,
    pass_kind: LayoutPassKind,
}

#[derive(Default)]
struct LayoutPostLayoutPhaseTimings {
    prepaint_after_layout: Option<Duration>,
    focus_repair: Option<Duration>,
    refresh_semantics: Option<Duration>,
    flush_deferred_cleanup: Option<Duration>,
}

impl LayoutAllProfileTimings {
    fn record_post_layout(&mut self, enabled: bool, timings: LayoutPostLayoutPhaseTimings) {
        if !enabled {
            return;
        }

        self.prepaint_after_layout = timings.prepaint_after_layout;
        self.focus_repair = timings.focus_repair;
        self.refresh_semantics = timings.refresh_semantics;
        self.flush_deferred_cleanup = timings.flush_deferred_cleanup;
    }

    fn emit<H: UiHost>(&self, tree: &UiTree<H>, started: Option<Instant>) {
        let Some(started) = started else {
            return;
        };

        let total = started.elapsed();
        tracing::info!(
            window = ?tree.window,
            total_ms = total.as_millis(),
            collect_roots_ms = self.collect_roots.map(|d| d.as_millis()),
            invalidate_scroll_handle_bindings_ms =
                self.invalidate_scroll_handle_bindings.map(|d| d.as_millis()),
            expand_view_cache_invalidations_ms =
                self.expand_view_cache_invalidations.map(|d| d.as_millis()),
            request_build_roots_ms = self.request_build_roots.map(|d| d.as_millis()),
            layout_roots_ms = self.layout_roots.map(|d| d.as_millis()),
            pending_barriers_ms = self.pending_barriers.map(|d| d.as_millis()),
            repair_view_cache_bounds_ms = self.repair_view_cache_bounds.map(|d| d.as_millis()),
            layout_contained_view_cache_roots_ms =
                self.layout_contained_view_cache_roots.map(|d| d.as_millis()),
            collapse_layout_observations_ms =
                self.collapse_layout_observations.map(|d| d.as_millis()),
            refresh_semantics_ms = self.refresh_semantics.map(|d| d.as_millis()),
            prepaint_after_layout_ms = self.prepaint_after_layout.map(|d| d.as_millis()),
            focus_repair_ms = self.focus_repair.map(|d| d.as_millis()),
            flush_deferred_cleanup_ms = self.flush_deferred_cleanup.map(|d| d.as_millis()),
            layout_nodes_performed = tree.debug_stats.layout_nodes_performed,
            "layout_all profile"
        );
    }
}

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
        let mut profile_timings = LayoutAllProfileTimings::default();

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
        self.scratch_bounds_records.clear();

        if pass_kind == LayoutPassKind::Final {
            self.update_interactive_resize_state_for_layout(app.frame_id(), bounds, scale_factor);
            self.prune_detached_layout_followups();
        }
        let force_post_resize_rebuild =
            pass_kind == LayoutPassKind::Final && self.interactive_resize_requires_full_rebuild();

        let started = self.debug_enabled.then(Instant::now);
        if self.debug_enabled {
            self.begin_debug_frame_if_needed(app.frame_id());
            self.debug_stats.frame_id = app.frame_id();
            self.debug_stats.layout_nodes_visited = 0;
            self.debug_stats.layout_nodes_performed = 0;
            self.debug_stats.layout_engine_solves = 0;
            self.debug_stats.layout_engine_solve_time = Duration::default();
            self.debug_stats.layout_clean_geometry_solve_skip_rejections = 0;
            self.debug_stats
                .layout_clean_geometry_solve_skip_first_rejection = None;
            self.debug_stats
                .layout_clean_geometry_solve_skip_first_element_kind = None;
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

        let trace_layout = tracing::enabled!(tracing::Level::TRACE);
        let layout_phase_time_enabled = self.debug_enabled || profile_layout_all;
        let window = self.window;
        let frame_id = app.frame_id();

        let (roots, collect_roots_elapsed) = fret_perf::measure_span(
            layout_phase_time_enabled,
            trace_layout,
            || {
                tracing::trace_span!(
                    "fret.ui.layout.collect_roots",
                    window = ?window,
                    frame_id = frame_id.0,
                    pass_kind = ?pass_kind,
                )
            },
            || {
                self.visible_layers_in_paint_order()
                    .map(|layer| self.layers[layer].root)
                    .collect::<Vec<NodeId>>()
            },
        );
        if profile_layout_all {
            profile_timings.collect_roots = collect_roots_elapsed;
        }
        if self.debug_enabled
            && let Some(collect_roots_elapsed) = collect_roots_elapsed
        {
            self.debug_stats.layout_collect_roots_time += collect_roots_elapsed;
        }

        let roots_len = roots.len();
        let mut viewport_cursor: usize = 0;

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
            profile_timings.invalidate_scroll_handle_bindings = invalidate_elapsed;
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
            self.node_is_attached_to_layer_tree(root)
                && !self.node_layout_dirty_suppressed_by_ancestor(root)
                && self
                    .nodes
                    .get(root)
                    .is_some_and(|node| node.invalidation.layout)
        });
        let any_view_cache_root_needs_layout = self.view_cache_active()
            && self.nodes.iter().any(|(id, node)| {
                self.node_is_attached_to_layer_tree(id)
                    && !self.node_layout_dirty_suppressed_by_ancestor(id)
                    && node.view_cache.enabled
                    && node.invalidation.layout
            });

        if pass_kind == LayoutPassKind::Final
            && !any_root_needs_layout_or_bounds
            && !any_view_cache_root_needs_layout
            && !any_pending_barrier_needs_layout
            && self.invalidated_paint_nodes == 0
            && self.invalidated_hit_test_nodes == 0
            && !force_post_resize_rebuild
        {
            self.pending_barrier_relayouts.retain(|&root| {
                self.nodes
                    .get(root)
                    .is_some_and(|node| node.invalidation.layout)
            });
            self.debug_stats.layout_skipped_engine_frame = true;
            let post_layout_timings = self.run_layout_post_layout_phases(
                app,
                services,
                LayoutPostLayoutPhaseOptions {
                    prepaint: Some(LayoutPrepaintPhase::StableFrame),
                    repair_focus: true,
                    resolve_pending_focus_target: true,
                    time_enabled: layout_phase_time_enabled,
                    trace_enabled: trace_layout,
                    window,
                    frame_id,
                    pass_kind,
                },
            );
            profile_timings.record_post_layout(profile_layout_all, post_layout_timings);
            self.last_layout_frame_id = Some(app.frame_id());
            if let Some(started) = started {
                self.debug_stats.layout_time = started
                    .elapsed()
                    .saturating_sub(self.debug_stats.layout_prepaint_after_layout_time);
            }
            self.refine_pending_window_runtime_snapshots_after_layout(app);
            profile_timings.emit(self, profile_started);
            self.emit_final_layout_profiles_if_needed(app, pass_kind);
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
                profile_timings.expand_view_cache_invalidations = expand_elapsed;
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
            && !self.any_attached_layout_invalidations()
            && !force_post_resize_rebuild
        {
            self.debug_stats.layout_fast_path_taken = true;
            let post_layout_timings = self.run_layout_post_layout_phases(
                app,
                services,
                LayoutPostLayoutPhaseOptions {
                    prepaint: Some(LayoutPrepaintPhase::AfterLayout { scale_factor }),
                    repair_focus: true,
                    resolve_pending_focus_target: false,
                    time_enabled: layout_phase_time_enabled,
                    trace_enabled: trace_layout,
                    window,
                    frame_id,
                    pass_kind,
                },
            );
            profile_timings.record_post_layout(profile_layout_all, post_layout_timings);
            self.last_layout_frame_id = Some(app.frame_id());

            self.last_layout_bounds = Some(bounds);
            self.last_layout_scale_factor = Some(scale_factor);

            if let Some(started) = started {
                self.debug_stats.layout_time = started
                    .elapsed()
                    .saturating_sub(self.debug_stats.layout_prepaint_after_layout_time);
            }
            self.refine_pending_window_runtime_snapshots_after_layout(app);
            profile_timings.emit(self, profile_started);
            self.emit_final_layout_profiles_if_needed(app, pass_kind);
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
            profile_timings.request_build_roots = request_build_elapsed;
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
                        crate::layout::overflow::LayoutOverflowContext::default(),
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
            profile_timings.layout_roots = roots_elapsed;
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
                profile_timings.pending_barriers = barrier_elapsed;
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
                        profile_timings.repair_view_cache_bounds = repair_elapsed;
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
                        profile_timings.layout_contained_view_cache_roots = contained_elapsed;
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
                        profile_timings.collapse_layout_observations = collapse_elapsed;
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

        let final_pass = pass_kind == LayoutPassKind::Final;
        if final_pass {
            self.flush_layout_bounds_records_if_needed(app);
        }
        let post_layout_timings = self.run_layout_post_layout_phases(
            app,
            services,
            LayoutPostLayoutPhaseOptions {
                prepaint: final_pass.then_some(LayoutPrepaintPhase::AfterLayout { scale_factor }),
                repair_focus: final_pass,
                resolve_pending_focus_target: final_pass,
                time_enabled: layout_phase_time_enabled,
                trace_enabled: trace_layout,
                window,
                frame_id,
                pass_kind,
            },
        );
        profile_timings.record_post_layout(profile_layout_all, post_layout_timings);

        // layout_time is computed below, and should exclude prepaint_after_layout time (since that
        // work is accounted separately and runs even on "layout fast path" frames).

        if let Some(started) = started {
            self.debug_stats.layout_time = started
                .elapsed()
                .saturating_sub(self.debug_stats.layout_prepaint_after_layout_time);
        }

        if pass_kind == LayoutPassKind::Final {
            self.finish_final_layout_frame(app);
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

        profile_timings.emit(self, profile_started);
        self.emit_final_layout_profiles_if_needed(app, pass_kind);
    }

    fn run_layout_post_layout_phases(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        options: LayoutPostLayoutPhaseOptions,
    ) -> LayoutPostLayoutPhaseTimings {
        let mut timings = LayoutPostLayoutPhaseTimings::default();

        if let Some(prepaint) = options.prepaint {
            let (_, elapsed) = match prepaint {
                LayoutPrepaintPhase::StableFrame => fret_perf::measure_span(
                    options.time_enabled,
                    options.trace_enabled,
                    || {
                        tracing::trace_span!(
                            "fret.ui.layout.prepaint_after_layout_stable_frame",
                            window = ?options.window,
                            frame_id = options.frame_id.0,
                            pass_kind = ?options.pass_kind,
                        )
                    },
                    || self.prepaint_after_layout_stable_frame(app),
                ),
                LayoutPrepaintPhase::AfterLayout { scale_factor } => fret_perf::measure_span(
                    options.time_enabled,
                    options.trace_enabled,
                    || {
                        tracing::trace_span!(
                            "fret.ui.layout.prepaint_after_layout",
                            window = ?options.window,
                            frame_id = options.frame_id.0,
                            pass_kind = ?options.pass_kind,
                            scale_factor,
                        )
                    },
                    || {
                        self.prepaint_after_layout(
                            app,
                            PrepaintAfterLayoutInputs::new(services, scale_factor),
                        )
                    },
                ),
            };
            timings.prepaint_after_layout = elapsed;
            if self.debug_enabled
                && let Some(elapsed) = elapsed
            {
                self.debug_stats.layout_prepaint_after_layout_time += elapsed;
            }
        }

        if options.repair_focus {
            let (_, elapsed) = fret_perf::measure_span(
                options.time_enabled,
                options.trace_enabled,
                || {
                    tracing::trace_span!(
                        "fret.ui.layout.focus_repair",
                        window = ?options.window,
                        frame_id = options.frame_id.0,
                        pass_kind = ?options.pass_kind,
                        resolve_pending_focus_target = options.resolve_pending_focus_target,
                    )
                },
                || {
                    if options.resolve_pending_focus_target {
                        self.resolve_pending_focus_target_if_needed(app);
                    }
                    self.repair_focus_node_from_focused_element_if_needed(app)
                },
            );
            timings.focus_repair = elapsed;
            if self.debug_enabled
                && let Some(elapsed) = elapsed
            {
                self.debug_stats.layout_focus_repair_time += elapsed;
            }
        }

        if self.semantics_requested {
            let (_, elapsed) = fret_perf::measure_span(
                options.time_enabled,
                options.trace_enabled,
                || {
                    tracing::trace_span!(
                        "fret.ui.layout.refresh_semantics",
                        window = ?options.window,
                        frame_id = options.frame_id.0,
                        pass_kind = ?options.pass_kind,
                    )
                },
                || {
                    self.semantics_requested = false;
                    self.refresh_semantics_snapshot(app);
                },
            );
            timings.refresh_semantics = elapsed;
            if self.debug_enabled
                && let Some(elapsed) = elapsed
            {
                self.debug_stats.layout_semantics_refresh_time += elapsed;
            }
        }

        let (_, elapsed) = fret_perf::measure_span(
            options.time_enabled,
            options.trace_enabled,
            || {
                tracing::trace_span!(
                    "fret.ui.layout.flush_deferred_cleanup",
                    window = ?options.window,
                    frame_id = options.frame_id.0,
                    pass_kind = ?options.pass_kind,
                )
            },
            || self.flush_deferred_cleanup(services),
        );
        timings.flush_deferred_cleanup = elapsed;
        if self.debug_enabled
            && let Some(elapsed) = elapsed
        {
            self.debug_stats.layout_deferred_cleanup_time += elapsed;
        }

        timings
    }

    fn emit_final_layout_profiles_if_needed(&mut self, app: &mut H, pass_kind: LayoutPassKind) {
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
            for node in snapshot.nodes.iter() {
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
            for node in snapshot.nodes.iter() {
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
        let Some(canonical) =
            self.resolve_live_attached_node_for_element(app, Some(window), element)
        else {
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
            self.request_post_layout_window_runtime_snapshot_refine();
        }

        let Some(focused) = self.focus() else {
            return;
        };
        let Some(node) = self.nodes.get(focused) else {
            return;
        };
        if node.bounds.size.width.0 <= 0.0 || node.bounds.size.height.0 <= 0.0 {
            #[cfg(debug_assertions)]
            if crate::runtime_config::ui_runtime_config().debug_focus_repair {
                eprintln!(
                    "focus_repair: clearing focus={focused:?} due to empty bounds={:?}",
                    node.bounds
                );
            }
            self.set_focus(None);
            self.request_post_layout_window_runtime_snapshot_refine();
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

        // Barrier relayouts can update descendant layout without invalidating ancestors. That
        // means scroll containers that rely on cached content extents can "pin" their scroll
        // ranges to the previous frame if a contained barrier expands near the bottom of a scroll
        // view.
        //
        // To keep scroll extents consistent, allow barrier relayouts to schedule a follow-up
        // relayout for the nearest scrollable ancestor.
        const MAX_PASSES: usize = 4;
        let mut passes: usize = 0;
        let mut scheduled_followups: HashSet<NodeId> = HashSet::new();

        while passes < MAX_PASSES {
            passes = passes.saturating_add(1);

            let pending = self.take_pending_barrier_relayouts();
            if pending.is_empty() {
                break;
            }

            let mut unique = HashSet::<NodeId>::with_capacity(pending.len());
            let mut targets: Vec<NodeId> = Vec::with_capacity(pending.len());
            for node in pending {
                if unique.insert(node) {
                    targets.push(node);
                }
            }

            let mut roots_with_bounds: Vec<(NodeId, Rect)> = Vec::with_capacity(targets.len());
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

                roots_with_bounds.push((root, bounds));
            }

            // Pending barrier relayouts run as contained solves. Pre-solve each root via the
            // layout engine to avoid widget-local fallback solves (which amplify tail latency by
            // triggering extra out-of-band engine passes).
            self.solve_barrier_flow_roots_if_needed(
                app,
                services,
                &roots_with_bounds,
                scale_factor,
            );

            for (root, bounds) in roots_with_bounds {
                let _ = self.layout_in_with_pass_kind(
                    app,
                    services,
                    root,
                    bounds,
                    scale_factor,
                    pass_kind,
                    crate::layout::overflow::LayoutOverflowContext::default(),
                );
                if self.debug_enabled {
                    self.debug_stats.barrier_relayouts_performed = self
                        .debug_stats
                        .barrier_relayouts_performed
                        .saturating_add(1);
                }

                // After contained relayout, schedule a follow-up barrier relayout for the nearest
                // scrollable ancestor so it can recompute scroll extents against the new subtree
                // bounds without forcing a full ancestor relayout.
                let mut current = self.nodes.get(root).and_then(|n| n.parent);
                while let Some(id) = current {
                    let can_scroll = self
                        .nodes
                        .get(id)
                        .and_then(|n| n.widget.as_ref())
                        .is_some_and(|w| w.can_scroll_descendant_into_view());
                    if can_scroll {
                        if scheduled_followups.insert(id) {
                            self.schedule_barrier_relayout_with_source_and_detail(
                                id,
                                UiDebugInvalidationSource::Other,
                                UiDebugInvalidationDetail::BarrierFollowupRelayout,
                            );
                        }
                        break;
                    }
                    current = self.nodes.get(id).and_then(|n| n.parent);
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
        self.update_interactive_resize_state_for_layout(app.frame_id(), bounds, scale_factor);
        let force_post_resize_rebuild = self.interactive_resize_requires_full_rebuild();
        if force_post_resize_rebuild {
            self.mark_subtree_invalidation_local_with_detail(
                root,
                Invalidation::Layout,
                UiDebugInvalidationDetail::InteractiveResizeFullRebuild,
            );
        }

        if self.invalidated_layout_nodes == 0
            && self.invalidated_hit_test_nodes == 0
            && let Some(n) = self.nodes.get(root)
            && !n.invalidation.layout
            && !n.invalidation.hit_test
            && n.bounds == bounds
            && n.measured_size != Size::default()
            && !force_post_resize_rebuild
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
            crate::layout::overflow::LayoutOverflowContext::default(),
        );
        self.flush_viewport_roots_after_root(
            app,
            services,
            scale_factor,
            LayoutPassKind::Final,
            &mut viewport_cursor,
        );

        self.finish_final_layout_frame(app);
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
        self.update_interactive_resize_state_for_layout(app.frame_id(), bounds, scale_factor);
        let force_post_resize_rebuild = self.interactive_resize_requires_full_rebuild();
        if force_post_resize_rebuild {
            self.mark_subtree_invalidation_local_with_detail(
                root,
                Invalidation::Layout,
                UiDebugInvalidationDetail::InteractiveResizeFullRebuild,
            );
        }
        if self.invalidated_layout_nodes == 0
            && self.invalidated_hit_test_nodes == 0
            && let Some(n) = self.nodes.get(root)
            && !n.invalidation.layout
            && !n.invalidation.hit_test
            && n.bounds == bounds
            && n.measured_size != Size::default()
            && !force_post_resize_rebuild
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
            crate::layout::overflow::LayoutOverflowContext::default(),
        );
        self.flush_viewport_roots_after_root(
            app,
            services,
            scale_factor,
            LayoutPassKind::Final,
            &mut viewport_cursor,
        );
        self.finish_final_layout_frame(app);
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
        overflow_ctx: crate::layout::overflow::LayoutOverflowContext,
    ) -> Size {
        self.layout_node(
            app,
            services,
            root,
            bounds,
            scale_factor,
            pass_kind,
            overflow_ctx,
        )
    }

    pub(crate) fn begin_scroll_layout_kind_profile(&mut self) {
        self.scroll_layout_kind_profile_stack
            .push(ScrollLayoutKindProfileScope::default());
    }

    pub(crate) fn end_scroll_layout_kind_profile(&mut self) -> Vec<UiDebugScrollLayoutKindProfile> {
        let profiles = self
            .scroll_layout_kind_profile_stack
            .pop()
            .map(ScrollLayoutKindProfileScope::into_debug_profiles)
            .unwrap_or_default();
        if let Some(parent) = self.scroll_layout_kind_profile_stack.last_mut() {
            parent.absorb_debug_profiles(&profiles);
        }
        profiles
    }

    fn sync_element_bounds_cache_after_layout(&mut self, app: &mut H) {
        let Some(window) = self.window else {
            return;
        };

        let mut element_nodes = std::mem::take(&mut self.scratch_element_nodes);
        let mut bounds_records = std::mem::take(&mut self.scratch_bounds_records);
        let mut element_root_bounds_records =
            std::mem::take(&mut self.scratch_element_root_bounds_records);

        crate::elements::with_window_state(app, window, |st| {
            st.element_nodes_copy_into(&mut element_nodes);
        });

        bounds_records.clear();
        element_root_bounds_records.clear();
        for &(element, node) in element_nodes.iter() {
            if let Some(rect) = self.nodes.get(node).map(|n| n.bounds) {
                bounds_records.push((element, rect));
            }
            if let Some(root_bounds) = self.viewport_root_bounds_for_node(node) {
                element_root_bounds_records.push((element, root_bounds));
            }
        }

        crate::elements::with_window_state(app, window, |st| {
            for (element, rect) in bounds_records.iter().copied() {
                st.record_bounds(element, rect);
            }
            st.replace_element_root_bounds(element_root_bounds_records.iter().copied());
        });

        element_nodes.clear();
        bounds_records.clear();
        element_root_bounds_records.clear();
        self.scratch_element_nodes = element_nodes;
        self.scratch_bounds_records = bounds_records;
        self.scratch_element_root_bounds_records = element_root_bounds_records;
    }

    fn viewport_root_bounds_for_node(&self, mut node: NodeId) -> Option<Rect> {
        loop {
            if let Some((_, bounds)) = self
                .viewport_roots()
                .iter()
                .rev()
                .find(|(root, _)| *root == node)
            {
                return Some(*bounds);
            }

            let parent = self.nodes.get(node).and_then(|n| n.parent)?;
            node = parent;
        }
    }

    fn finish_final_layout_frame(&mut self, app: &mut H) {
        self.layout_engine.end_frame();
        if let Some(window) = self.window {
            let frame_id = app.frame_id();
            crate::elements::with_window_state(app, window, |st| {
                st.clear_stale_interaction_targets_for_frame(frame_id);
                st.sync_active_text_selection_node(|element, seeded| {
                    self.resolve_live_attached_node_for_element_seeded(element, seeded)
                });
                st.sync_interaction_target_nodes(|element, seeded| {
                    self.resolve_live_attached_node_for_element_seeded(element, seeded)
                });
            });
        }

        // Keep cross-frame `bounds_for_element(...)` queries in sync with the latest layout.
        // These bounds are used by component-layer policies (e.g. overlay placement) and are
        // expected to reflect the most recent layout pass.
        self.sync_element_bounds_cache_after_layout(app);
        self.validate_subtree_layout_dirty_counts_if_enabled();
        if !self.interactive_resize_active() {
            self.interactive_resize_needs_full_rebuild = false;
        }
        self.last_layout_frame_id = Some(app.frame_id());
        self.refine_pending_window_runtime_snapshots_after_layout(app);
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

    pub(crate) fn node_is_attached_to_layer_tree(&self, node: NodeId) -> bool {
        self.node_root(node)
            .is_some_and(|root| self.root_to_layer.contains_key(&root))
    }

    fn any_attached_layout_invalidations(&self) -> bool {
        let mut stack: Vec<NodeId> = self
            .visible_layers_in_paint_order()
            .map(|layer| self.layers[layer].root)
            .collect();
        while let Some(id) = stack.pop() {
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            if node.invalidation.layout {
                return true;
            }
            if node.layout_dirty_children_suppressed {
                continue;
            }
            stack.extend(node.children.iter().copied());
        }
        false
    }

    fn prune_detached_layout_followups(&mut self) {
        let retained_dirty_boundaries: std::collections::HashSet<NodeId> = self
            .dirty_boundaries
            .iter()
            .copied()
            .filter(|&root| self.node_is_attached_to_layer_tree(root))
            .collect();
        let detached: Vec<NodeId> = self
            .dirty_boundaries
            .difference(&retained_dirty_boundaries)
            .copied()
            .collect();
        for root in detached {
            self.clear_boundary_layout_dirty(root);
        }
        self.dirty_boundaries = retained_dirty_boundaries;
        self.pending_barrier_relayouts = self
            .pending_barrier_relayouts
            .iter()
            .copied()
            .filter(|&root| self.node_is_attached_to_layer_tree(root))
            .collect();
    }

    fn begin_layout_engine_frame(&mut self, app: &mut H) {
        self.layout_engine.begin_frame(app.frame_id());
        self.viewport_roots.clear();
    }

    pub(super) fn mark_layout_engine_seen_subtree_from_ui_children(
        &mut self,
        engine: &mut crate::layout_engine::TaffyLayoutEngine,
        root: NodeId,
    ) -> u32 {
        if engine.layout_id_for_node(root).is_none() {
            return 0;
        }

        let mut marked = 0u32;
        self.scratch_node_stack.clear();
        self.scratch_node_stack.push(root);
        while let Some(node) = self.scratch_node_stack.pop() {
            if engine.layout_id_for_node(node).is_some() {
                marked = marked.saturating_add(1);
            }
            engine.mark_seen_if_present(node);
            if let Some(entry) = self.nodes.get(node) {
                if entry.layout_dirty_children_suppressed {
                    continue;
                }
                for &child in &entry.children {
                    self.scratch_node_stack.push(child);
                }
            }
        }
        marked
    }

    #[allow(clippy::too_many_arguments)]
    fn debug_record_layout_request_build_root_if_enabled(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        root_kind: &'static str,
        root: NodeId,
        started: Option<Instant>,
        mode: &'static str,
        had_layout_engine_node: bool,
        layout_invalidated: bool,
        subtree_layout_dirty: bool,
        subtree_layout_dirty_count: u32,
        needs_layout: bool,
        is_translation_only: bool,
        nodes_marked_seen: u32,
    ) {
        if !self.debug_enabled {
            return;
        }
        let Some(started) = started else {
            return;
        };
        let (root_element, root_element_kind, root_element_path) =
            self.debug_resolve_layout_solve_root_label(app, window, root);
        let dirty_descendants =
            self.debug_collect_layout_dirty_descendant_samples(app, window, root, 4);
        self.debug_record_layout_request_build_root(super::UiDebugLayoutRequestBuildRoot {
            root,
            root_kind,
            root_element,
            root_element_kind,
            root_element_path,
            elapsed: started.elapsed(),
            mode,
            had_layout_engine_node,
            layout_invalidated,
            subtree_layout_dirty,
            subtree_layout_dirty_count,
            descendant_layout_dirty_count: subtree_layout_dirty_count
                .saturating_sub(layout_invalidated as u32),
            needs_layout,
            is_translation_only,
            nodes_marked_seen,
            dirty_descendants,
        });
    }

    fn debug_collect_layout_dirty_descendant_samples(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        root: NodeId,
        max_samples: usize,
    ) -> Vec<super::UiDebugLayoutDirtyDescendant> {
        if !self.debug_enabled || max_samples == 0 {
            return Vec::new();
        }

        let mut stack: Vec<NodeId> = Vec::new();
        if let Some(entry) = self.nodes.get(root) {
            for &child in entry.children.iter().rev() {
                if self.node_subtree_layout_dirty_count(child) > 0 {
                    stack.push(child);
                }
            }
        }

        let mut samples = Vec::new();
        while let Some(node) = stack.pop() {
            let (layout_invalidated, subtree_layout_dirty_count) = {
                let Some(entry) = self.nodes.get(node) else {
                    continue;
                };
                if entry.subtree_layout_dirty_count == 0 {
                    continue;
                }
                for &child in entry.children.iter().rev() {
                    if self.node_subtree_layout_dirty_count(child) > 0 {
                        stack.push(child);
                    }
                }
                (entry.invalidation.layout, entry.subtree_layout_dirty_count)
            };

            if !layout_invalidated {
                continue;
            }

            let (source_root, source, detail) =
                self.debug_layout_invalidation_origin_for_node(node);
            let (element, element_kind, element_path) =
                self.debug_resolve_layout_solve_root_label(app, window, node);
            samples.push(super::UiDebugLayoutDirtyDescendant {
                node,
                element,
                element_kind,
                element_path,
                subtree_layout_dirty_count,
                source_root,
                source,
                detail,
            });
            if samples.len() >= max_samples {
                break;
            }
        }
        samples
    }

    fn debug_layout_invalidation_origin_for_node(
        &self,
        node: NodeId,
    ) -> (
        Option<NodeId>,
        Option<super::UiDebugInvalidationSource>,
        Option<super::UiDebugInvalidationDetail>,
    ) {
        if let Some(source) = self.debug_layout_dirty_sources.get(&node) {
            return (
                Some(source.source_root),
                Some(source.source),
                Some(source.detail),
            );
        }
        for walk in self.debug_invalidation_walks.iter().rev() {
            if !matches!(walk.inv, Invalidation::Layout | Invalidation::HitTest) {
                continue;
            }
            if self.debug_node_is_descendant_or_self(walk.root, node) {
                return (Some(walk.root), Some(walk.source), Some(walk.detail));
            }
        }
        (None, None, None)
    }

    fn debug_node_is_descendant_or_self(&self, node: NodeId, ancestor: NodeId) -> bool {
        let mut current = Some(node);
        let mut remaining = self.nodes.len().saturating_add(1);
        while let Some(id) = current {
            if remaining == 0 {
                return false;
            }
            remaining = remaining.saturating_sub(1);
            if id == ancestor {
                return true;
            }
            current = self.nodes.get(id).and_then(|n| n.parent);
        }
        false
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
        // Hot path: avoid scanning the whole node store. Boundary invalidations are tracked in
        // `dirty_boundaries`, so we can restrict this pass to the subset that actually changed.
        let mut candidates: Vec<NodeId> = Vec::with_capacity(16);
        for &id in &self.dirty_boundaries {
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            if !node.view_cache.enabled || !node.view_cache.layout_contained_when_bounds_known() {
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
        let mut scheduled_followups: std::collections::HashSet<NodeId> =
            std::collections::HashSet::new();

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

        // Contained cache-root relayouts run as independent solves after the main viewport roots.
        // Pre-solve via the layout engine so cache-root subtrees don't trigger widget-local
        // fallback solves (which create extra solves and jitter within the same frame).
        self.solve_barrier_flow_roots_if_needed(app, services, &targets, scale_factor);

        for (root, bounds) in targets {
            if self.debug_enabled {
                self.debug_stats.view_cache_contained_relayouts = self
                    .debug_stats
                    .view_cache_contained_relayouts
                    .saturating_add(1);
                self.debug_view_cache_contained_relayout_roots.push(root);
            }
            let _ = self.layout_in_with_pass_kind(
                app,
                services,
                root,
                bounds,
                scale_factor,
                pass_kind,
                crate::layout::overflow::LayoutOverflowContext::default(),
            );
            self.flush_viewport_roots_after_root(
                app,
                services,
                scale_factor,
                pass_kind,
                viewport_cursor,
            );
            let layout_transition = self.nodes.get_mut(root).map(|node| {
                let prev = node.invalidation;
                let layout_before = node.invalidation.layout;
                node.invalidation.layout = false;
                let next = node.invalidation;
                let layout_after = node.invalidation.layout;
                (prev, next, layout_before, layout_after)
            });
            if let Some((prev, next, layout_before, layout_after)) = layout_transition
                && layout_before != layout_after
            {
                record_layout_invalidation_transition(
                    &mut self.layout_invalidations_count,
                    layout_before,
                    layout_after,
                );
                self.note_layout_invalidation_transition_for_subtree_aggregation(
                    root,
                    layout_before,
                    layout_after,
                );
                if layout_before && !layout_after {
                    self.debug_clear_layout_dirty_source(root);
                }
                self.update_invalidation_counters(prev, next);
            }
            // Contained relayout is a layout-only repair path. It may consume a layout-invalidated
            // cache root without implying that the declarative subtree must rerun next frame.
            // Keep an explicit `needs_rerender` bit authoritative, and clear the scheduling-only
            // dirty marker once both layout invalidation and rerender pressure are gone.
            self.clear_boundary_dirty_tracking_if_clean(root);

            // Contained view-cache relayouts run after the main root layout pass, so any scroll
            // ancestor that inferred its content extent earlier in the frame can be left with a
            // stale range. Re-run the nearest scrollable ancestor in the same frame so scroll
            // extents track the reconciled cache-root bounds immediately.
            let mut current = self.nodes.get(root).and_then(|n| n.parent);
            while let Some(id) = current {
                let can_scroll = self
                    .nodes
                    .get(id)
                    .and_then(|n| n.widget.as_ref())
                    .is_some_and(|w| w.can_scroll_descendant_into_view());
                if can_scroll {
                    if scheduled_followups.insert(id) {
                        self.schedule_barrier_relayout_with_source_and_detail(
                            id,
                            UiDebugInvalidationSource::Other,
                            UiDebugInvalidationDetail::BarrierFollowupRelayout,
                        );
                    }
                    break;
                }
                current = self.nodes.get(id).and_then(|n| n.parent);
            }
        }

        if !scheduled_followups.is_empty() {
            self.layout_pending_barrier_relayouts_if_needed(
                app,
                services,
                scale_factor,
                pass_kind,
                viewport_cursor,
            );
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
        let force_post_resize_rebuild = self.interactive_resize_requires_full_rebuild();
        if force_post_resize_rebuild {
            for &root in roots {
                self.mark_subtree_invalidation_local_with_detail(
                    root,
                    Invalidation::Layout,
                    UiDebugInvalidationDetail::InteractiveResizeFullRebuild,
                );
            }
        }
        // Phase 1: request/build for stable identity, even if we later skip compute/apply.
        for &root in roots {
            let root_started = self.debug_enabled.then(Instant::now);
            let Some((
                has_element,
                layout_invalidated,
                subtree_layout_dirty,
                subtree_layout_dirty_count,
                prev_bounds,
                measured,
            )) = self.nodes.get(root).map(|node| {
                let subtree_layout_dirty_count = self.node_subtree_layout_dirty_count(root);
                (
                    node.element.is_some(),
                    node.invalidation.layout,
                    subtree_layout_dirty_count > 0,
                    subtree_layout_dirty_count,
                    node.bounds,
                    node.measured_size,
                )
            })
            else {
                continue;
            };
            let had_layout_engine_node = engine.layout_id_for_node(root).is_some();
            let needs_layout = layout_invalidated || prev_bounds != bounds;
            let is_translation_only = !layout_invalidated
                && prev_bounds.size == bounds.size
                && prev_bounds.origin != bounds.origin
                && measured != Size::default();

            if !has_element {
                self.debug_record_layout_request_build_root_if_enabled(
                    app,
                    window,
                    "window",
                    root,
                    root_started,
                    "skip_no_element",
                    had_layout_engine_node,
                    layout_invalidated,
                    subtree_layout_dirty,
                    subtree_layout_dirty_count,
                    needs_layout,
                    is_translation_only,
                    0,
                );
                continue;
            }

            if had_layout_engine_node && (!needs_layout || is_translation_only) {
                let nodes_marked_seen =
                    self.mark_layout_engine_seen_subtree_from_ui_children(&mut engine, root);
                self.debug_record_layout_request_build_root_if_enabled(
                    app,
                    window,
                    "window",
                    root,
                    root_started,
                    "mark_seen",
                    had_layout_engine_node,
                    layout_invalidated,
                    subtree_layout_dirty,
                    subtree_layout_dirty_count,
                    needs_layout,
                    is_translation_only,
                    nodes_marked_seen,
                );
                continue;
            }
            if reuse_cached_flow
                && had_layout_engine_node
                && !layout_invalidated
                && !subtree_layout_dirty
            {
                engine.set_viewport_root_override_size(root, bounds.size, sf);
                self.note_interactive_resize_cached_flow_reuse();
                let nodes_marked_seen =
                    self.mark_layout_engine_seen_subtree_from_ui_children(&mut engine, root);
                self.debug_record_layout_request_build_root_if_enabled(
                    app,
                    window,
                    "window",
                    root,
                    root_started,
                    "cached_flow_reuse",
                    had_layout_engine_node,
                    layout_invalidated,
                    subtree_layout_dirty,
                    subtree_layout_dirty_count,
                    needs_layout,
                    is_translation_only,
                    nodes_marked_seen,
                );
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
                self.debug_record_layout_request_build_root_if_enabled(
                    app,
                    window,
                    "window",
                    root,
                    root_started,
                    "build_flow",
                    had_layout_engine_node,
                    layout_invalidated,
                    subtree_layout_dirty,
                    subtree_layout_dirty_count,
                    needs_layout,
                    is_translation_only,
                    0,
                );
            }
        }
        let phase1_elapsed = phase1_started.map(|s| s.elapsed());

        let phase2_started = profile_layout.then(Instant::now);
        // Phase 2: compute/apply only when layout is needed.
        //
        // When multiple independent viewport roots need layout in the same frame (window root +
        // overlays + other detached flow roots), solving them one-by-one can amplify fixed per-solve
        // overhead into tail spikes. Prefer batching via the layout engine's synthetic-root path.
        let mut pending_solves: Vec<(NodeId, LayoutSize<AvailableSpace>)> = Vec::new();
        for &root in roots {
            let (has_element, needs_layout, is_translation_only, prev_bounds, layout_invalidated) =
                match self.nodes.get(root) {
                    Some(node) => {
                        let has_element = node.element.is_some();
                        let needs_layout = node.invalidation.layout || node.bounds != bounds;
                        let is_translation_only = !node.invalidation.layout
                            && node.bounds.size == bounds.size
                            && node.bounds.origin != bounds.origin
                            && node.measured_size != Size::default();
                        (
                            has_element,
                            needs_layout,
                            is_translation_only,
                            node.bounds,
                            node.invalidation.layout,
                        )
                    }
                    None => continue,
                };

            if !has_element || !needs_layout || is_translation_only {
                continue;
            }
            if !layout_invalidated
                && engine.layout_id_for_node(root).is_some()
                && self.can_skip_clean_geometry_engine_solve_for_resize(
                    app,
                    root,
                    bounds,
                    prev_bounds,
                    sf,
                )
            {
                continue;
            }

            pending_solves.push((root, available));
        }

        if !pending_solves.is_empty() {
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

            for &(root, _available) in &pending_solves {
                self.maybe_dump_taffy_subtree(app, window, &engine, root, bounds, sf);
            }
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
            let force_post_resize_rebuild = pass_kind == LayoutPassKind::Final
                && self.interactive_resize_requires_full_rebuild();

            if force_post_resize_rebuild {
                let roots_to_invalidate: Vec<NodeId> = self.viewport_roots[batch_start..batch_end]
                    .iter()
                    .map(|(root, _)| *root)
                    .collect();
                for root in roots_to_invalidate {
                    self.mark_subtree_invalidation_local_with_detail(
                        root,
                        Invalidation::Layout,
                        UiDebugInvalidationDetail::InteractiveResizeFullRebuild,
                    );
                }
            }

            struct ViewportWorkItem {
                root: NodeId,
                bounds: Rect,
                prev_bounds: Rect,
                has_element: bool,
                needs_layout: bool,
                is_translation_only: bool,
                layout_invalidated: bool,
                subtree_layout_dirty: bool,
                subtree_layout_dirty_count: u32,
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
                let has_element = self
                    .nodes
                    .get(root)
                    .is_some_and(|node| node.element.is_some());

                let needs_layout = invalidated || prev_bounds != bounds;
                let is_translation_only = !invalidated
                    && prev_bounds.size == bounds.size
                    && prev_bounds.origin != bounds.origin
                    && measured != Size::default();
                let subtree_layout_dirty_count = self.node_subtree_layout_dirty_count(root);

                batch.push(ViewportWorkItem {
                    root,
                    bounds,
                    prev_bounds,
                    has_element,
                    needs_layout,
                    is_translation_only,
                    layout_invalidated: invalidated,
                    subtree_layout_dirty: subtree_layout_dirty_count > 0,
                    subtree_layout_dirty_count,
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
                    let root_started = self.debug_enabled.then(Instant::now);
                    let had_layout_engine_node = engine.layout_id_for_node(item.root).is_some();
                    if !item.has_element {
                        self.debug_record_layout_request_build_root_if_enabled(
                            app,
                            window,
                            "viewport",
                            item.root,
                            root_started,
                            "skip_no_element",
                            had_layout_engine_node,
                            item.layout_invalidated,
                            item.subtree_layout_dirty,
                            item.subtree_layout_dirty_count,
                            item.needs_layout,
                            item.is_translation_only,
                            0,
                        );
                        continue;
                    }
                    if had_layout_engine_node && (!item.needs_layout || item.is_translation_only) {
                        let nodes_marked_seen = self
                            .mark_layout_engine_seen_subtree_from_ui_children(
                                &mut engine,
                                item.root,
                            );
                        self.debug_record_layout_request_build_root_if_enabled(
                            app,
                            window,
                            "viewport",
                            item.root,
                            root_started,
                            "mark_seen",
                            had_layout_engine_node,
                            item.layout_invalidated,
                            item.subtree_layout_dirty,
                            item.subtree_layout_dirty_count,
                            item.needs_layout,
                            item.is_translation_only,
                            nodes_marked_seen,
                        );
                        continue;
                    }
                    if reuse_cached_flow
                        && had_layout_engine_node
                        && !item.layout_invalidated
                        && !item.subtree_layout_dirty
                    {
                        engine.set_viewport_root_override_size(item.root, item.bounds.size, sf);
                        self.note_interactive_resize_cached_flow_reuse();
                        let nodes_marked_seen = self
                            .mark_layout_engine_seen_subtree_from_ui_children(
                                &mut engine,
                                item.root,
                            );
                        self.debug_record_layout_request_build_root_if_enabled(
                            app,
                            window,
                            "viewport",
                            item.root,
                            root_started,
                            "cached_flow_reuse",
                            had_layout_engine_node,
                            item.layout_invalidated,
                            item.subtree_layout_dirty,
                            item.subtree_layout_dirty_count,
                            item.needs_layout,
                            item.is_translation_only,
                            nodes_marked_seen,
                        );
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
                        self.debug_record_layout_request_build_root_if_enabled(
                            app,
                            window,
                            "viewport",
                            item.root,
                            root_started,
                            "build_flow",
                            had_layout_engine_node,
                            item.layout_invalidated,
                            item.subtree_layout_dirty,
                            item.subtree_layout_dirty_count,
                            item.needs_layout,
                            item.is_translation_only,
                            0,
                        );
                    }
                }

                // Phase 2: compute/apply only for roots that need layout and are not translation-only.
                let mut pending_solves: Vec<(NodeId, LayoutSize<AvailableSpace>)> = Vec::new();
                for item in &batch {
                    if !item.needs_layout || item.is_translation_only {
                        continue;
                    }
                    if !item.layout_invalidated
                        && engine.layout_id_for_node(item.root).is_some()
                        && self.can_skip_clean_geometry_engine_solve_for_resize(
                            app,
                            item.root,
                            item.bounds,
                            item.prev_bounds,
                            sf,
                        )
                    {
                        continue;
                    }
                    pending_solves.push((
                        item.root,
                        LayoutSize::new(
                            AvailableSpace::Definite(item.bounds.size.width),
                            AvailableSpace::Definite(item.bounds.size.height),
                        ),
                    ));
                }

                if !pending_solves.is_empty() {
                    let solves_before = engine.solve_count();
                    let solve_time_before = engine.last_solve_time();
                    engine.compute_independent_roots_with_measure_if_needed(
                        &pending_solves,
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

                    for item in &batch {
                        if !item.needs_layout || item.is_translation_only {
                            continue;
                        }
                        self.maybe_dump_taffy_subtree(
                            app,
                            window,
                            &engine,
                            item.root,
                            item.bounds,
                            sf,
                        );
                    }
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
                    crate::layout::overflow::LayoutOverflowContext::default(),
                );
            }

            *viewport_cursor = batch_end;
        }
    }
}
