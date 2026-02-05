use super::*;
use std::any::TypeId;
use std::sync::OnceLock;

fn paint_cache_relax_view_cache_gating() -> bool {
    static RELAX: OnceLock<bool> = OnceLock::new();
    *RELAX.get_or_init(|| {
        std::env::var_os("FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING")
            .is_some_and(|v| !v.is_empty())
    })
}

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
        let started = self.debug_enabled.then(Instant::now);
        if self.debug_enabled {
            self.begin_debug_frame_if_needed(app.frame_id());
            self.debug_stats.frame_id = app.frame_id();
            self.debug_stats.paint_nodes = 0;
            self.debug_stats.paint_nodes_performed = 0;
            self.debug_stats.paint_cache_hits = 0;
            self.debug_stats.paint_cache_misses = 0;
            self.debug_stats.paint_cache_replayed_ops = 0;
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
        let focus_is_text_input = self.focus_is_text_input();
        self.set_ime_allowed(app, focus_is_text_input);
        let input_ctx_started = self.debug_enabled.then(Instant::now);
        let (active_layers, barrier_root) = self.active_input_layers();
        let _ = active_layers;
        if let Some(window) = self.window {
            let caps = app
                .global::<PlatformCapabilities>()
                .cloned()
                .unwrap_or_default();
            let window_arbitration = self.window_input_arbitration_snapshot();
            let mut input_ctx = InputContext {
                platform: Platform::current(),
                caps,
                ui_has_modal: barrier_root.is_some(),
                window_arbitration: Some(window_arbitration),
                focus_is_text_input,
                text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
                edit_can_undo: app
                    .global::<fret_runtime::WindowCommandAvailabilityService>()
                    .and_then(|svc| svc.snapshot(window))
                    .map(|s| s.edit_can_undo)
                    .unwrap_or(true),
                edit_can_redo: app
                    .global::<fret_runtime::WindowCommandAvailabilityService>()
                    .and_then(|svc| svc.snapshot(window))
                    .map(|s| s.edit_can_redo)
                    .unwrap_or(true),
                dispatch_phase: InputDispatchPhase::Bubble,
            };
            if let Some(mode) = app
                .global::<fret_runtime::WindowTextBoundaryModeService>()
                .and_then(|svc| svc.mode(window))
            {
                input_ctx.text_boundary_mode = mode;
            }
            if let Some(mode) = self.focus_text_boundary_mode_override() {
                input_ctx.text_boundary_mode = mode;
            }
            let needs_update = app
                .global::<fret_runtime::WindowInputContextService>()
                .and_then(|svc| svc.snapshot(window))
                .is_none_or(|prev| prev != &input_ctx);
            if needs_update {
                app.with_global_mut(
                    fret_runtime::WindowInputContextService::default,
                    |svc, _app| {
                        svc.set_snapshot(window, input_ctx);
                    },
                );
            }
        }
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
        let scroll_inv_started = self.debug_enabled.then(Instant::now);
        self.invalidate_scroll_handle_bindings_for_changed_handles(
            app,
            crate::layout_pass::LayoutPassKind::Final,
        );
        if let Some(scroll_inv_started) = scroll_inv_started {
            self.debug_stats.paint_scroll_handle_invalidation_time = self
                .debug_stats
                .paint_scroll_handle_invalidation_time
                .saturating_add(scroll_inv_started.elapsed());
        }

        let cache_enabled = self.paint_cache_enabled();
        if cache_enabled {
            self.paint_cache.begin_frame();
        } else {
            self.paint_cache.invalidate_recording();
        }

        let roots_started = self.debug_enabled.then(Instant::now);
        let roots: Vec<NodeId> = self
            .visible_layers_in_paint_order()
            .map(|layer| self.layers[layer].root)
            .collect();
        if let Some(roots_started) = roots_started {
            self.debug_stats.paint_collect_roots_time = self
                .debug_stats
                .paint_collect_roots_time
                .saturating_add(roots_started.elapsed());
        }
        for root in roots {
            self.paint(app, services, root, bounds, scene, scale_factor);
        }

        // Publish a platform-facing text-input snapshot after paint so text widgets can update
        // their IME cursor area in the same frame (ADR 0012).
        if let Some(window) = self.window {
            let text_snapshot_started = self.debug_enabled.then(Instant::now);
            let mut next = if focus_is_text_input {
                self.focus
                    .and_then(|focus| self.nodes.get(focus))
                    .and_then(|n| n.widget.as_ref())
                    .and_then(|w| w.platform_text_input_snapshot())
                    .unwrap_or_else(|| fret_runtime::WindowTextInputSnapshot {
                        focus_is_text_input: true,
                        ..Default::default()
                    })
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
            if let Some(text_snapshot_started) = text_snapshot_started {
                self.debug_stats.paint_publish_text_input_snapshot_time = self
                    .debug_stats
                    .paint_publish_text_input_snapshot_time
                    .saturating_add(text_snapshot_started.elapsed());
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

        let collapse_started = self.debug_enabled.then(Instant::now);
        self.collapse_paint_observations_to_view_cache_roots_if_needed();
        if let Some(collapse_started) = collapse_started {
            self.debug_stats.paint_collapse_observations_time = self
                .debug_stats
                .paint_collapse_observations_time
                .saturating_add(collapse_started.elapsed());
        }

        if let Some(started) = started {
            self.debug_stats.paint_time = started.elapsed();
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
            Transform2D::IDENTITY,
        );
    }

    #[stacksafe::stacksafe]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn paint_node(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        node: NodeId,
        bounds: Rect,
        scene: &mut Scene,
        scale_factor: f32,
        accumulated_transform: Transform2D,
    ) {
        if self.debug_enabled {
            self.debug_stats.paint_nodes = self.debug_stats.paint_nodes.saturating_add(1);
        }

        if let Some(n) = self.nodes.get_mut(node) {
            n.bounds = bounds;
        }

        let local_transform = self.node_render_transform(node);
        let current_transform = match local_transform {
            Some(t) => accumulated_transform.compose(t),
            None => accumulated_transform,
        };
        let sf = scale_factor;

        let (invalidated, prev_cache) = match self.nodes.get(node) {
            Some(n) => (n.invalidation.paint, n.paint_cache),
            None => return,
        };

        let view_cache = self
            .nodes
            .get(node)
            .map(|n| n.view_cache)
            .unwrap_or_default();
        let span = if view_cache.enabled && tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace_span!(
                "ui.cache_root.paint",
                node = ?node,
                view_cache_active = self.view_cache_active(),
                contained_layout = view_cache.contained_layout,
                invalidated = invalidated,
                frame_id = app.frame_id().0,
            )
        } else {
            tracing::Span::none()
        };
        let _span_guard = span.enter();

        if let Some(window) = self.window
            && let Some(element) = self.nodes.get(node).and_then(|n| n.element)
        {
            let record_started = self.debug_enabled.then(Instant::now);
            let visual = rect_aabb_transformed(bounds, current_transform);
            crate::elements::record_visual_bounds_for_element(app, window, element, visual);
            if let Some(record_started) = record_started {
                self.debug_stats.paint_record_visual_bounds_time = self
                    .debug_stats
                    .paint_record_visual_bounds_time
                    .saturating_add(record_started.elapsed());
                self.debug_stats.paint_record_visual_bounds_calls = self
                    .debug_stats
                    .paint_record_visual_bounds_calls
                    .saturating_add(1);
            }
        }

        let key_started = self.debug_enabled.then(Instant::now);
        let theme_revision = Theme::global(&*app).revision();
        let children_render_transform = self.node_children_render_transform(node);
        let child_transform = children_render_transform.unwrap_or(Transform2D::IDENTITY);
        let key = PaintCacheKey::new(bounds, sf, theme_revision, child_transform);
        let relax_view_cache_gating = paint_cache_relax_view_cache_gating();
        let cache_enabled = self.paint_cache_enabled()
            && self.node_render_transform(node).is_none()
            && (!self.view_cache_active()
                || relax_view_cache_gating
                || self.nodes.get(node).is_some_and(|n| n.view_cache.enabled));
        if let Some(key_started) = key_started {
            self.debug_stats.paint_cache_key_time = self
                .debug_stats
                .paint_cache_key_time
                .saturating_add(key_started.elapsed());
        }

        if cache_enabled && !invalidated {
            let hit_check_started = self.debug_enabled.then(Instant::now);
            if let Some(prev) = prev_cache
                && prev.generation == self.paint_cache.source_generation
                && prev.key == key
            {
                let start = scene.ops_len();
                let range = prev.start as usize..prev.end as usize;
                if range.start <= range.end && range.end <= self.paint_cache.prev_ops.len() {
                    let delta = Point::new(
                        bounds.origin.x - prev.origin.x,
                        bounds.origin.y - prev.origin.y,
                    );
                    if let Some(hit_check_started) = hit_check_started {
                        self.debug_stats.paint_cache_hit_check_time = self
                            .debug_stats
                            .paint_cache_hit_check_time
                            .saturating_add(hit_check_started.elapsed());
                    }
                    let replay_span = if tracing::enabled!(tracing::Level::TRACE) {
                        tracing::trace_span!(
                            "fret.ui.paint_cache.replay",
                            node = ?node,
                            ops = tracing::field::Empty,
                            scale_factor = sf,
                        )
                    } else {
                        tracing::Span::none()
                    };
                    let _replay_guard = replay_span.enter();
                    let replay_started = self.debug_enabled.then(Instant::now);
                    scene.replay_ops_translated(&self.paint_cache.prev_ops[range.clone()], delta);
                    if let Some(replay_started) = replay_started {
                        self.debug_stats.paint_cache_replay_time = self
                            .debug_stats
                            .paint_cache_replay_time
                            .saturating_add(replay_started.elapsed());
                    }
                    let end = scene.ops_len();
                    replay_span.record("ops", (end - start) as u64);
                    self.debug_record_paint_cache_replay(node, (end - start) as u32);

                    if let Some((prev, next)) = self.nodes.get_mut(node).map(|n| {
                        let prev = n.invalidation;
                        n.paint_cache = Some(PaintCacheEntry {
                            generation: self.paint_cache.target_generation,
                            key,
                            origin: bounds.origin,
                            start: start as u32,
                            end: end as u32,
                        });
                        n.invalidation.paint = false;
                        (prev, n.invalidation)
                    }) {
                        self.update_invalidation_counters(prev, next);
                    }

                    if delta.x.0 != 0.0 || delta.y.0 != 0.0 {
                        // Paint-cache replay translates recorded draw ops by `delta` without visiting
                        // descendants. Keep hit-testing and semantics consistent by translating the
                        // retained subtree bounds too (mirrors the layout-only translation fast path).
                        //
                        // Without this, cached subtrees that move (e.g. due to parent layout changes)
                        // can render correctly while their descendant bounds remain stale, causing
                        // incorrect pointer routing and stale semantics geometry.
                        let translate_started = self.debug_enabled.then(Instant::now);
                        let mut translated_nodes: u32 = 0;
                        let window = self.window;
                        let mut stack = self.take_scratch_node_stack();
                        stack.clear();
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
                            let Some(n) = self.nodes.get_mut(id) else {
                                continue;
                            };
                            translated_nodes = translated_nodes.saturating_add(1);
                            n.bounds.origin = Point::new(
                                n.bounds.origin.x + delta.x,
                                n.bounds.origin.y + delta.y,
                            );
                            if let Some(window) = window
                                && let Some(element) = n.element
                            {
                                crate::elements::record_bounds_for_element(
                                    app, window, element, n.bounds,
                                );
                            }
                            for &child in &n.children {
                                stack.push(child);
                            }
                        }
                        self.restore_scratch_node_stack(stack);
                        if let Some(translate_started) = translate_started {
                            self.debug_stats.paint_cache_bounds_translate_time = self
                                .debug_stats
                                .paint_cache_bounds_translate_time
                                .saturating_add(translate_started.elapsed());
                            self.debug_stats.paint_cache_bounds_translated_nodes = self
                                .debug_stats
                                .paint_cache_bounds_translated_nodes
                                .saturating_add(translated_nodes);
                        }
                    }

                    self.paint_cache.hits = self.paint_cache.hits.saturating_add(1);
                    self.paint_cache.replayed_ops = self
                        .paint_cache
                        .replayed_ops
                        .saturating_add((end - start) as u32);
                    return;
                }
            }
            self.paint_cache.misses = self.paint_cache.misses.saturating_add(1);
            if let Some(hit_check_started) = hit_check_started {
                self.debug_stats.paint_cache_hit_check_time = self
                    .debug_stats
                    .paint_cache_hit_check_time
                    .saturating_add(hit_check_started.elapsed());
            }
        }

        // Clear the "dirty" flag before invoking widget paint so that paint-triggered invalidations
        // (e.g. `request_animation_frame()`) can be recorded for the next frame even when paint
        // caching is enabled.
        if let Some((prev, next)) = self.nodes.get_mut(node).map(|n| {
            let prev = n.invalidation;
            n.invalidation.paint = false;
            (prev, n.invalidation)
        }) {
            self.update_invalidation_counters(prev, next);
        }

        let mut observations = SmallCopyList::<(ModelId, Invalidation), 8>::default();
        let mut observe_model = |model: ModelId, inv: Invalidation| {
            observations.push((model, inv));
        };

        let mut global_observations = SmallCopyList::<(TypeId, Invalidation), 8>::default();
        let mut observe_global = |id: TypeId, inv: Invalidation| {
            global_observations.push((id, inv));
        };

        if self.debug_enabled {
            self.debug_stats.paint_nodes_performed =
                self.debug_stats.paint_nodes_performed.saturating_add(1);
        }

        let widget_started = self.debug_enabled.then(Instant::now);
        let mut widget_type: &'static str = "<unknown>";
        if self.debug_enabled {
            self.debug_paint_stack.push(DebugPaintStackFrame {
                child_inclusive_time: Duration::default(),
                child_inclusive_scene_ops_delta: 0,
            });
        }

        let start = scene.ops_len();
        self.with_widget_mut(node, |widget, tree| {
            if tree.debug_enabled {
                widget_type = widget.debug_type_name();
            }
            let children_render_transform = widget
                .children_render_transform(bounds)
                .filter(|t| t.inverse().is_some());
            let mut children_buf = SmallNodeList::<32>::default();
            if let Some(children) = tree.nodes.get(node).map(|n| n.children.as_slice()) {
                children_buf.set(children);
            }
            let window = tree.window;
            let focus = tree.focus;
            let mut cx = PaintCx {
                app,
                node,
                window,
                focus,
                children: children_buf.as_slice(),
                bounds,
                scale_factor: sf,
                accumulated_transform: current_transform,
                children_render_transform,
                services: &mut *services,
                observe_model: &mut observe_model,
                observe_global: &mut observe_global,
                scene,
                tree,
            };
            let transform = widget.render_transform(bounds);
            let pushed_transform = if let Some(transform) = transform
                && transform.inverse().is_some()
            {
                cx.scene.push(SceneOp::PushTransform { transform });
                true
            } else {
                false
            };

            cx.tree.debug_paint_widget_exclusive_resume();
            widget.paint(&mut cx);
            let _ = cx.tree.debug_paint_widget_exclusive_pause();

            if pushed_transform {
                cx.scene.push(SceneOp::PopTransform);
            }
        });
        let end = scene.ops_len();

        if let Some(widget_started) = widget_started {
            const MAX_PAINT_WIDGET_HOTSPOTS: usize = 16;
            let inclusive_time = widget_started.elapsed();
            let inclusive_scene_ops_delta = end.saturating_sub(start).min(u32::MAX as usize) as u32;
            let (child_inclusive_time, child_inclusive_scene_ops_delta) = self
                .debug_paint_stack
                .pop()
                .map(|f| (f.child_inclusive_time, f.child_inclusive_scene_ops_delta))
                .unwrap_or_default();
            let exclusive_time = inclusive_time.saturating_sub(child_inclusive_time);
            let exclusive_scene_ops_delta =
                inclusive_scene_ops_delta.saturating_sub(child_inclusive_scene_ops_delta);

            if let Some(parent) = self.debug_paint_stack.last_mut() {
                parent.child_inclusive_time += inclusive_time;
                parent.child_inclusive_scene_ops_delta = parent
                    .child_inclusive_scene_ops_delta
                    .saturating_add(inclusive_scene_ops_delta);
            }

            let element = self.nodes.get(node).and_then(|n| n.element);
            let element_kind = self.window.and_then(|window| {
                crate::declarative::frame::element_record_for_node(app, window, node)
                    .map(|record| record.instance.kind_name())
            });
            let record = UiDebugPaintWidgetHotspot {
                node,
                element,
                element_kind,
                widget_type,
                inclusive_time,
                exclusive_time,
                inclusive_scene_ops_delta,
                exclusive_scene_ops_delta,
            };
            let idx = self
                .debug_paint_widget_hotspots
                .iter()
                .position(|h| h.exclusive_time < record.exclusive_time)
                .unwrap_or(self.debug_paint_widget_hotspots.len());
            self.debug_paint_widget_hotspots.insert(idx, record);
            if self.debug_paint_widget_hotspots.len() > MAX_PAINT_WIDGET_HOTSPOTS {
                self.debug_paint_widget_hotspots
                    .truncate(MAX_PAINT_WIDGET_HOTSPOTS);
            }
        }

        let obs_started = self.debug_enabled.then(Instant::now);
        self.observed_in_paint.record(node, observations.as_slice());
        self.observed_globals_in_paint
            .record(node, global_observations.as_slice());
        if let Some(obs_started) = obs_started {
            self.debug_stats.paint_observation_record_time = self
                .debug_stats
                .paint_observation_record_time
                .saturating_add(obs_started.elapsed());
        }
        if let Some(n) = self.nodes.get_mut(node) {
            if cache_enabled {
                n.paint_cache = Some(PaintCacheEntry {
                    generation: self.paint_cache.target_generation,
                    key,
                    origin: bounds.origin,
                    start: start as u32,
                    end: end as u32,
                });
            } else {
                // When caching is disabled for this node (e.g. due to a render transform),
                // ensure we don't keep a stale cache entry that could be replayed later.
                n.paint_cache = None;
            }
        }
    }
}
