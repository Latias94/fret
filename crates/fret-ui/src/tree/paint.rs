use super::*;
use std::any::TypeId;

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
            self.debug_stats.view_cache_active = self.view_cache_active();
            self.debug_stats.focus = self.focus;
            self.debug_stats.captured = self.captured_for(fret_core::PointerId(0));
        }

        // Keep IME enabled state in sync even if focus is set programmatically and no input event
        // has been dispatched yet (ADR 0012).
        let focus_is_text_input = self.focus_is_text_input();
        self.set_ime_allowed(app, focus_is_text_input);
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

        // Scroll offsets can change without triggering layout invalidations (e.g. wheel deltas that
        // only affect hit-testing/paint, or programmatic scroll handle updates in frames that skip
        // layout). Ensure we consume scroll-handle change invalidations before paint-cache replay
        // so cached ancestors cannot replay stale ops.
        self.invalidate_scroll_handle_bindings_for_changed_handles(
            app,
            crate::layout_pass::LayoutPassKind::Final,
        );

        let cache_enabled = self.paint_cache_enabled();
        if cache_enabled {
            self.paint_cache.begin_frame();
        } else {
            self.paint_cache.invalidate_recording();
        }

        let roots: Vec<NodeId> = self
            .visible_layers_in_paint_order()
            .map(|layer| self.layers[layer].root)
            .collect();
        for root in roots {
            self.paint(app, services, root, bounds, scene, scale_factor);
        }

        if cache_enabled {
            self.paint_cache.finish_frame();
            if self.debug_enabled {
                self.debug_stats.paint_cache_hits = self.paint_cache.hits;
                self.debug_stats.paint_cache_misses = self.paint_cache.misses;
                self.debug_stats.paint_cache_replayed_ops = self.paint_cache.replayed_ops;
            }
        }

        self.collapse_paint_observations_to_view_cache_roots_if_needed();

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
            let visual = rect_aabb_transformed(bounds, current_transform);
            crate::elements::record_visual_bounds_for_element(app, window, element, visual);
        }

        let theme_revision = Theme::global(&*app).revision();
        let children_render_transform = self.node_children_render_transform(node);
        let child_transform = children_render_transform.unwrap_or(Transform2D::IDENTITY);
        let key = PaintCacheKey::new(bounds, sf, theme_revision, child_transform);
        let cache_enabled = self.paint_cache_enabled()
            && self.node_render_transform(node).is_none()
            && (!self.view_cache_active()
                || self.nodes.get(node).is_some_and(|n| n.view_cache.enabled));

        if cache_enabled && !invalidated {
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
                    scene.replay_ops_translated(&self.paint_cache.prev_ops[range.clone()], delta);
                    let end = scene.ops_len();
                    replay_span.record("ops", (end - start) as u64);
                    self.debug_record_paint_cache_replay(node, (end - start) as u32);

                    if let Some(n) = self.nodes.get_mut(node) {
                        n.paint_cache = Some(PaintCacheEntry {
                            generation: self.paint_cache.target_generation,
                            key,
                            origin: bounds.origin,
                            start: start as u32,
                            end: end as u32,
                        });
                        n.invalidation.paint = false;
                    }

                    if delta.x.0 != 0.0 || delta.y.0 != 0.0 {
                        // Paint-cache replay translates recorded draw ops by `delta` without visiting
                        // descendants. Keep hit-testing and semantics consistent by translating the
                        // retained subtree bounds too (mirrors the layout-only translation fast path).
                        //
                        // Without this, cached subtrees that move (e.g. due to parent layout changes)
                        // can render correctly while their descendant bounds remain stale, causing
                        // incorrect pointer routing and stale semantics geometry.
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
                            let Some(n) = self.nodes.get_mut(id) else {
                                continue;
                            };
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
        }

        // Clear the "dirty" flag before invoking widget paint so that paint-triggered invalidations
        // (e.g. `request_animation_frame()`) can be recorded for the next frame even when paint
        // caching is enabled.
        if let Some(n) = self.nodes.get_mut(node) {
            n.invalidation.paint = false;
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

        let start = scene.ops_len();
        self.with_widget_mut(node, |widget, tree| {
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

            widget.paint(&mut cx);

            if pushed_transform {
                cx.scene.push(SceneOp::PopTransform);
            }
        });
        let end = scene.ops_len();

        self.observed_in_paint.record(node, observations.as_slice());
        self.observed_globals_in_paint
            .record(node, global_observations.as_slice());
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
