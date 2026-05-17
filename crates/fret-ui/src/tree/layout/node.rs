use super::*;
use std::any::TypeId;

use crate::layout_constraints::{AvailableSpace, LayoutConstraints};
use crate::layout_pass::LayoutPassKind;
use crate::tree::UiDebugCleanGeometrySolveSkipRejection;

#[derive(Debug, Clone, Copy)]
enum CleanGeometrySolveSkipDecision {
    Supported,
    Rejected(CleanGeometrySolveSkipRejection),
}

#[derive(Debug, Clone, Copy)]
struct CleanGeometrySolveSkipRejection {
    reason: CleanGeometrySolveSkipRejectionReason,
    node: Option<NodeId>,
    element_kind: Option<&'static str>,
}

impl CleanGeometrySolveSkipRejection {
    fn new(reason: CleanGeometrySolveSkipRejectionReason) -> Self {
        Self {
            reason,
            node: None,
            element_kind: None,
        }
    }

    fn for_kind(reason: CleanGeometrySolveSkipRejectionReason, element_kind: &'static str) -> Self {
        Self {
            reason,
            node: None,
            element_kind: Some(element_kind),
        }
    }

    fn at_node(mut self, node: NodeId) -> Self {
        self.node = Some(node);
        self
    }

    fn at_node_if_missing(mut self, node: NodeId) -> Self {
        if self.node.is_none() {
            self.node = Some(node);
        }
        self
    }
}

#[derive(Debug, Clone, Copy)]
enum CleanGeometrySolveSkipRejectionReason {
    NotInteractiveResizeSmallStep,
    NoSizeDelta,
    HeightDelta,
    MissingWindow,
    MissingNode,
    LayoutDirty,
    SubtreeLayoutDirty,
    MissingMeasuredSize,
    DirtyChildrenSuppressed,
    MissingElementRecord,
    SideEffectBoundary,
    UnsupportedKind,
    PositionedChild,
    NonPxMargin,
    FlexWrap,
    FlexDirection,
    FlexMainAlign,
    FlexCrossAlign,
    FlexHeightDelta,
    FlexItemSizing,
    GridTrackSizing,
    GridItemSizing,
    NonPxSpacing,
    AutoChildHeight,
    TextReflow,
    ContainerHeightDelta,
    FractionalChildSize,
}

#[derive(Clone)]
struct CleanGeometryNodeContract {
    layout_effect: CleanGeometryLayoutEffect,
    child_bounds: CleanGeometryChildBoundsStrategy,
    size_stability: CleanGeometryWidthDeltaSizeStability,
}

#[derive(Clone, Copy)]
enum CleanGeometryLayoutEffect {
    /// Layout publishes no side effects beyond geometry.
    Pure,
    /// Own layout must run; ancestors may only propagate this node's resized bounds.
    SideEffectBoundary,
}

#[derive(Clone)]
enum CleanGeometryChildBoundsStrategy {
    /// Leaf node: no child bounds to derive.
    None,
    /// Geometry-only wrapper whose children keep previous local origins and may stretch in width.
    PreserveLocalOrigins,
    /// Box-sizing:border-box container subset with px insets and static children.
    ContainerPxInsets(crate::element::ContainerProps),
    /// Vertical, no-wrap flex subset whose line structure is stable across small width deltas.
    VerticalNoWrapFlex(crate::element::FlexProps),
    /// Horizontal, no-wrap flex subset with stable main-axis distribution.
    HorizontalFixedFlex(crate::element::FlexProps),
    /// One-column grid subset whose row structure is stable across small width deltas.
    SingleColumnAutoRowsGrid(crate::element::GridProps),
}

#[derive(Clone, Copy)]
enum CleanGeometryWidthDeltaSizeStability {
    /// The node may take the propagated bounds size.
    Propagated,
    /// The node is stable only when the computed box size does not change.
    StableComputedBox,
}

impl CleanGeometryNodeContract {
    fn pure(child_bounds: CleanGeometryChildBoundsStrategy) -> Self {
        Self {
            layout_effect: CleanGeometryLayoutEffect::Pure,
            child_bounds,
            size_stability: CleanGeometryWidthDeltaSizeStability::Propagated,
        }
    }

    fn stable_leaf() -> Self {
        Self {
            layout_effect: CleanGeometryLayoutEffect::Pure,
            child_bounds: CleanGeometryChildBoundsStrategy::None,
            size_stability: CleanGeometryWidthDeltaSizeStability::StableComputedBox,
        }
    }

    fn propagated_leaf() -> Self {
        Self {
            layout_effect: CleanGeometryLayoutEffect::Pure,
            child_bounds: CleanGeometryChildBoundsStrategy::None,
            size_stability: CleanGeometryWidthDeltaSizeStability::Propagated,
        }
    }

    fn side_effect_boundary() -> Self {
        Self {
            layout_effect: CleanGeometryLayoutEffect::SideEffectBoundary,
            child_bounds: CleanGeometryChildBoundsStrategy::None,
            size_stability: CleanGeometryWidthDeltaSizeStability::Propagated,
        }
    }
}

impl CleanGeometrySolveSkipRejectionReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::NotInteractiveResizeSmallStep => "not_interactive_resize_small_step",
            Self::NoSizeDelta => "no_size_delta",
            Self::HeightDelta => "height_delta",
            Self::MissingWindow => "missing_window",
            Self::MissingNode => "missing_node",
            Self::LayoutDirty => "layout_dirty",
            Self::SubtreeLayoutDirty => "subtree_layout_dirty",
            Self::MissingMeasuredSize => "missing_measured_size",
            Self::DirtyChildrenSuppressed => "dirty_children_suppressed",
            Self::MissingElementRecord => "missing_element_record",
            Self::SideEffectBoundary => "side_effect_boundary",
            Self::UnsupportedKind => "unsupported_kind",
            Self::PositionedChild => "positioned_child",
            Self::NonPxMargin => "non_px_margin",
            Self::FlexWrap => "flex_wrap",
            Self::FlexDirection => "flex_direction",
            Self::FlexMainAlign => "flex_main_align",
            Self::FlexCrossAlign => "flex_cross_align",
            Self::FlexHeightDelta => "flex_height_delta",
            Self::FlexItemSizing => "flex_item_sizing",
            Self::GridTrackSizing => "grid_track_sizing",
            Self::GridItemSizing => "grid_item_sizing",
            Self::NonPxSpacing => "non_px_spacing",
            Self::AutoChildHeight => "auto_child_height",
            Self::TextReflow => "text_reflow",
            Self::ContainerHeightDelta => "container_height_delta",
            Self::FractionalChildSize => "fractional_child_size",
        }
    }
}

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
        let manual_child_bounds_required =
            self.clean_engine_geometry_propagation_requires_manual_child_bounds(app, window, node);

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
            if manual_child_bounds_required {
                return None;
            }
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
        match self.clean_geometry_engine_solve_skip_decision(app, root, bounds, prev_bounds) {
            CleanGeometrySolveSkipDecision::Supported => {
                if self.debug_enabled {
                    self.debug_clean_geometry_solve_skip_rejections
                        .remove(&root);
                }
                true
            }
            CleanGeometrySolveSkipDecision::Rejected(rejection) => {
                self.debug_record_clean_geometry_solve_skip_rejection(
                    app,
                    root,
                    rejection.at_node_if_missing(root),
                );
                false
            }
        }
    }

    fn clean_geometry_engine_solve_skip_decision(
        &mut self,
        app: &mut H,
        root: NodeId,
        bounds: Rect,
        prev_bounds: Rect,
    ) -> CleanGeometrySolveSkipDecision {
        if !self.interactive_resize_is_small_step() {
            return CleanGeometrySolveSkipDecision::Rejected(CleanGeometrySolveSkipRejection::new(
                CleanGeometrySolveSkipRejectionReason::NotInteractiveResizeSmallStep,
            ));
        }
        if prev_bounds == bounds || prev_bounds.size == bounds.size {
            return CleanGeometrySolveSkipDecision::Rejected(CleanGeometrySolveSkipRejection::new(
                CleanGeometrySolveSkipRejectionReason::NoSizeDelta,
            ));
        }
        if (bounds.size.height.0 - prev_bounds.size.height.0).abs() > 0.01 {
            return CleanGeometrySolveSkipDecision::Rejected(CleanGeometrySolveSkipRejection::new(
                CleanGeometrySolveSkipRejectionReason::HeightDelta,
            ));
        }
        let Some(window) = self.window else {
            return CleanGeometrySolveSkipDecision::Rejected(CleanGeometrySolveSkipRejection::new(
                CleanGeometrySolveSkipRejectionReason::MissingWindow,
            ));
        };
        if let Err(rejection) = self.clean_geometry_node_clean_result(root) {
            return CleanGeometrySolveSkipDecision::Rejected(rejection.at_node_if_missing(root));
        }
        match self.clean_manual_geometry_subtree_supported_checked(
            app,
            window,
            root,
            bounds,
            prev_bounds,
            true,
        ) {
            Ok(()) => CleanGeometrySolveSkipDecision::Supported,
            Err(rejection) => {
                CleanGeometrySolveSkipDecision::Rejected(rejection.at_node_if_missing(root))
            }
        }
    }

    fn debug_record_clean_geometry_solve_skip_rejection(
        &mut self,
        app: &mut H,
        root: NodeId,
        rejection: CleanGeometrySolveSkipRejection,
    ) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.layout_clean_geometry_solve_skip_rejections = self
            .debug_stats
            .layout_clean_geometry_solve_skip_rejections
            .saturating_add(1);
        if self
            .debug_stats
            .layout_clean_geometry_solve_skip_first_rejection
            .is_none()
        {
            self.debug_stats
                .layout_clean_geometry_solve_skip_first_rejection = Some(rejection.reason.as_str());
            self.debug_stats
                .layout_clean_geometry_solve_skip_first_element_kind = rejection.element_kind;
        }
        let rejected_node = rejection.node.unwrap_or(root);
        let rejected_element = self
            .nodes
            .get(rejected_node)
            .and_then(|entry| entry.element)
            .or_else(|| {
                self.window.and_then(|window| {
                    crate::declarative::frame::element_record_for_node(app, window, rejected_node)
                        .map(|record| record.element)
                })
            });
        let rejected_element_kind = rejection.element_kind.or_else(|| {
            self.window.and_then(|window| {
                crate::declarative::frame::element_record_for_node(app, window, rejected_node)
                    .map(|record| record.instance.kind_name())
            })
        });
        let rejected_element_path = {
            #[cfg(feature = "diagnostics")]
            {
                self.window.and_then(|window| {
                    rejected_element.and_then(|element| {
                        crate::elements::with_window_state(app, window, |st| {
                            st.debug_path_for_element(element)
                        })
                    })
                })
            }
            #[cfg(not(feature = "diagnostics"))]
            {
                let _ = rejected_element;
                None
            }
        };
        self.debug_clean_geometry_solve_skip_rejections.insert(
            root,
            UiDebugCleanGeometrySolveSkipRejection {
                reason: rejection.reason.as_str(),
                node: Some(rejected_node),
                element: rejected_element,
                element_kind: rejected_element_kind,
                element_path: rejected_element_path,
            },
        );
    }

    fn clean_manual_geometry_subtree_supported_checked(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        bounds: Rect,
        prev_bounds: Rect,
        is_root: bool,
    ) -> Result<(), CleanGeometrySolveSkipRejection> {
        self.clean_geometry_node_clean_result(node)?;
        if !is_root
            && self
                .clean_geometry_boundary_layout_node_kind(app, window, node)
                .is_some()
        {
            return Ok(());
        }
        let children = self
            .nodes
            .get(node)
            .map(|n| n.children.clone())
            .ok_or_else(|| {
                CleanGeometrySolveSkipRejection::new(
                    CleanGeometrySolveSkipRejectionReason::MissingNode,
                )
                .at_node(node)
            })?;
        let child_bounds = self
            .clean_manual_geometry_child_bounds_checked(
                app,
                window,
                node,
                &children,
                bounds,
                prev_bounds,
            )
            .map_err(|rejection| rejection.at_node_if_missing(node))?;
        for (child, child_bounds) in child_bounds {
            if self
                .clean_geometry_boundary_layout_node_kind(app, window, child)
                .is_some()
            {
                continue;
            }
            let child_prev_bounds =
                self.nodes
                    .get(child)
                    .map(|entry| entry.bounds)
                    .ok_or_else(|| {
                        CleanGeometrySolveSkipRejection::new(
                            CleanGeometrySolveSkipRejectionReason::MissingNode,
                        )
                        .at_node(child)
                    })?;
            self.clean_manual_geometry_subtree_supported_checked(
                app,
                window,
                child,
                child_bounds,
                child_prev_bounds,
                false,
            )?;
        }
        Ok(())
    }

    fn clean_geometry_node_clean_result(
        &self,
        node: NodeId,
    ) -> Result<(), CleanGeometrySolveSkipRejection> {
        let Some(entry) = self.nodes.get(node) else {
            return Err(CleanGeometrySolveSkipRejection::new(
                CleanGeometrySolveSkipRejectionReason::MissingNode,
            )
            .at_node(node));
        };
        if entry.invalidation.layout {
            return Err(CleanGeometrySolveSkipRejection::new(
                CleanGeometrySolveSkipRejectionReason::LayoutDirty,
            )
            .at_node(node));
        }
        if self.node_subtree_layout_dirty(node) {
            return Err(CleanGeometrySolveSkipRejection::new(
                CleanGeometrySolveSkipRejectionReason::SubtreeLayoutDirty,
            )
            .at_node(node));
        }
        if entry.measured_size == Size::default() {
            return Err(CleanGeometrySolveSkipRejection::new(
                CleanGeometrySolveSkipRejectionReason::MissingMeasuredSize,
            )
            .at_node(node));
        }
        if entry.layout_dirty_children_suppressed {
            return Err(CleanGeometrySolveSkipRejection::new(
                CleanGeometrySolveSkipRejectionReason::DirtyChildrenSuppressed,
            )
            .at_node(node));
        }
        Ok(())
    }

    fn clean_geometry_boundary_layout_node_kind(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
    ) -> Option<&'static str> {
        let children = self
            .nodes
            .get(node)
            .map(|entry| entry.children.as_slice())?;
        crate::declarative::frame::with_element_record_for_node(app, window, node, |record| {
            let kind = record.instance.kind_name();
            let Ok(contract) = Self::clean_geometry_node_contract(&record.instance, children, kind)
            else {
                return None;
            };
            if matches!(
                contract.layout_effect,
                CleanGeometryLayoutEffect::SideEffectBoundary
            ) {
                Some(kind)
            } else {
                None
            }
        })
        .flatten()
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
        self.clean_manual_geometry_child_bounds_checked(
            app,
            window,
            node,
            children,
            bounds,
            prev_bounds,
        )
        .ok()
    }

    fn clean_manual_geometry_child_bounds_checked(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        children: &[NodeId],
        bounds: Rect,
        prev_bounds: Rect,
    ) -> Result<Vec<(NodeId, Rect)>, CleanGeometrySolveSkipRejection> {
        let record = crate::declarative::frame::element_record_for_node(app, window, node)
            .ok_or_else(|| {
                CleanGeometrySolveSkipRejection::new(
                    CleanGeometrySolveSkipRejectionReason::MissingElementRecord,
                )
                .at_node(node)
            })?;
        let kind = record.instance.kind_name();
        let contract = Self::clean_geometry_node_contract(&record.instance, children, kind)
            .map_err(|rejection| rejection.at_node_if_missing(node))?;
        match contract.layout_effect {
            CleanGeometryLayoutEffect::Pure => {}
            CleanGeometryLayoutEffect::SideEffectBoundary => {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::SideEffectBoundary,
                    kind,
                )
                .at_node(node));
            }
        }
        match contract.size_stability {
            CleanGeometryWidthDeltaSizeStability::Propagated => {}
            CleanGeometryWidthDeltaSizeStability::StableComputedBox => {
                if !Self::clean_size_matches(bounds.size, prev_bounds.size) {
                    return Err(CleanGeometrySolveSkipRejection::for_kind(
                        CleanGeometrySolveSkipRejectionReason::TextReflow,
                        kind,
                    )
                    .at_node(node));
                }
            }
        }
        match contract.child_bounds {
            CleanGeometryChildBoundsStrategy::PreserveLocalOrigins => self
                .clean_preserved_origin_width_delta_child_bounds_checked(
                    app,
                    window,
                    children,
                    bounds,
                    prev_bounds,
                    kind,
                ),
            CleanGeometryChildBoundsStrategy::VerticalNoWrapFlex(props) => self
                .clean_vertical_flex_width_delta_child_bounds(
                    app,
                    window,
                    children,
                    bounds,
                    prev_bounds,
                    props,
                    kind,
                ),
            CleanGeometryChildBoundsStrategy::HorizontalFixedFlex(props) => self
                .clean_horizontal_fixed_flex_width_delta_child_bounds(
                    app,
                    window,
                    children,
                    bounds,
                    prev_bounds,
                    props,
                    kind,
                ),
            CleanGeometryChildBoundsStrategy::ContainerPxInsets(props) => self
                .clean_container_width_delta_child_bounds(
                    app,
                    window,
                    children,
                    bounds,
                    prev_bounds,
                    props,
                    kind,
                ),
            CleanGeometryChildBoundsStrategy::SingleColumnAutoRowsGrid(props) => self
                .clean_single_column_auto_rows_grid_width_delta_child_bounds(
                    app,
                    window,
                    children,
                    bounds,
                    prev_bounds,
                    props,
                    kind,
                ),
            CleanGeometryChildBoundsStrategy::None => Ok(Vec::new()),
        }
    }

    fn clean_geometry_node_contract(
        instance: &crate::declarative::frame::ElementInstance,
        children: &[NodeId],
        kind: &'static str,
    ) -> Result<CleanGeometryNodeContract, CleanGeometrySolveSkipRejection> {
        match instance {
            crate::declarative::frame::ElementInstance::Scroll(_)
            | crate::declarative::frame::ElementInstance::TextInput(_) => {
                Ok(CleanGeometryNodeContract::side_effect_boundary())
            }
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
            | crate::declarative::frame::ElementInstance::DismissibleLayer(_) => {
                Ok(CleanGeometryNodeContract::pure(
                    CleanGeometryChildBoundsStrategy::PreserveLocalOrigins,
                ))
            }
            crate::declarative::frame::ElementInstance::Container(props) => {
                Ok(CleanGeometryNodeContract::pure(
                    CleanGeometryChildBoundsStrategy::ContainerPxInsets(*props),
                ))
            }
            crate::declarative::frame::ElementInstance::Flex(props) => Ok(
                CleanGeometryNodeContract::pure(Self::clean_flex_child_bounds_strategy(*props)),
            ),
            crate::declarative::frame::ElementInstance::Grid(props) => {
                Ok(CleanGeometryNodeContract::pure(
                    CleanGeometryChildBoundsStrategy::SingleColumnAutoRowsGrid(props.clone()),
                ))
            }
            crate::declarative::frame::ElementInstance::SemanticFlex(props) => Ok(
                CleanGeometryNodeContract::pure(Self::clean_flex_child_bounds_strategy(props.flex)),
            ),
            crate::declarative::frame::ElementInstance::RovingFlex(props) => Ok(
                CleanGeometryNodeContract::pure(Self::clean_flex_child_bounds_strategy(props.flex)),
            ),
            crate::declarative::frame::ElementInstance::Text(_)
            | crate::declarative::frame::ElementInstance::StyledText(_)
            | crate::declarative::frame::ElementInstance::SelectableText(_) => {
                Ok(CleanGeometryNodeContract::stable_leaf())
            }
            crate::declarative::frame::ElementInstance::Spacer(_)
            | crate::declarative::frame::ElementInstance::Image(_)
            | crate::declarative::frame::ElementInstance::SvgIcon(_)
            | crate::declarative::frame::ElementInstance::SvgImage(_)
            | crate::declarative::frame::ElementInstance::Spinner(_)
            | crate::declarative::frame::ElementInstance::Scrollbar(_)
                if children.is_empty() =>
            {
                Ok(CleanGeometryNodeContract::propagated_leaf())
            }
            _ => Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::UnsupportedKind,
                kind,
            )),
        }
    }

    fn clean_flex_child_bounds_strategy(
        props: crate::element::FlexProps,
    ) -> CleanGeometryChildBoundsStrategy {
        match props.direction {
            fret_core::Axis::Vertical => {
                CleanGeometryChildBoundsStrategy::VerticalNoWrapFlex(props)
            }
            fret_core::Axis::Horizontal => {
                CleanGeometryChildBoundsStrategy::HorizontalFixedFlex(props)
            }
        }
    }

    fn clean_preserved_origin_width_delta_child_bounds_checked(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        children: &[NodeId],
        bounds: Rect,
        prev_bounds: Rect,
        element_kind: &'static str,
    ) -> Result<Vec<(NodeId, Rect)>, CleanGeometrySolveSkipRejection> {
        let mut out = Vec::with_capacity(children.len());
        for &child in children {
            let child_style = crate::declarative::frame::layout_style_for_node(app, window, child);
            let prev_child = self
                .nodes
                .get(child)
                .ok_or_else(|| {
                    CleanGeometrySolveSkipRejection::new(
                        CleanGeometrySolveSkipRejectionReason::MissingNode,
                    )
                    .at_node(child)
                })?
                .bounds;
            if child_style.position == crate::element::PositionStyle::Absolute {
                out.push((
                    child,
                    Self::clean_absolute_px_inset_child_bounds(
                        child_style,
                        bounds,
                        prev_child,
                        element_kind,
                    )?,
                ));
                continue;
            }
            if child_style.position != crate::element::PositionStyle::Static {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::PositionedChild,
                    element_kind,
                ));
            }
            if !Self::clean_margin_edges_are_px(child_style.margin) {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::NonPxMargin,
                    element_kind,
                ));
            }
            Self::clean_child_width_style_supported_for_width_delta(child_style, element_kind)?;
            Self::clean_child_height_style_supported_for_width_delta(child_style, element_kind)?;
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
        Ok(out)
    }

    fn clean_absolute_px_inset_child_bounds(
        child_style: crate::element::LayoutStyle,
        containing_bounds: Rect,
        prev_child: Rect,
        element_kind: &'static str,
    ) -> Result<Rect, CleanGeometrySolveSkipRejection> {
        if !Self::clean_margin_edges_are_zero_px(child_style.margin) {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxMargin,
                element_kind,
            ));
        }
        if child_style.aspect_ratio.is_some() {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::PositionedChild,
                element_kind,
            ));
        }

        let left = Self::clean_inset_edge_px_or_auto(child_style.inset.left, element_kind)?;
        let right = Self::clean_inset_edge_px_or_auto(child_style.inset.right, element_kind)?;
        let top = Self::clean_inset_edge_px_or_auto(child_style.inset.top, element_kind)?;
        let bottom = Self::clean_inset_edge_px_or_auto(child_style.inset.bottom, element_kind)?;

        let width = Self::clean_absolute_axis_size(
            child_style.size.width,
            left,
            right,
            containing_bounds.size.width,
            element_kind,
        )?;
        let height = Self::clean_absolute_axis_size(
            child_style.size.height,
            top,
            bottom,
            containing_bounds.size.height,
            element_kind,
        )?;
        Self::clean_absolute_axis_constraints_allow_size(
            child_style.size.min_width,
            child_style.size.max_width,
            width,
            element_kind,
        )?;
        Self::clean_absolute_axis_constraints_allow_size(
            child_style.size.min_height,
            child_style.size.max_height,
            height,
            element_kind,
        )?;
        let x = Self::clean_absolute_axis_origin(
            containing_bounds.origin.x,
            containing_bounds.size.width,
            width,
            left,
            right,
            element_kind,
        )?;
        let y = Self::clean_absolute_axis_origin(
            containing_bounds.origin.y,
            containing_bounds.size.height,
            height,
            top,
            bottom,
            element_kind,
        )?;

        if prev_child.size == Size::default() {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::MissingMeasuredSize,
                element_kind,
            ));
        }

        Ok(Rect::new(Point::new(x, y), Size::new(width, height)))
    }

    fn clean_inset_edge_px_or_auto(
        edge: crate::element::InsetEdge,
        element_kind: &'static str,
    ) -> Result<Option<f32>, CleanGeometrySolveSkipRejection> {
        match edge {
            crate::element::InsetEdge::Px(px) if px.0.is_finite() && px.0 >= -0.01 => {
                Ok(Some(px.0.max(0.0)))
            }
            crate::element::InsetEdge::Auto => Ok(None),
            crate::element::InsetEdge::Px(_)
            | crate::element::InsetEdge::Fill
            | crate::element::InsetEdge::Fraction(_) => {
                Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                    element_kind,
                ))
            }
        }
    }

    fn clean_absolute_axis_size(
        length: crate::element::Length,
        start: Option<f32>,
        end: Option<f32>,
        containing_size: Px,
        element_kind: &'static str,
    ) -> Result<Px, CleanGeometrySolveSkipRejection> {
        match length {
            crate::element::Length::Px(px) if px.0.is_finite() && px.0 >= -0.01 => {
                Ok(Px(px.0.max(0.0)))
            }
            crate::element::Length::Auto => {
                let (Some(start), Some(end)) = (start, end) else {
                    return Err(CleanGeometrySolveSkipRejection::for_kind(
                        CleanGeometrySolveSkipRejectionReason::PositionedChild,
                        element_kind,
                    ));
                };
                Ok(Px((containing_size.0 - start - end).max(0.0)))
            }
            crate::element::Length::Px(_)
            | crate::element::Length::Fill
            | crate::element::Length::Fraction(_) => {
                Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::FractionalChildSize,
                    element_kind,
                ))
            }
        }
    }

    fn clean_absolute_axis_constraints_allow_size(
        min: Option<crate::element::Length>,
        max: Option<crate::element::Length>,
        size: Px,
        element_kind: &'static str,
    ) -> Result<(), CleanGeometrySolveSkipRejection> {
        match min {
            Some(crate::element::Length::Px(px)) if size.0 + 0.01 < px.0.max(0.0) => {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::PositionedChild,
                    element_kind,
                ));
            }
            Some(crate::element::Length::Fill | crate::element::Length::Fraction(_)) => {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::FractionalChildSize,
                    element_kind,
                ));
            }
            Some(crate::element::Length::Auto | crate::element::Length::Px(_)) | None => {}
        }
        match max {
            Some(crate::element::Length::Px(px)) if size.0 - 0.01 > px.0.max(0.0) => {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::PositionedChild,
                    element_kind,
                ));
            }
            Some(crate::element::Length::Fill | crate::element::Length::Fraction(_)) => {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::FractionalChildSize,
                    element_kind,
                ));
            }
            Some(crate::element::Length::Auto | crate::element::Length::Px(_)) | None => {}
        }
        Ok(())
    }

    fn clean_absolute_axis_origin(
        containing_origin: Px,
        containing_size: Px,
        child_size: Px,
        start: Option<f32>,
        end: Option<f32>,
        element_kind: &'static str,
    ) -> Result<Px, CleanGeometrySolveSkipRejection> {
        if let Some(start) = start {
            return Ok(Px(containing_origin.0 + start));
        }
        if let Some(end) = end {
            return Ok(Px(containing_origin.0 + containing_size.0
                - child_size.0
                - end));
        }
        Err(CleanGeometrySolveSkipRejection::for_kind(
            CleanGeometrySolveSkipRejectionReason::PositionedChild,
            element_kind,
        ))
    }

    fn clean_container_width_delta_child_bounds(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        children: &[NodeId],
        bounds: Rect,
        prev_bounds: Rect,
        props: crate::element::ContainerProps,
        element_kind: &'static str,
    ) -> Result<Vec<(NodeId, Rect)>, CleanGeometrySolveSkipRejection> {
        if (bounds.size.height.0 - prev_bounds.size.height.0).abs() > 0.01 {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::ContainerHeightDelta,
                element_kind,
            ));
        }

        let (pad_left, pad_right, pad_top, _pad_bottom) =
            Self::clean_container_insets(props, element_kind)?;
        let pad_w = pad_left + pad_right;
        let prev_inner_width = (prev_bounds.size.width.0 - pad_w).max(0.0);
        let next_inner_width = (bounds.size.width.0 - pad_w).max(0.0);
        let prev_inner_origin = Point::new(
            Px(prev_bounds.origin.x.0 + pad_left),
            Px(prev_bounds.origin.y.0 + pad_top),
        );
        let next_inner_origin = Point::new(
            Px(bounds.origin.x.0 + pad_left),
            Px(bounds.origin.y.0 + pad_top),
        );

        let mut out = Vec::with_capacity(children.len());
        for &child in children {
            let child_style = crate::declarative::frame::layout_style_for_node(app, window, child);
            if child_style.position != crate::element::PositionStyle::Static {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::PositionedChild,
                    element_kind,
                ));
            }
            if !Self::clean_margin_edges_are_px(child_style.margin) {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::NonPxMargin,
                    element_kind,
                ));
            }
            Self::clean_child_height_style_supported_for_width_delta(child_style, element_kind)?;
            Self::clean_child_width_style_supported_for_width_delta(child_style, element_kind)?;

            let prev_child = self
                .nodes
                .get(child)
                .ok_or_else(|| {
                    CleanGeometrySolveSkipRejection::new(
                        CleanGeometrySolveSkipRejectionReason::MissingNode,
                    )
                    .at_node(child)
                })?
                .bounds;
            let local_x = prev_child.origin.x.0 - prev_inner_origin.x.0;
            let local_y = prev_child.origin.y.0 - prev_inner_origin.y.0;
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
                        Px(next_inner_origin.x.0 + local_x),
                        Px(next_inner_origin.y.0 + local_y),
                    ),
                    Size::new(width, prev_child.size.height),
                ),
            ));
        }

        Ok(out)
    }

    fn clean_vertical_flex_width_delta_child_bounds(
        &self,
        app: &mut H,
        window: AppWindowId,
        children: &[NodeId],
        bounds: Rect,
        prev_bounds: Rect,
        props: crate::element::FlexProps,
        element_kind: &'static str,
    ) -> Result<Vec<(NodeId, Rect)>, CleanGeometrySolveSkipRejection> {
        if props.wrap {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexWrap,
                element_kind,
            ));
        }
        if props.direction != fret_core::Axis::Vertical {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexDirection,
                element_kind,
            ));
        }
        if props.justify != crate::element::MainAlign::Start {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexMainAlign,
                element_kind,
            ));
        }
        if props.align != crate::element::CrossAlign::Stretch {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexCrossAlign,
                element_kind,
            ));
        }
        if (bounds.size.height.0 - prev_bounds.size.height.0).abs() > 0.01 {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexHeightDelta,
                element_kind,
            ));
        }
        let pad_left = Self::clean_spacing_px(props.padding.left).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let pad_right = Self::clean_spacing_px(props.padding.right).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let _pad_top = Self::clean_spacing_px(props.padding.top).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let _pad_bottom = Self::clean_spacing_px(props.padding.bottom).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let _gap = Self::clean_spacing_px(props.gap).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;

        let prev_inner_width = (prev_bounds.size.width.0 - pad_left - pad_right).max(0.0);
        let next_inner_width = (bounds.size.width.0 - pad_left - pad_right).max(0.0);
        let mut out = Vec::with_capacity(children.len());
        for &child in children {
            let child_style = crate::declarative::frame::layout_style_for_node(app, window, child);
            if child_style.position != crate::element::PositionStyle::Static {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::PositionedChild,
                    element_kind,
                ));
            }
            if !Self::clean_margin_edges_are_px(child_style.margin) {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::NonPxMargin,
                    element_kind,
                ));
            }
            Self::clean_child_height_style_supported_for_width_delta(child_style, element_kind)?;
            Self::clean_child_width_style_supported_for_width_delta(child_style, element_kind)?;
            let prev_child = self
                .nodes
                .get(child)
                .ok_or_else(|| {
                    CleanGeometrySolveSkipRejection::new(
                        CleanGeometrySolveSkipRejectionReason::MissingNode,
                    )
                    .at_node(child)
                })?
                .bounds;
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
        Ok(out)
    }

    fn clean_horizontal_fixed_flex_width_delta_child_bounds(
        &self,
        app: &mut H,
        window: AppWindowId,
        children: &[NodeId],
        bounds: Rect,
        prev_bounds: Rect,
        props: crate::element::FlexProps,
        element_kind: &'static str,
    ) -> Result<Vec<(NodeId, Rect)>, CleanGeometrySolveSkipRejection> {
        if props.wrap {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexWrap,
                element_kind,
            ));
        }
        if (bounds.size.height.0 - prev_bounds.size.height.0).abs() > 0.01 {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexHeightDelta,
                element_kind,
            ));
        }

        let pad_left = Self::clean_spacing_px(props.padding.left).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let pad_right = Self::clean_spacing_px(props.padding.right).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let pad_top = Self::clean_spacing_px(props.padding.top).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let pad_bottom = Self::clean_spacing_px(props.padding.bottom).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let _gap = Self::clean_spacing_px(props.gap).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;

        let prev_inner_width = (prev_bounds.size.width.0 - pad_left - pad_right).max(0.0);
        let next_inner_width = (bounds.size.width.0 - pad_left - pad_right).max(0.0);
        if props.justify != crate::element::MainAlign::Start
            && (next_inner_width - prev_inner_width).abs() > 0.01
        {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexMainAlign,
                element_kind,
            ));
        }
        let width_delta = next_inner_width - prev_inner_width;
        let next_inner_height = (bounds.size.height.0 - pad_top - pad_bottom).max(0.0);
        let mut out = Vec::with_capacity(children.len());
        let mut width_offset_after_flexible = 0.0;
        let mut flexible_children = 0usize;
        for &child in children {
            let child_style = crate::declarative::frame::layout_style_for_node(app, window, child);
            if child_style.position != crate::element::PositionStyle::Static {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::PositionedChild,
                    element_kind,
                ));
            }
            if !Self::clean_margin_edges_are_px(child_style.margin) {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::NonPxMargin,
                    element_kind,
                ));
            }
            Self::clean_child_height_style_supported_for_width_delta(child_style, element_kind)?;

            let prev_child = self
                .nodes
                .get(child)
                .ok_or_else(|| {
                    CleanGeometrySolveSkipRejection::new(
                        CleanGeometrySolveSkipRejectionReason::MissingNode,
                    )
                    .at_node(child)
                })?
                .bounds;
            let next_width = if width_delta.abs() <= 0.01 {
                Self::clean_horizontal_preserved_flex_item_supported(
                    child_style,
                    prev_child.size.width,
                    element_kind,
                )?;
                prev_child.size.width
            } else if let Some(next_width) =
                Self::clean_horizontal_flex_basis0_grow_item_next_width(
                    child_style,
                    prev_child.size.width,
                    width_delta,
                    element_kind,
                )?
            {
                flexible_children = flexible_children.saturating_add(1);
                if flexible_children > 1 {
                    return Err(CleanGeometrySolveSkipRejection::for_kind(
                        CleanGeometrySolveSkipRejectionReason::FlexItemSizing,
                        element_kind,
                    ));
                }
                width_offset_after_flexible += next_width.0 - prev_child.size.width.0;
                next_width
            } else {
                Self::clean_horizontal_fixed_flex_item_supported(
                    child_style,
                    prev_child.size.width,
                    element_kind,
                )?;
                prev_child.size.width
            };
            let local_x = prev_child.origin.x.0 - prev_bounds.origin.x.0;
            let local_y = prev_child.origin.y.0 - prev_bounds.origin.y.0;
            let x = bounds.origin.x.0 + local_x + width_offset_after_flexible
                - (next_width.0 - prev_child.size.width.0);
            let height = if (prev_child.size.height.0
                - (prev_bounds.size.height.0 - pad_top - pad_bottom).max(0.0))
            .abs()
                <= 0.01
                || matches!(child_style.size.height, crate::element::Length::Fill)
            {
                Px(next_inner_height)
            } else {
                prev_child.size.height
            };
            out.push((
                child,
                Rect::new(
                    Point::new(Px(x), Px(bounds.origin.y.0 + local_y)),
                    Size::new(next_width, height),
                ),
            ));
        }

        Ok(out)
    }

    fn clean_single_column_auto_rows_grid_width_delta_child_bounds(
        &self,
        app: &mut H,
        window: AppWindowId,
        children: &[NodeId],
        bounds: Rect,
        prev_bounds: Rect,
        props: crate::element::GridProps,
        element_kind: &'static str,
    ) -> Result<Vec<(NodeId, Rect)>, CleanGeometrySolveSkipRejection> {
        if (bounds.size.height.0 - prev_bounds.size.height.0).abs() > 0.01 {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::GridItemSizing,
                element_kind,
            ));
        }
        let Some(row_count) =
            Self::clean_grid_explicit_auto_or_px_track_count(props.template_rows.as_deref())
        else {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::GridTrackSizing,
                element_kind,
            ));
        };
        if props.cols != 1
            || props
                .template_columns
                .as_ref()
                .is_some_and(|tracks| !tracks.is_empty())
            || props.rows.is_some_and(|rows| rows as usize != row_count)
            || children.len() > row_count
        {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::GridTrackSizing,
                element_kind,
            ));
        }
        if props.justify != crate::element::MainAlign::Start
            || props.align != crate::element::CrossAlign::Start
            || props.justify_items.is_some()
        {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::GridItemSizing,
                element_kind,
            ));
        }

        let pad_left = Self::clean_spacing_px(props.padding.left).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let pad_right = Self::clean_spacing_px(props.padding.right).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let pad_top = Self::clean_spacing_px(props.padding.top).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let _pad_bottom = Self::clean_spacing_px(props.padding.bottom).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let _column_gap = Self::clean_spacing_px(props.resolved_column_gap()).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;
        let _row_gap = Self::clean_spacing_px(props.resolved_row_gap()).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })?;

        let prev_inner_width = (prev_bounds.size.width.0 - pad_left - pad_right).max(0.0);
        let next_inner_width = (bounds.size.width.0 - pad_left - pad_right).max(0.0);
        let prev_inner_origin = Point::new(
            Px(prev_bounds.origin.x.0 + pad_left),
            Px(prev_bounds.origin.y.0 + pad_top),
        );
        let next_inner_origin = Point::new(
            Px(bounds.origin.x.0 + pad_left),
            Px(bounds.origin.y.0 + pad_top),
        );

        let mut out = Vec::with_capacity(children.len());
        for &child in children {
            let child_style = crate::declarative::frame::layout_style_for_node(app, window, child);
            if child_style.position != crate::element::PositionStyle::Static {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::PositionedChild,
                    element_kind,
                ));
            }
            if !Self::clean_margin_edges_are_px(child_style.margin) {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::NonPxMargin,
                    element_kind,
                ));
            }
            if !Self::clean_grid_item_line_is_auto_or_single(child_style.grid.column)
                || !Self::clean_grid_item_line_is_auto_or_single(child_style.grid.row)
                || child_style.grid.align_self.is_some()
                || child_style.grid.justify_self.is_some()
            {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::GridItemSizing,
                    element_kind,
                ));
            }
            Self::clean_child_height_style_supported_for_width_delta(child_style, element_kind)?;
            Self::clean_child_width_style_supported_for_width_delta(child_style, element_kind)?;

            let prev_child = self
                .nodes
                .get(child)
                .ok_or_else(|| {
                    CleanGeometrySolveSkipRejection::new(
                        CleanGeometrySolveSkipRejectionReason::MissingNode,
                    )
                    .at_node(child)
                })?
                .bounds;
            let local_x = prev_child.origin.x.0 - prev_inner_origin.x.0;
            let local_y = prev_child.origin.y.0 - prev_inner_origin.y.0;
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
                        Px(next_inner_origin.x.0 + local_x),
                        Px(next_inner_origin.y.0 + local_y),
                    ),
                    Size::new(width, prev_child.size.height),
                ),
            ));
        }

        Ok(out)
    }

    fn clean_grid_explicit_auto_or_px_track_count(
        tracks: Option<&[crate::element::GridTrackSizing]>,
    ) -> Option<usize> {
        tracks.and_then(|tracks| {
            if tracks.is_empty() {
                return None;
            }
            tracks
                .iter()
                .all(|track| {
                    matches!(
                        track,
                        crate::element::GridTrackSizing::Auto
                            | crate::element::GridTrackSizing::Px(_)
                    )
                })
                .then_some(tracks.len())
        })
    }

    fn clean_grid_item_line_is_auto_or_single(line: crate::element::GridLine) -> bool {
        matches!(line.start, None | Some(1)) && matches!(line.span, None | Some(1))
    }

    fn clean_horizontal_flex_basis0_grow_item_next_width(
        child_style: crate::element::LayoutStyle,
        prev_width: Px,
        width_delta: f32,
        element_kind: &'static str,
    ) -> Result<Option<Px>, CleanGeometrySolveSkipRejection> {
        let grow = child_style.flex.grow;
        if grow.abs() <= 0.01 {
            return Ok(None);
        }
        if grow < -0.01 {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexItemSizing,
                element_kind,
            ));
        }
        if !grow.is_finite()
            || !child_style.flex.shrink.is_finite()
            || child_style.flex.shrink < -0.01
            || child_style.flex.order != 0
            || child_style.flex.align_self.is_some()
            || !matches!(
                child_style.flex.basis,
                crate::element::Length::Px(px) if px.0.abs() <= 0.01
            )
            || !matches!(
                child_style.size.width,
                crate::element::Length::Auto | crate::element::Length::Fill
            )
            || matches!(
                child_style.size.max_width,
                Some(crate::element::Length::Fill | crate::element::Length::Fraction(_))
            )
        {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexItemSizing,
                element_kind,
            ));
        }

        let min_width = match child_style.size.min_width {
            Some(crate::element::Length::Px(px)) => px.0.max(0.0),
            Some(crate::element::Length::Auto) | None => 0.0,
            Some(crate::element::Length::Fill | crate::element::Length::Fraction(_)) => {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::FractionalChildSize,
                    element_kind,
                ));
            }
        };
        let next_width = prev_width.0 + width_delta;
        if next_width + 0.01 < min_width {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexItemSizing,
                element_kind,
            ));
        }
        if let Some(crate::element::Length::Px(max_width)) = child_style.size.max_width
            && next_width - 0.01 > max_width.0.max(0.0)
        {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexItemSizing,
                element_kind,
            ));
        }

        Ok(Some(Px(next_width.max(min_width))))
    }

    fn clean_horizontal_preserved_flex_item_supported(
        child_style: crate::element::LayoutStyle,
        prev_width: Px,
        element_kind: &'static str,
    ) -> Result<(), CleanGeometrySolveSkipRejection> {
        if child_style.flex.order != 0
            || child_style.flex.grow.abs() > 0.01
            || !child_style.flex.shrink.is_finite()
            || child_style.flex.shrink < -0.01
            || !matches!(child_style.flex.basis, crate::element::Length::Auto)
            || child_style.flex.align_self.is_some()
        {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexItemSizing,
                element_kind,
            ));
        }
        if !matches!(
            child_style.size.width,
            crate::element::Length::Auto | crate::element::Length::Px(_)
        ) || !Self::clean_child_width_constraints_allow_preserved_width(child_style, prev_width)
        {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexItemSizing,
                element_kind,
            ));
        }

        Ok(())
    }

    fn clean_horizontal_fixed_flex_item_supported(
        child_style: crate::element::LayoutStyle,
        prev_width: Px,
        element_kind: &'static str,
    ) -> Result<(), CleanGeometrySolveSkipRejection> {
        if child_style.flex.order != 0
            || child_style.flex.grow.abs() > 0.01
            || child_style.flex.shrink.abs() > 0.01
            || !matches!(child_style.flex.basis, crate::element::Length::Auto)
            || child_style.flex.align_self.is_some()
        {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexItemSizing,
                element_kind,
            ));
        }
        if matches!(child_style.size.width, crate::element::Length::Auto)
            && Self::clean_child_width_constraints_allow_preserved_width(child_style, prev_width)
        {
            return Ok(());
        }
        if matches!(
            child_style.size.width,
            crate::element::Length::Auto
                | crate::element::Length::Fill
                | crate::element::Length::Fraction(_)
        ) {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FlexItemSizing,
                element_kind,
            ));
        }
        Self::clean_child_width_style_supported_for_width_delta(child_style, element_kind)
    }

    fn clean_child_width_constraints_allow_preserved_width(
        child_style: crate::element::LayoutStyle,
        prev_width: Px,
    ) -> bool {
        let min_ok = match child_style.size.min_width {
            Some(crate::element::Length::Px(px)) => prev_width.0 + 0.01 >= px.0.max(0.0),
            Some(crate::element::Length::Auto) | None => true,
            Some(crate::element::Length::Fill | crate::element::Length::Fraction(_)) => false,
        };
        let max_ok = match child_style.size.max_width {
            Some(crate::element::Length::Px(px)) => prev_width.0 - 0.01 <= px.0.max(0.0),
            Some(crate::element::Length::Auto) | None => true,
            Some(crate::element::Length::Fill | crate::element::Length::Fraction(_)) => false,
        };
        min_ok && max_ok
    }

    fn clean_child_width_style_supported_for_width_delta(
        child_style: crate::element::LayoutStyle,
        element_kind: &'static str,
    ) -> Result<(), CleanGeometrySolveSkipRejection> {
        if matches!(child_style.size.width, crate::element::Length::Fraction(_))
            || matches!(
                child_style.size.min_width,
                Some(crate::element::Length::Fill | crate::element::Length::Fraction(_))
            )
            || matches!(
                child_style.size.max_width,
                Some(crate::element::Length::Fill | crate::element::Length::Fraction(_))
            )
        {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::FractionalChildSize,
                element_kind,
            ));
        }

        Ok(())
    }

    fn clean_child_height_style_supported_for_width_delta(
        child_style: crate::element::LayoutStyle,
        element_kind: &'static str,
    ) -> Result<(), CleanGeometrySolveSkipRejection> {
        if !matches!(child_style.size.height, crate::element::Length::Auto) {
            return Ok(());
        }
        if child_style.aspect_ratio.is_some()
            || !Self::clean_optional_height_bound_stable_for_width_delta(
                child_style.size.min_height,
            )
            || !Self::clean_optional_height_bound_stable_for_width_delta(
                child_style.size.max_height,
            )
        {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::AutoChildHeight,
                element_kind,
            ));
        }

        Ok(())
    }

    fn clean_optional_height_bound_stable_for_width_delta(
        length: Option<crate::element::Length>,
    ) -> bool {
        matches!(
            length,
            None | Some(crate::element::Length::Auto | crate::element::Length::Px(_))
        )
    }

    fn clean_size_matches(a: Size, b: Size) -> bool {
        (a.width.0 - b.width.0).abs() <= 0.01 && (a.height.0 - b.height.0).abs() <= 0.01
    }

    fn clean_spacing_px(length: crate::element::SpacingLength) -> Option<f32> {
        match length {
            crate::element::SpacingLength::Px(px) => Some(px.0.max(0.0)),
            crate::element::SpacingLength::Fill | crate::element::SpacingLength::Fraction(_) => {
                None
            }
        }
    }

    fn clean_container_insets(
        props: crate::element::ContainerProps,
        element_kind: &'static str,
    ) -> Result<(f32, f32, f32, f32), CleanGeometrySolveSkipRejection> {
        let pad_left = Self::clean_spacing_px(props.padding.left).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })? + props.border.left.0.max(0.0);
        let pad_right = Self::clean_spacing_px(props.padding.right).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })? + props.border.right.0.max(0.0);
        let pad_top = Self::clean_spacing_px(props.padding.top).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })? + props.border.top.0.max(0.0);
        let pad_bottom = Self::clean_spacing_px(props.padding.bottom).ok_or_else(|| {
            CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::NonPxSpacing,
                element_kind,
            )
        })? + props.border.bottom.0.max(0.0);

        Ok((pad_left, pad_right, pad_top, pad_bottom))
    }

    fn clean_margin_edges_are_px(margin: crate::element::MarginEdges) -> bool {
        matches!(margin.left, crate::element::MarginEdge::Px(_))
            && matches!(margin.right, crate::element::MarginEdge::Px(_))
            && matches!(margin.top, crate::element::MarginEdge::Px(_))
            && matches!(margin.bottom, crate::element::MarginEdge::Px(_))
    }

    fn clean_margin_edges_are_zero_px(margin: crate::element::MarginEdges) -> bool {
        let is_zero = |edge: crate::element::MarginEdge| matches!(edge, crate::element::MarginEdge::Px(px) if px.0.abs() <= 0.01);
        is_zero(margin.left)
            && is_zero(margin.right)
            && is_zero(margin.top)
            && is_zero(margin.bottom)
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
            crate::declarative::frame::ElementInstance::Container(_) => true,
            crate::declarative::frame::ElementInstance::Grid(_) => true,
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

    fn clean_engine_geometry_propagation_requires_manual_child_bounds(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
    ) -> bool {
        crate::declarative::frame::with_element_record_for_node(app, window, node, |record| {
            matches!(
                record.instance,
                crate::declarative::frame::ElementInstance::Container(_)
                    | crate::declarative::frame::ElementInstance::Grid(_)
            )
        })
        .unwrap_or(false)
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
