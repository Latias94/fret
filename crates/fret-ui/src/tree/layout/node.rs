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
        overflow_ctx: crate::layout::overflow::LayoutOverflowContext,
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
        let subtree_dirty = self.node_subtree_layout_dirty(node);
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
                layout_dependency = view_cache.parent_layout_dependency.as_debug_str(),
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
            && !subtree_dirty
        {
            let delta = Point::new(
                bounds.origin.x - prev_bounds.origin.x,
                bounds.origin.y - prev_bounds.origin.y,
            );
            if delta.x.0 != 0.0 || delta.y.0 != 0.0 {
                self.layout_engine.mark_seen_if_present(node);
                let mut propagated_bounds = Vec::new();
                propagated_bounds.push((node, bounds));

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

                    let Some(child_bounds) = (|| {
                        let n = self.nodes.get_mut(id)?;
                        n.bounds.origin =
                            Point::new(n.bounds.origin.x + delta.x, n.bounds.origin.y + delta.y);
                        let child_bounds = n.bounds;
                        if !n.layout_dirty_children_suppressed {
                            for &child in &n.children {
                                stack.push(child);
                            }
                        }
                        Some(child_bounds)
                    })() else {
                        continue;
                    };
                    propagated_bounds.push((id, child_bounds));
                }
                if let Some(window) = self.window {
                    for (id, bounds) in propagated_bounds {
                        self.queue_layout_bounds_for_node_element(app, window, id, bounds);
                    }
                }
            }
            return measured;
        }

        // `subtree_dirty` is intentionally *not* used to force a relayout of otherwise-clean
        // ancestors here. Contained view-cache roots can hold descendant layout invalidations that
        // must be handled by the contained relayout pass; forcing ancestor relayouts would clear
        // those invalidations early and defeat the cache boundary semantics.
        //
        // Consumers that need to observe descendant invalidations (e.g. scroll extent updates at
        // the scroll edge) should consult `node_subtree_layout_dirty()` inside their own layout
        // logic instead.
        let needs_layout = invalidated_for_pass || prev_bounds != bounds;
        if !needs_layout {
            return measured;
        }
        if !invalidated_for_pass
            && !subtree_dirty
            && let Some(size) = self.try_propagate_clean_engine_layout(
                app,
                services,
                node,
                bounds,
                prev_bounds,
                measured,
                scale_factor,
                pass_kind,
                overflow_ctx,
            )
        {
            return size;
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
        let profile_widget_timing =
            self.debug_enabled || !self.scroll_layout_kind_profile_stack.is_empty();
        let widget_started = profile_widget_timing.then(Instant::now);
        let mut widget_type: &'static str = "<unknown>";
        if profile_widget_timing {
            self.debug_layout_stack.push(super::DebugLayoutStackFrame {
                child_inclusive_time: Duration::default(),
            });
        }
        let size = self.with_widget_mut(node, |widget, tree| {
            struct LayoutCallDepthGuard(*mut u32);

            impl Drop for LayoutCallDepthGuard {
                fn drop(&mut self) {
                    unsafe {
                        *self.0 = (*self.0).saturating_sub(1);
                    }
                }
            }

            if tree.debug_enabled {
                widget_type = widget.debug_type_name();
            }
            let mut children_buf = SmallNodeList::<32>::default();
            if let Some(children) = tree.nodes.get(node).map(|n| n.children.as_slice()) {
                children_buf.set(children);
            }
            let _layout_call_depth_guard = if pass_kind == LayoutPassKind::Final {
                tree.layout_call_depth = tree.layout_call_depth.saturating_add(1);
                Some(LayoutCallDepthGuard(
                    &mut tree.layout_call_depth as *mut u32,
                ))
            } else {
                None
            };
            let mut cx = LayoutCx {
                app,
                node,
                window: tree.window,
                focus: tree.focus,
                children: children_buf.as_slice(),
                bounds,
                available: bounds.size,
                pass_kind,
                overflow_ctx,
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
            if let Some(scope) = self.scroll_layout_kind_profile_stack.last_mut() {
                let element_kind = self
                    .window
                    .and_then(|window| {
                        crate::declarative::frame::element_record_for_node(app, window, node)
                            .map(|record| record.instance.kind_name())
                    })
                    .unwrap_or("<unknown>");
                scope.record(element_kind, exclusive_time, inclusive_time);
            }
            if let Some(parent) = self.debug_layout_stack.last_mut() {
                parent.child_inclusive_time += inclusive_time;
            }

            let wants_exclusive = self.debug_enabled
                && (self.debug_layout_hotspots.len() < MAX_LAYOUT_HOTSPOTS
                    || self
                        .debug_layout_hotspots
                        .last()
                        .map(|h| h.exclusive_time < exclusive_time)
                        .unwrap_or(true));
            let wants_inclusive = self.debug_enabled
                && (self.debug_layout_inclusive_hotspots.len() < MAX_LAYOUT_HOTSPOTS
                    || self
                        .debug_layout_inclusive_hotspots
                        .last()
                        .map(|h| h.inclusive_time < inclusive_time)
                        .unwrap_or(true));

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
            if let Some((prev, next, layout_before, layout_after)) =
                self.nodes.get_mut(node).map(|n| {
                    n.measured_size = size;
                    let prev = n.invalidation;
                    let layout_before = n.invalidation.layout;
                    if layout_before {
                        debug_assert!(self.layout_invalidations_count > 0);
                        self.layout_invalidations_count =
                            self.layout_invalidations_count.saturating_sub(1);
                    }
                    n.invalidation.layout = false;
                    let layout_after = n.invalidation.layout;
                    (prev, n.invalidation, layout_before, layout_after)
                })
            {
                self.note_layout_invalidation_transition_for_subtree_aggregation(
                    node,
                    layout_before,
                    layout_after,
                );
                if layout_before && !layout_after {
                    self.debug_clear_layout_dirty_source(node);
                }
                self.update_invalidation_counters(prev, next);
                // Main-pass layout can consume a boundary's scheduling-only layout dirty marker
                // before the contained-relayout pass ever looks at it (for example, initial mount
                // or ancestor-driven layout). Keep `dirty_boundaries` aligned with authoritative
                // layout state so clean cache roots do not remain queued across stable frames.
                self.clear_boundary_dirty_tracking_if_clean(node);
            }
            self.recompute_paint_geometry_fingerprint(node);
        }

        size
    }

    #[allow(clippy::too_many_arguments)]
    fn try_propagate_clean_engine_layout(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        node: NodeId,
        bounds: Rect,
        prev_bounds: Rect,
        measured_size: Size,
        scale_factor: f32,
        pass_kind: LayoutPassKind,
        overflow_ctx: crate::layout::overflow::LayoutOverflowContext,
    ) -> Option<Size> {
        if pass_kind != LayoutPassKind::Final {
            return None;
        }
        let window = self.window?;
        let (children, layout_dirty_children_suppressed) = {
            let entry = self.nodes.get(node)?;
            if entry.invalidation.layout || self.node_subtree_layout_dirty(node) {
                return None;
            }
            (
                entry.children.clone(),
                entry.layout_dirty_children_suppressed,
            )
        };
        if measured_size == Size::default() || layout_dirty_children_suppressed {
            return None;
        }
        let element = self.clean_engine_geometry_propagation_supported_element(
            app,
            window,
            node,
            &children,
            bounds,
            prev_bounds,
        )?;

        let child_bounds = if let Some(child_bounds) = self.clean_manual_geometry_child_bounds(
            app,
            window,
            node,
            &children,
            bounds,
            prev_bounds,
        ) {
            child_bounds
        } else {
            let mut child_bounds = Vec::with_capacity(children.len());
            for &child in &children {
                let child_style =
                    crate::declarative::frame::layout_style_for_node(app, window, child);
                if child_style.position != crate::element::PositionStyle::Static {
                    return None;
                }
                let local = self.layout_engine_child_local_rect_profiled(node, child)?;
                child_bounds.push((
                    child,
                    Rect::new(
                        Point::new(
                            Px(bounds.origin.x.0 + local.origin.x.0),
                            Px(bounds.origin.y.0 + local.origin.y.0),
                        ),
                        local.size,
                    ),
                ));
            }
            child_bounds
        };

        self.layout_engine.mark_seen_if_present(node);
        let size = if children.is_empty() && prev_bounds.size == bounds.size {
            measured_size
        } else {
            bounds.size
        };
        if let Some(entry) = self.nodes.get_mut(node) {
            entry.bounds = bounds;
            entry.measured_size = size;
        }
        self.queue_layout_bounds_for_element(element, bounds);

        for (child, child_bounds) in child_bounds {
            let child_prev_bounds = self
                .nodes
                .get(child)
                .map(|entry| entry.bounds)
                .unwrap_or_default();
            let child_measured_size = self
                .nodes
                .get(child)
                .map(|entry| entry.measured_size)
                .unwrap_or_default();
            if self
                .try_propagate_clean_engine_layout(
                    app,
                    services,
                    child,
                    child_bounds,
                    child_prev_bounds,
                    child_measured_size,
                    scale_factor,
                    pass_kind,
                    overflow_ctx,
                )
                .is_none()
            {
                let _ = self.layout_node(
                    app,
                    services,
                    child,
                    child_bounds,
                    scale_factor,
                    pass_kind,
                    overflow_ctx,
                );
            }
        }

        self.recompute_paint_geometry_fingerprint(node);
        Some(size)
    }

    pub(crate) fn can_skip_clean_geometry_engine_solve_for_resize(
        &mut self,
        app: &mut H,
        root: NodeId,
        bounds: Rect,
        prev_bounds: Rect,
    ) -> bool {
        if !self.interactive_resize_is_small_step() {
            return false;
        }
        if prev_bounds == bounds || prev_bounds.size == bounds.size {
            return false;
        }
        if (bounds.size.height.0 - prev_bounds.size.height.0).abs() > 0.01 {
            return false;
        }
        let Some(window) = self.window else {
            return false;
        };
        if !self.clean_geometry_node_is_clean(root) {
            return false;
        }
        self.clean_manual_geometry_subtree_supported(app, window, root, bounds, prev_bounds, true)
    }

    fn clean_manual_geometry_subtree_supported(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        bounds: Rect,
        prev_bounds: Rect,
        is_root: bool,
    ) -> bool {
        if !self.clean_geometry_node_is_clean(node) {
            return false;
        }
        if !is_root && self.clean_geometry_boundary_layout_node(app, window, node) {
            return true;
        }
        let Some(children) = self.nodes.get(node).map(|n| n.children.clone()) else {
            return false;
        };
        let Some(child_bounds) = self.clean_manual_geometry_child_bounds(
            app,
            window,
            node,
            &children,
            bounds,
            prev_bounds,
        ) else {
            return false;
        };
        for (child, child_bounds) in child_bounds {
            if self.clean_geometry_boundary_layout_node(app, window, child) {
                continue;
            }
            let Some(child_prev_bounds) = self.nodes.get(child).map(|entry| entry.bounds) else {
                return false;
            };
            if !self.clean_manual_geometry_subtree_supported(
                app,
                window,
                child,
                child_bounds,
                child_prev_bounds,
                false,
            ) {
                return false;
            }
        }
        true
    }

    fn clean_geometry_node_is_clean(&self, node: NodeId) -> bool {
        let Some(entry) = self.nodes.get(node) else {
            return false;
        };
        !entry.invalidation.layout
            && !self.node_subtree_layout_dirty(node)
            && entry.measured_size != Size::default()
            && !entry.layout_dirty_children_suppressed
    }

    fn clean_geometry_boundary_layout_node(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
    ) -> bool {
        crate::declarative::frame::with_element_record_for_node(app, window, node, |record| {
            matches!(
                record.instance,
                crate::declarative::frame::ElementInstance::Scroll(_)
            )
        })
        .unwrap_or(false)
    }

    fn clean_manual_geometry_child_bounds(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        children: &[NodeId],
        bounds: Rect,
        prev_bounds: Rect,
    ) -> Option<Vec<(NodeId, Rect)>> {
        let record = crate::declarative::frame::element_record_for_node(app, window, node)?;
        match record.instance {
            crate::declarative::frame::ElementInstance::Stack(_)
            | crate::declarative::frame::ElementInstance::Pressable(_)
            | crate::declarative::frame::ElementInstance::Semantics(_)
            | crate::declarative::frame::ElementInstance::FocusScope(_)
            | crate::declarative::frame::ElementInstance::ForegroundScope(_)
            | crate::declarative::frame::ElementInstance::Opacity(_)
            | crate::declarative::frame::ElementInstance::EffectLayer(_)
            | crate::declarative::frame::ElementInstance::BackdropSourceGroup(_)
            | crate::declarative::frame::ElementInstance::MaskLayer(_)
            | crate::declarative::frame::ElementInstance::CompositeGroup(_)
            | crate::declarative::frame::ElementInstance::PointerRegion(_)
            | crate::declarative::frame::ElementInstance::HoverRegion(_)
            | crate::declarative::frame::ElementInstance::WheelRegion(_)
            | crate::declarative::frame::ElementInstance::InternalDragRegion(_)
            | crate::declarative::frame::ElementInstance::ExternalDragRegion(_)
            | crate::declarative::frame::ElementInstance::InteractivityGate(_)
            | crate::declarative::frame::ElementInstance::HitTestGate(_)
            | crate::declarative::frame::ElementInstance::FocusTraversalGate(_)
            | crate::declarative::frame::ElementInstance::DismissibleLayer(_) => self
                .clean_passthrough_width_delta_child_bounds(
                    app,
                    window,
                    children,
                    bounds,
                    prev_bounds,
                ),
            crate::declarative::frame::ElementInstance::Flex(props)
            | crate::declarative::frame::ElementInstance::SemanticFlex(
                crate::element::SemanticFlexProps { flex: props, .. },
            )
            | crate::declarative::frame::ElementInstance::RovingFlex(
                crate::element::RovingFlexProps { flex: props, .. },
            ) => self.clean_vertical_flex_width_delta_child_bounds(
                app,
                window,
                children,
                bounds,
                prev_bounds,
                props,
            ),
            crate::declarative::frame::ElementInstance::Spacer(_)
            | crate::declarative::frame::ElementInstance::Image(_)
            | crate::declarative::frame::ElementInstance::SvgIcon(_)
            | crate::declarative::frame::ElementInstance::SvgImage(_)
            | crate::declarative::frame::ElementInstance::Spinner(_)
                if children.is_empty() =>
            {
                Some(Vec::new())
            }
            _ => None,
        }
    }

    fn clean_passthrough_width_delta_child_bounds(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        children: &[NodeId],
        bounds: Rect,
        prev_bounds: Rect,
    ) -> Option<Vec<(NodeId, Rect)>> {
        let mut out = Vec::with_capacity(children.len());
        for &child in children {
            let child_style = crate::declarative::frame::layout_style_for_node(app, window, child);
            if child_style.position != crate::element::PositionStyle::Static
                || !Self::clean_margin_edges_are_px(child_style.margin)
            {
                return None;
            }
            let prev_child = self.nodes.get(child)?.bounds;
            let local_x = prev_child.origin.x.0 - prev_bounds.origin.x.0;
            let local_y = prev_child.origin.y.0 - prev_bounds.origin.y.0;
            let width = if (prev_child.size.width.0 - prev_bounds.size.width.0).abs() <= 0.01
                || matches!(child_style.size.width, crate::element::Length::Fill)
            {
                bounds.size.width
            } else {
                prev_child.size.width
            };
            out.push((
                child,
                Rect::new(
                    Point::new(
                        Px(bounds.origin.x.0 + local_x),
                        Px(bounds.origin.y.0 + local_y),
                    ),
                    Size::new(width, prev_child.size.height),
                ),
            ));
        }
        Some(out)
    }

    fn clean_vertical_flex_width_delta_child_bounds(
        &self,
        app: &mut H,
        window: AppWindowId,
        children: &[NodeId],
        bounds: Rect,
        prev_bounds: Rect,
        props: crate::element::FlexProps,
    ) -> Option<Vec<(NodeId, Rect)>> {
        if props.wrap
            || props.direction != fret_core::Axis::Vertical
            || props.justify != crate::element::MainAlign::Start
            || props.align != crate::element::CrossAlign::Stretch
            || (bounds.size.height.0 - prev_bounds.size.height.0).abs() > 0.01
        {
            return None;
        }
        let pad_left = Self::clean_spacing_px(props.padding.left)?;
        let pad_right = Self::clean_spacing_px(props.padding.right)?;
        let _pad_top = Self::clean_spacing_px(props.padding.top)?;
        let _pad_bottom = Self::clean_spacing_px(props.padding.bottom)?;
        let _gap = Self::clean_spacing_px(props.gap)?;

        let prev_inner_width = (prev_bounds.size.width.0 - pad_left - pad_right).max(0.0);
        let next_inner_width = (bounds.size.width.0 - pad_left - pad_right).max(0.0);
        let mut out = Vec::with_capacity(children.len());
        for &child in children {
            let child_style = crate::declarative::frame::layout_style_for_node(app, window, child);
            if child_style.position != crate::element::PositionStyle::Static
                || !Self::clean_margin_edges_are_px(child_style.margin)
                || matches!(child_style.size.height, crate::element::Length::Auto)
            {
                return None;
            }
            let prev_child = self.nodes.get(child)?.bounds;
            let local_x = prev_child.origin.x.0 - prev_bounds.origin.x.0;
            let local_y = prev_child.origin.y.0 - prev_bounds.origin.y.0;
            let width = if (prev_child.size.width.0 - prev_inner_width).abs() <= 0.01
                || matches!(child_style.size.width, crate::element::Length::Fill)
            {
                Px(next_inner_width)
            } else {
                prev_child.size.width
            };
            out.push((
                child,
                Rect::new(
                    Point::new(
                        Px(bounds.origin.x.0 + local_x),
                        Px(bounds.origin.y.0 + local_y),
                    ),
                    Size::new(width, prev_child.size.height),
                ),
            ));
        }
        Some(out)
    }

    fn clean_spacing_px(length: crate::element::SpacingLength) -> Option<f32> {
        match length {
            crate::element::SpacingLength::Px(px) => Some(px.0.max(0.0)),
            crate::element::SpacingLength::Fill | crate::element::SpacingLength::Fraction(_) => {
                None
            }
        }
    }

    fn clean_margin_edges_are_px(margin: crate::element::MarginEdges) -> bool {
        matches!(margin.left, crate::element::MarginEdge::Px(_))
            && matches!(margin.right, crate::element::MarginEdge::Px(_))
            && matches!(margin.top, crate::element::MarginEdge::Px(_))
            && matches!(margin.bottom, crate::element::MarginEdge::Px(_))
    }

    fn clean_engine_geometry_propagation_supported_element(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        children: &[NodeId],
        bounds: Rect,
        prev_bounds: Rect,
    ) -> Option<GlobalElementId> {
        let Some(record) = crate::declarative::frame::element_record_for_node(app, window, node)
        else {
            return None;
        };

        let supported = match record.instance {
            crate::declarative::frame::ElementInstance::Stack(_) => true,
            crate::declarative::frame::ElementInstance::Flex(_)
            | crate::declarative::frame::ElementInstance::SemanticFlex(_)
            | crate::declarative::frame::ElementInstance::RovingFlex(_) => {
                !children.iter().copied().any(|child| {
                    let style =
                        crate::declarative::frame::layout_style_for_node(app, window, child);
                    matches!(style.margin.left, crate::element::MarginEdge::Auto)
                        || matches!(style.margin.right, crate::element::MarginEdge::Auto)
                        || matches!(style.margin.top, crate::element::MarginEdge::Auto)
                        || matches!(style.margin.bottom, crate::element::MarginEdge::Auto)
                })
            }
            crate::declarative::frame::ElementInstance::Spacer(_) => children.is_empty(),
            crate::declarative::frame::ElementInstance::Text(_)
            | crate::declarative::frame::ElementInstance::StyledText(_)
            | crate::declarative::frame::ElementInstance::SelectableText(_) => {
                children.is_empty() && prev_bounds.size == bounds.size
            }
            _ => false,
        };
        supported.then_some(record.element)
    }

    fn queue_layout_bounds_for_node_element(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        bounds: Rect,
    ) {
        if let Some(element) =
            crate::declarative::frame::with_element_record_for_node(app, window, node, |record| {
                record.element
            })
        {
            self.queue_layout_bounds_for_element(element, bounds);
        }
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
        let inherited_text_style_fingerprint = self.window.and_then(|window| {
            crate::declarative::frame::inherited_text_style_for_node(app, window, node)
                .as_ref()
                .map(crate::text_props::text_style_refinement_fingerprint)
        });
        let cache_key = NodeMeasureCacheKey {
            known_w_bits: constraints.known.width.map(|px| px.0.to_bits()),
            known_h_bits: constraints.known.height.map(|px| px.0.to_bits()),
            avail_w,
            avail_h,
            scale_bits: scale_factor.to_bits(),
            text_style_present: inherited_text_style_fingerprint.is_some(),
            text_style_fingerprint: inherited_text_style_fingerprint.unwrap_or(0),
        };

        let key = MeasureStackKey {
            node,
            known_w_bits: cache_key.known_w_bits,
            known_h_bits: cache_key.known_h_bits,
            avail_w,
            avail_h,
            scale_bits: cache_key.scale_bits,
            text_style_present: cache_key.text_style_present,
            text_style_fingerprint: cache_key.text_style_fingerprint,
        };

        if let Some(size) = self.measure_cache_this_frame.get(&key) {
            return *size;
        }

        if let Some(n) = self.nodes.get(node)
            && !n.invalidation.layout
            && !self.node_subtree_layout_dirty(node)
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
            let record = super::UiDebugWidgetMeasureHotspot {
                node,
                element,
                element_kind,
                element_path,
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
