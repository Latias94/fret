use super::*;
use std::any::TypeId;

use crate::layout_constraints::{AvailableSpace, LayoutConstraints};
use crate::layout_pass::LayoutPassKind;

impl<H: UiHost> UiTree<H> {
    pub(super) fn layout_node(
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
                    for &child in &n.children {
                        stack.push(child);
                    }
                }
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

        let skip_observation_recording =
            !is_probe && self.interactive_resize_active() && !invalidated_for_pass;

        let mut observations = SmallCopyList::<(ModelId, Invalidation), 8>::default();
        let mut global_observations = SmallCopyList::<(TypeId, Invalidation), 8>::default();

        let mut record_model_observation = |model: ModelId, inv: Invalidation| {
            observations.push((model, inv));
        };
        let mut record_global_observation = |id: TypeId, inv: Invalidation| {
            global_observations.push((id, inv));
        };

        let mut discard_model_observation = |_model: ModelId, _inv: Invalidation| {};
        let mut discard_global_observation = |_id: TypeId, _inv: Invalidation| {};

        let observe_model: &mut dyn FnMut(ModelId, Invalidation) = if skip_observation_recording {
            &mut discard_model_observation
        } else {
            &mut record_model_observation
        };
        let observe_global: &mut dyn FnMut(TypeId, Invalidation) = if skip_observation_recording {
            &mut discard_global_observation
        } else {
            &mut record_global_observation
        };

        if !skip_observation_recording {
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
        }

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
                observe_model,
                observe_global,
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

            let wants_exclusive = self.debug_layout_hotspots.len() < MAX_LAYOUT_HOTSPOTS
                || self
                    .debug_layout_hotspots
                    .last()
                    .map(|h| h.exclusive_time < exclusive_time)
                    .unwrap_or(true);
            let wants_inclusive = self.debug_layout_inclusive_hotspots.len() < MAX_LAYOUT_HOTSPOTS
                || self
                    .debug_layout_inclusive_hotspots
                    .last()
                    .map(|h| h.inclusive_time < inclusive_time)
                    .unwrap_or(true);

            if wants_exclusive || wants_inclusive {
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

                if wants_exclusive {
                    let idx = self
                        .debug_layout_hotspots
                        .iter()
                        .position(|h| h.exclusive_time < record.exclusive_time)
                        .unwrap_or(self.debug_layout_hotspots.len());
                    self.debug_layout_hotspots.insert(idx, record.clone());
                    if self.debug_layout_hotspots.len() > MAX_LAYOUT_HOTSPOTS {
                        self.debug_layout_hotspots.truncate(MAX_LAYOUT_HOTSPOTS);
                    }
                }

                if wants_inclusive {
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
            }
        }

        if !is_probe {
            if !skip_observation_recording {
                let obs_started = self.debug_enabled.then(Instant::now);
                let model_items = observations.as_slice().len().min(u32::MAX as usize) as u32;
                let global_items =
                    global_observations.as_slice().len().min(u32::MAX as usize) as u32;
                self.observed_in_layout
                    .record(node, observations.as_slice());
                self.observed_globals_in_layout
                    .record(node, global_observations.as_slice());
                if let Some(obs_started) = obs_started {
                    self.debug_stats.layout_observation_record_time = self
                        .debug_stats
                        .layout_observation_record_time
                        .saturating_add(obs_started.elapsed());
                }
                if self.debug_enabled {
                    self.debug_stats.layout_observation_record_models_items = self
                        .debug_stats
                        .layout_observation_record_models_items
                        .saturating_add(model_items);
                    self.debug_stats.layout_observation_record_globals_items = self
                        .debug_stats
                        .layout_observation_record_globals_items
                        .saturating_add(global_items);
                }
            }
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

    pub(super) fn measure_node(
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

        let skip_observation_recording = self.interactive_resize_active()
            && self.nodes.get(node).is_some_and(|n| !n.invalidation.layout);

        let mut observations = SmallCopyList::<(ModelId, Invalidation), 8>::default();
        let mut global_observations = SmallCopyList::<(TypeId, Invalidation), 8>::default();

        let mut record_model_observation = |model: ModelId, inv: Invalidation| {
            observations.push((model, inv));
        };
        let mut record_global_observation = |id: TypeId, inv: Invalidation| {
            global_observations.push((id, inv));
        };

        let mut discard_model_observation = |_model: ModelId, _inv: Invalidation| {};
        let mut discard_global_observation = |_id: TypeId, _inv: Invalidation| {};

        let observe_model: &mut dyn FnMut(ModelId, Invalidation) = if skip_observation_recording {
            &mut discard_model_observation
        } else {
            &mut record_model_observation
        };
        let observe_global: &mut dyn FnMut(TypeId, Invalidation) = if skip_observation_recording {
            &mut discard_global_observation
        } else {
            &mut record_global_observation
        };

        if !skip_observation_recording {
            observe_global(TypeId::of::<Theme>(), Invalidation::Layout);
            observe_global(
                TypeId::of::<fret_runtime::TextFontStackKey>(),
                Invalidation::Layout,
            );
        }

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
                observe_model,
                observe_global,
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

        if !skip_observation_recording {
            let obs_started = self.debug_enabled.then(Instant::now);
            let model_items = observations.as_slice().len().min(u32::MAX as usize) as u32;
            let global_items = global_observations.as_slice().len().min(u32::MAX as usize) as u32;
            self.observed_in_layout
                .record(node, observations.as_slice());
            self.observed_globals_in_layout
                .record(node, global_observations.as_slice());
            if let Some(obs_started) = obs_started {
                self.debug_stats.layout_observation_record_time = self
                    .debug_stats
                    .layout_observation_record_time
                    .saturating_add(obs_started.elapsed());
            }
            if self.debug_enabled {
                self.debug_stats.layout_observation_record_models_items = self
                    .debug_stats
                    .layout_observation_record_models_items
                    .saturating_add(model_items);
                self.debug_stats.layout_observation_record_globals_items = self
                    .debug_stats
                    .layout_observation_record_globals_items
                    .saturating_add(global_items);
            }
        }

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
