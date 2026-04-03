use super::super::*;

impl<H: UiHost> UiTree<H> {
    #[stacksafe::stacksafe]
    pub fn paint_all(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        bounds: Rect,
        scene: &mut Scene,
        scale_factor: f32,
    ) {
        let trace_paint = tracing::enabled!(tracing::Level::TRACE);
        let window = self.window;
        let frame_id = app.frame_id();

        // `paint_node` can run multiple times within the same runner `FrameId` in tests and
        // diagnostics scenarios. Use a tree-local paint pass token to avoid conflating distinct
        // invocations when tracking "bounds were already updated this pass" markers.
        let paint_pass = self.paint_pass.saturating_add(1);
        self.paint_pass = paint_pass;

        let ((), paint_elapsed) = fret_perf::measure_span(
            self.debug_enabled,
            trace_paint,
            || {
                tracing::trace_span!(
                    "fret.ui.paint_all",
                    window = ?window,
                    frame_id = frame_id.0,
                    paint_pass,
                    scale_factor,
                )
            },
            || {
                if let Some(window) = self.window {
                    let frame_id = app.frame_id();
                    app.with_global_mut_untracked(
                        fret_core::WindowFrameClockService::default,
                        |svc, _host| svc.record_frame(window, frame_id),
                    );
                }
                if self.debug_enabled {
                    self.begin_debug_frame_if_needed(app.frame_id());
                    self.debug_stats.frame_id = app.frame_id();
                    self.debug_stats.paint_nodes = 0;
                    self.debug_stats.paint_nodes_performed = 0;
                    self.debug_stats.paint_cache_hits = 0;
                    self.debug_stats.paint_cache_misses = 0;
                    self.debug_stats.paint_cache_replayed_ops = 0;
                    self.debug_stats.paint_cache_hit_test_only_replay_allowed = 0;
                    self.debug_stats
                        .paint_cache_hit_test_only_replay_rejected_key_mismatch = 0;
                    self.debug_stats.paint_cache_replay_time = Duration::default();
                    self.debug_stats.paint_cache_bounds_translate_time = Duration::default();
                    self.debug_stats.paint_cache_bounds_translated_nodes = 0;
                    self.debug_stats.paint_record_visual_bounds_time = Duration::default();
                    self.debug_stats.paint_record_visual_bounds_calls = 0;
                    self.debug_stats.paint_cache_key_time = Duration::default();
                    self.debug_stats.paint_cache_hit_check_time = Duration::default();
                    self.debug_stats.paint_widget_time = Duration::default();
                    self.debug_stats.paint_observation_record_time = Duration::default();
                    self.debug_stats.paint_host_widget_observed_models_time = Duration::default();
                    self.debug_stats.paint_host_widget_observed_models_items = 0;
                    self.debug_stats.paint_host_widget_observed_globals_time = Duration::default();
                    self.debug_stats.paint_host_widget_observed_globals_items = 0;
                    self.debug_stats.paint_host_widget_instance_lookup_time = Duration::default();
                    self.debug_stats.paint_host_widget_instance_lookup_calls = 0;
                    self.debug_stats.paint_text_prepare_time = Duration::default();
                    self.debug_stats.paint_text_prepare_calls = 0;
                    self.debug_stats.paint_text_prepare_reason_blob_missing = 0;
                    self.debug_stats.paint_text_prepare_reason_scale_changed = 0;
                    self.debug_stats.paint_text_prepare_reason_text_changed = 0;
                    self.debug_stats.paint_text_prepare_reason_rich_changed = 0;
                    self.debug_stats.paint_text_prepare_reason_style_changed = 0;
                    self.debug_stats.paint_text_prepare_reason_wrap_changed = 0;
                    self.debug_stats.paint_text_prepare_reason_overflow_changed = 0;
                    self.debug_stats.paint_text_prepare_reason_width_changed = 0;
                    self.debug_stats
                        .paint_text_prepare_reason_font_stack_changed = 0;
                    self.debug_paint_widget_exclusive_started = None;
                    self.debug_stats.paint_input_context_time = Duration::default();
                    self.debug_stats.paint_scroll_handle_invalidation_time = Duration::default();
                    self.debug_stats.paint_collect_roots_time = Duration::default();
                    self.debug_stats.paint_publish_text_input_snapshot_time = Duration::default();
                    self.debug_stats.paint_collapse_observations_time = Duration::default();
                    self.debug_stats.view_cache_active = self.view_cache_active();
                    self.debug_stats.focus = self.focus;
                    self.debug_stats.captured = self.captured_for(fret_core::PointerId(0));
                }

                // Keep IME enabled state in sync even if focus is set programmatically and no input event
                // has been dispatched yet (ADR 0012).
                let focus_is_text_input = self.focus_is_text_input(app);
                self.set_ime_allowed(app, focus_is_text_input);
                let input_ctx_started = self.debug_enabled.then(Instant::now);
                let (active_layers, barrier_root) = self.active_input_layers();
                let _ = active_layers;
                let input_ctx = self.current_window_input_context(
                    app,
                    barrier_root.is_some(),
                    focus_is_text_input,
                );
                self.publish_window_input_context_snapshot_untracked(app, &input_ctx, true);
                if let Some(input_ctx_started) = input_ctx_started {
                    self.debug_stats.paint_input_context_time = self
                        .debug_stats
                        .paint_input_context_time
                        .saturating_add(input_ctx_started.elapsed());
                }

                // Scroll offsets can change without triggering layout invalidations (e.g. wheel deltas that
                // only affect hit-testing/paint, or programmatic scroll handle updates in frames that skip
                // layout). Ensure we consume scroll-handle change invalidations before paint-cache replay
                // so cached ancestors cannot replay stale ops.
                let (_, scroll_inv_elapsed) = fret_perf::measure(self.debug_enabled, || {
                    self.invalidate_scroll_handle_bindings_for_changed_handles(
                        app,
                        crate::layout_pass::LayoutPassKind::Final,
                        false,
                        true,
                    );
                });
                if let Some(scroll_inv_elapsed) = scroll_inv_elapsed {
                    self.debug_stats.paint_scroll_handle_invalidation_time = self
                        .debug_stats
                        .paint_scroll_handle_invalidation_time
                        .saturating_add(scroll_inv_elapsed);
                }

                let cache_enabled = self.paint_cache_enabled();
                if cache_enabled {
                    self.paint_cache.begin_frame();
                } else {
                    self.paint_cache.invalidate_recording();
                }

                self.scratch_visual_bounds_records.clear();

                let (roots, roots_elapsed) = fret_perf::measure(self.debug_enabled, || {
                    self.visible_layers_in_paint_order()
                        .map(|layer| self.layers[layer].root)
                        .collect::<Vec<NodeId>>()
                });
                if let Some(roots_elapsed) = roots_elapsed {
                    self.debug_stats.paint_collect_roots_time = self
                        .debug_stats
                        .paint_collect_roots_time
                        .saturating_add(roots_elapsed);
                }
                for root in roots {
                    self.paint(app, services, root, bounds, scene, scale_factor);
                }

                if let Some(window) = self.window
                    && !self.scratch_visual_bounds_records.is_empty()
                {
                    let mut records = std::mem::take(&mut self.scratch_visual_bounds_records);
                    let (_, flush_elapsed) = fret_perf::measure(self.debug_enabled, || {
                        crate::elements::with_window_state(app, window, |st| {
                            for (element, visual) in records.drain(..) {
                                if st
                                    .current_bounds(element)
                                    .is_some_and(|bounds| bounds == visual)
                                {
                                    continue;
                                }
                                st.record_visual_bounds(element, visual);
                            }
                        });
                    });
                    self.scratch_visual_bounds_records = records;
                    if let Some(flush_elapsed) = flush_elapsed {
                        self.debug_stats.paint_record_visual_bounds_time = self
                            .debug_stats
                            .paint_record_visual_bounds_time
                            .saturating_add(flush_elapsed);
                    }
                }

                // Publish a platform-facing text-input snapshot after paint so text widgets can update
                // their IME cursor area in the same frame (ADR 0012).
                if let Some(window) = self.window {
                    let (_, text_snapshot_elapsed) = fret_perf::measure(self.debug_enabled, || {
                        let mut next = if focus_is_text_input {
                            if let Some(focus) = self.focus
                                && let Some(snapshot) =
                                    crate::declarative::frame::with_element_record_for_node(
                                        app,
                                        window,
                                        focus,
                                        |r| match &r.instance {
                                            crate::declarative::ElementInstance::TextInputRegion(
                                                props,
                                            ) => Some(
                                                super::super::ui_tree_text_input::text_input_region_platform_text_input_snapshot(props),
                                            ),
                                            _ => None,
                                        },
                                    )
                                    .flatten()
                            {
                                snapshot
                            } else {
                                self.focus
                                    .and_then(|focus| self.nodes.get(focus))
                                    .and_then(|n| n.widget.as_ref())
                                    .and_then(|w| w.platform_text_input_snapshot())
                                    .unwrap_or_else(|| fret_runtime::WindowTextInputSnapshot {
                                        focus_is_text_input: true,
                                        ..Default::default()
                                    })
                            }
                        } else {
                            fret_runtime::WindowTextInputSnapshot::default()
                        };
                        next.focus_is_text_input = focus_is_text_input;

                        let needs_update = app
                            .global::<fret_runtime::WindowTextInputSnapshotService>()
                            .and_then(|svc| svc.snapshot(window))
                            .is_none_or(|prev| prev != &next);
                        if needs_update {
                            app.with_global_mut(
                                fret_runtime::WindowTextInputSnapshotService::default,
                                |svc, _app| {
                                    svc.set_snapshot(window, next);
                                },
                            );
                        }
                    });
                    if let Some(text_snapshot_elapsed) = text_snapshot_elapsed {
                        self.debug_stats.paint_publish_text_input_snapshot_time = self
                            .debug_stats
                            .paint_publish_text_input_snapshot_time
                            .saturating_add(text_snapshot_elapsed);
                    }
                }

                if cache_enabled {
                    self.paint_cache.finish_frame();
                    if self.debug_enabled {
                        self.debug_stats.paint_cache_hits = self.paint_cache.hits;
                        self.debug_stats.paint_cache_misses = self.paint_cache.misses;
                        self.debug_stats.paint_cache_replayed_ops = self.paint_cache.replayed_ops;
                    }
                }

                let (_, collapse_elapsed) = fret_perf::measure(self.debug_enabled, || {
                    self.collapse_paint_observations_to_view_cache_roots_if_needed();
                });
                if let Some(collapse_elapsed) = collapse_elapsed {
                    self.debug_stats.paint_collapse_observations_time = self
                        .debug_stats
                        .paint_collapse_observations_time
                        .saturating_add(collapse_elapsed);
                }
            },
        );
        if let Some(paint_elapsed) = paint_elapsed {
            self.debug_stats.paint_time = paint_elapsed;
        }
    }

    pub fn paint(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        bounds: Rect,
        scene: &mut Scene,
        scale_factor: f32,
    ) {
        self.paint_node(
            app,
            services,
            root,
            bounds,
            scene,
            scale_factor,
            crate::tree::paint_style::PaintStyleState::default(),
            Transform2D::IDENTITY,
        );
    }

    #[cfg(test)]
    pub(crate) fn test_set_paint_cache_allow_hit_test_only_override(value: Option<bool>) {
        super::set_paint_cache_allow_hit_test_only_for_test(value);
    }
}
