use super::*;

use crate::layout_pass::LayoutPassKind;
use crate::tree::UiDebugCleanGeometrySolveSkipRejection;
use fret_core::{TextAlign, TextOverflow, TextWrap};

#[derive(Debug, Clone, Copy)]
enum CleanGeometrySolveSkipDecision {
    Supported,
    Rejected(CleanGeometrySolveSkipRejection),
}

#[derive(Debug, Clone, Copy)]
struct CleanGeometrySolveSkipRejection {
    reason: CleanGeometrySolveSkipRejectionReason,
    detail: Option<CleanGeometrySolveSkipRejectionDetail>,
    node: Option<NodeId>,
    element_kind: Option<&'static str>,
}

impl CleanGeometrySolveSkipRejection {
    fn new(reason: CleanGeometrySolveSkipRejectionReason) -> Self {
        Self {
            reason,
            detail: None,
            node: None,
            element_kind: None,
        }
    }

    fn for_kind(reason: CleanGeometrySolveSkipRejectionReason, element_kind: &'static str) -> Self {
        Self {
            reason,
            detail: None,
            node: None,
            element_kind: Some(element_kind),
        }
    }

    fn with_detail(mut self, detail: CleanGeometrySolveSkipRejectionDetail) -> Self {
        self.detail = Some(detail);
        self
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

#[derive(Debug, Clone, Copy)]
enum CleanGeometrySolveSkipRejectionDetail {
    TextHeightDelta,
    TextWrapNotNone,
    TextOverflowNotClip,
    TextAlignNotStart,
    TextMissingWrapNoneMeasureCache,
    TextCachedSizeMismatch,
    TextFingerprintMismatch,
    TextUnsupportedInstance,
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
    /// `TextWrap::None` text with cached width-independent metrics may keep stable height while
    /// its parent-provided width changes.
    TextCachedMetrics,
}

impl CleanGeometryNodeContract {
    fn pure(child_bounds: CleanGeometryChildBoundsStrategy) -> Self {
        Self {
            layout_effect: CleanGeometryLayoutEffect::Pure,
            child_bounds,
            size_stability: CleanGeometryWidthDeltaSizeStability::Propagated,
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

impl CleanGeometrySolveSkipRejectionDetail {
    fn as_str(self) -> &'static str {
        match self {
            Self::TextHeightDelta => "text_height_delta",
            Self::TextWrapNotNone => "text_wrap_not_none",
            Self::TextOverflowNotClip => "text_overflow_not_clip",
            Self::TextAlignNotStart => "text_align_not_start",
            Self::TextMissingWrapNoneMeasureCache => "text_missing_wrap_none_measure_cache",
            Self::TextCachedSizeMismatch => "text_cached_size_mismatch",
            Self::TextFingerprintMismatch => "text_fingerprint_mismatch",
            Self::TextUnsupportedInstance => "text_unsupported_instance",
        }
    }
}

impl<H: UiHost> UiTree<H> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn try_propagate_clean_engine_layout(
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
            scale_factor,
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
            scale_factor,
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
        scale_factor: f32,
    ) -> bool {
        match self.clean_geometry_engine_solve_skip_decision(
            app,
            root,
            bounds,
            prev_bounds,
            scale_factor,
        ) {
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
        scale_factor: f32,
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
            scale_factor,
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
                .layout_clean_geometry_solve_skip_first_detail =
                rejection.detail.map(|detail| detail.as_str());
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
                detail: rejection.detail.map(|detail| detail.as_str()),
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
        scale_factor: f32,
        is_root: bool,
    ) -> Result<(), CleanGeometrySolveSkipRejection> {
        if self.clean_geometry_absent_interactivity_gate_leaf(app, window, node) {
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
            return Ok(());
        }
        if self.clean_geometry_explicit_zero_driver_leaf(app, window, node) {
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
            return Ok(());
        }
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
                scale_factor,
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
                scale_factor,
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
        scale_factor: f32,
    ) -> Option<Vec<(NodeId, Rect)>> {
        self.clean_manual_geometry_child_bounds_checked(
            app,
            window,
            node,
            children,
            bounds,
            prev_bounds,
            scale_factor,
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
        scale_factor: f32,
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
            CleanGeometryWidthDeltaSizeStability::TextCachedMetrics => {
                self.clean_text_cached_metrics_supported(
                    app,
                    window,
                    node,
                    bounds,
                    prev_bounds,
                    &record.instance,
                    kind,
                    scale_factor,
                )?;
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
            crate::declarative::frame::ElementInstance::InteractivityGate(props)
                if !props.present =>
            {
                Ok(CleanGeometryNodeContract::propagated_leaf())
            }
            crate::declarative::frame::ElementInstance::Scroll(_)
            | crate::declarative::frame::ElementInstance::TextInput(_)
            | crate::declarative::frame::ElementInstance::ViewCache(_) => {
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
                Ok(CleanGeometryNodeContract {
                    layout_effect: CleanGeometryLayoutEffect::Pure,
                    child_bounds: CleanGeometryChildBoundsStrategy::None,
                    size_stability: CleanGeometryWidthDeltaSizeStability::TextCachedMetrics,
                })
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

    fn clean_text_cached_metrics_supported(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        bounds: Rect,
        prev_bounds: Rect,
        instance: &crate::declarative::frame::ElementInstance,
        element_kind: &'static str,
        scale_factor: f32,
    ) -> Result<(), CleanGeometrySolveSkipRejection> {
        if Self::clean_size_matches(bounds.size, prev_bounds.size) {
            return Ok(());
        }
        if (bounds.size.height.0 - prev_bounds.size.height.0).abs() > 0.01 {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::TextReflow,
                element_kind,
            )
            .with_detail(CleanGeometrySolveSkipRejectionDetail::TextHeightDelta)
            .at_node(node));
        }
        let theme = crate::Theme::global(&*app).snapshot();
        let inherited_text_style =
            crate::declarative::frame::inherited_text_style_for_node(app, window, node);
        let font_stack_key = app
            .global::<fret_runtime::TextFontStackKey>()
            .map(|k| k.0)
            .unwrap_or(0);
        let fingerprint = match instance {
            crate::declarative::frame::ElementInstance::Text(props) => {
                if !matches!(props.overflow, TextOverflow::Clip | TextOverflow::Ellipsis) {
                    return Err(CleanGeometrySolveSkipRejection::for_kind(
                        CleanGeometrySolveSkipRejectionReason::TextReflow,
                        element_kind,
                    )
                    .with_detail(CleanGeometrySolveSkipRejectionDetail::TextOverflowNotClip)
                    .at_node(node));
                }
                if props.align != TextAlign::Start {
                    return Err(CleanGeometrySolveSkipRejection::for_kind(
                        CleanGeometrySolveSkipRejectionReason::TextReflow,
                        element_kind,
                    )
                    .with_detail(CleanGeometrySolveSkipRejectionDetail::TextAlignNotStart)
                    .at_node(node));
                }
                let resolved_style =
                    props.resolved_text_style_with_inherited(theme, inherited_text_style.as_ref());
                match props.wrap {
                    TextWrap::None => crate::text_props::text_wrap_none_measure_fingerprint_plain(
                        &props.text,
                        &resolved_style,
                        props.overflow,
                        props.align,
                        scale_factor,
                        font_stack_key,
                    ),
                    TextWrap::Word if props.overflow == TextOverflow::Clip => {
                        let fingerprint = crate::text_props::text_wrapped_measure_fingerprint_plain(
                            &props.text,
                            &resolved_style,
                            props.wrap,
                            props.overflow,
                            props.align,
                            scale_factor,
                            font_stack_key,
                        );
                        return self.clean_wrapped_text_cached_metrics_supported(
                            node,
                            bounds,
                            instance,
                            element_kind,
                            fingerprint,
                        );
                    }
                    _ => {
                        return Err(CleanGeometrySolveSkipRejection::for_kind(
                            CleanGeometrySolveSkipRejectionReason::TextReflow,
                            element_kind,
                        )
                        .with_detail(CleanGeometrySolveSkipRejectionDetail::TextWrapNotNone)
                        .at_node(node));
                    }
                }
            }
            crate::declarative::frame::ElementInstance::StyledText(props) => {
                if props.wrap != TextWrap::None {
                    return Err(CleanGeometrySolveSkipRejection::for_kind(
                        CleanGeometrySolveSkipRejectionReason::TextReflow,
                        element_kind,
                    )
                    .with_detail(CleanGeometrySolveSkipRejectionDetail::TextWrapNotNone)
                    .at_node(node));
                }
                if !matches!(props.overflow, TextOverflow::Clip | TextOverflow::Ellipsis) {
                    return Err(CleanGeometrySolveSkipRejection::for_kind(
                        CleanGeometrySolveSkipRejectionReason::TextReflow,
                        element_kind,
                    )
                    .with_detail(CleanGeometrySolveSkipRejectionDetail::TextOverflowNotClip)
                    .at_node(node));
                }
                if props.align != TextAlign::Start {
                    return Err(CleanGeometrySolveSkipRejection::for_kind(
                        CleanGeometrySolveSkipRejectionReason::TextReflow,
                        element_kind,
                    )
                    .with_detail(CleanGeometrySolveSkipRejectionDetail::TextAlignNotStart)
                    .at_node(node));
                }
                let resolved_style =
                    props.resolved_text_style_with_inherited(theme, inherited_text_style.as_ref());
                crate::text_props::text_wrap_none_measure_fingerprint_rich(
                    &props.rich,
                    &resolved_style,
                    props.overflow,
                    props.align,
                    scale_factor,
                    font_stack_key,
                )
            }
            crate::declarative::frame::ElementInstance::SelectableText(props) => {
                if props.wrap != TextWrap::None {
                    return Err(CleanGeometrySolveSkipRejection::for_kind(
                        CleanGeometrySolveSkipRejectionReason::TextReflow,
                        element_kind,
                    )
                    .with_detail(CleanGeometrySolveSkipRejectionDetail::TextWrapNotNone)
                    .at_node(node));
                }
                if !matches!(props.overflow, TextOverflow::Clip | TextOverflow::Ellipsis) {
                    return Err(CleanGeometrySolveSkipRejection::for_kind(
                        CleanGeometrySolveSkipRejectionReason::TextReflow,
                        element_kind,
                    )
                    .with_detail(CleanGeometrySolveSkipRejectionDetail::TextOverflowNotClip)
                    .at_node(node));
                }
                if props.align != TextAlign::Start {
                    return Err(CleanGeometrySolveSkipRejection::for_kind(
                        CleanGeometrySolveSkipRejectionReason::TextReflow,
                        element_kind,
                    )
                    .with_detail(CleanGeometrySolveSkipRejectionDetail::TextAlignNotStart)
                    .at_node(node));
                }
                let resolved_style =
                    props.resolved_text_style_with_inherited(theme, inherited_text_style.as_ref());
                crate::text_props::text_wrap_none_measure_fingerprint_rich(
                    &props.rich,
                    &resolved_style,
                    props.overflow,
                    props.align,
                    scale_factor,
                    font_stack_key,
                )
            }
            _ => {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::TextReflow,
                    element_kind,
                )
                .with_detail(CleanGeometrySolveSkipRejectionDetail::TextUnsupportedInstance)
                .at_node(node));
            }
        };
        let Some((cached_fingerprint, cached_size)) = self.node_text_wrap_none_measure_cache(node)
        else {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::TextReflow,
                element_kind,
            )
            .with_detail(CleanGeometrySolveSkipRejectionDetail::TextMissingWrapNoneMeasureCache)
            .at_node(node));
        };
        let expected_size = if matches!(
            instance,
            crate::declarative::frame::ElementInstance::Text(props)
                if props.overflow == TextOverflow::Ellipsis
        ) || matches!(
            instance,
            crate::declarative::frame::ElementInstance::StyledText(props)
                if props.overflow == TextOverflow::Ellipsis
        ) || matches!(
            instance,
            crate::declarative::frame::ElementInstance::SelectableText(props)
                if props.overflow == TextOverflow::Ellipsis
        ) {
            Size::new(bounds.size.width, cached_size.height)
        } else {
            cached_size
        };
        if !Self::clean_size_matches(expected_size, bounds.size) {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::TextReflow,
                element_kind,
            )
            .with_detail(CleanGeometrySolveSkipRejectionDetail::TextCachedSizeMismatch)
            .at_node(node));
        }
        if fingerprint != cached_fingerprint {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::TextReflow,
                element_kind,
            )
            .with_detail(CleanGeometrySolveSkipRejectionDetail::TextFingerprintMismatch)
            .at_node(node));
        }

        Ok(())
    }

    fn clean_wrapped_text_cached_metrics_supported(
        &self,
        node: NodeId,
        bounds: Rect,
        instance: &crate::declarative::frame::ElementInstance,
        element_kind: &'static str,
        fingerprint: u64,
    ) -> Result<(), CleanGeometrySolveSkipRejection> {
        let Some((cached_fingerprint, constraints_max_width, measured_size, clamped_size)) =
            self.node_text_wrapped_measure_cache(node)
        else {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::TextReflow,
                element_kind,
            )
            .with_detail(CleanGeometrySolveSkipRejectionDetail::TextMissingWrapNoneMeasureCache)
            .at_node(node));
        };
        if fingerprint != cached_fingerprint {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::TextReflow,
                element_kind,
            )
            .with_detail(CleanGeometrySolveSkipRejectionDetail::TextFingerprintMismatch)
            .at_node(node));
        }

        let next_max_width = match instance {
            crate::declarative::frame::ElementInstance::Text(props) => {
                match props.layout.size.width {
                    crate::element::Length::Fill => Some(bounds.size.width),
                    _ => constraints_max_width,
                }
            }
            _ => constraints_max_width,
        };

        let Some(next_max_width) = next_max_width else {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::TextReflow,
                element_kind,
            )
            .with_detail(CleanGeometrySolveSkipRejectionDetail::TextWrapNotNone)
            .at_node(node));
        };
        if measured_size.width.0 > next_max_width.0 + 0.01
            || next_max_width.0 > constraints_max_width.map(|w| w.0).unwrap_or(f32::INFINITY) + 0.01
        {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::TextReflow,
                element_kind,
            )
            .with_detail(CleanGeometrySolveSkipRejectionDetail::TextWrapNotNone)
            .at_node(node));
        }
        let expected_size = Size::new(bounds.size.width, clamped_size.height);
        if !Self::clean_size_matches(expected_size, bounds.size) {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::TextReflow,
                element_kind,
            )
            .with_detail(CleanGeometrySolveSkipRejectionDetail::TextCachedSizeMismatch)
            .at_node(node));
        }

        Ok(())
    }

    fn clean_geometry_absent_interactivity_gate_leaf(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
    ) -> bool {
        crate::declarative::frame::element_record_for_node(app, window, node).is_some_and(
            |record| {
                matches!(
                    record.instance,
                    crate::declarative::frame::ElementInstance::InteractivityGate(props)
                        if !props.present
                )
            },
        )
    }

    fn clean_geometry_explicit_zero_driver_leaf(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
    ) -> bool {
        let Some(entry) = self.nodes.get(node) else {
            return false;
        };
        if !entry.children.is_empty() {
            return false;
        }
        crate::declarative::frame::element_record_for_node(app, window, node).is_some_and(
            |record| match record.instance {
                crate::declarative::frame::ElementInstance::Spacer(props) => {
                    props.min.0.abs() <= 0.01
                        && Self::clean_explicit_zero_driver_layout_supported(props.layout)
                }
                crate::declarative::frame::ElementInstance::Container(props) => {
                    Self::clean_zero_driver_container_chrome_is_empty(props)
                        && Self::clean_explicit_zero_driver_layout_supported(props.layout)
                }
                _ => false,
            },
        )
    }

    fn clean_explicit_zero_driver_layout_supported(layout: crate::element::LayoutStyle) -> bool {
        layout.position == crate::element::PositionStyle::Static
            && layout.overflow == crate::element::Overflow::Visible
            && layout.inset == crate::element::InsetStyle::default()
            && layout.grid == crate::element::GridItemStyle::default()
            && layout.aspect_ratio.is_none()
            && Self::clean_margin_edges_are_zero_px(layout.margin)
            && Self::clean_length_is_zero_px(layout.size.width)
            && Self::clean_length_is_zero_px(layout.size.height)
            && Self::clean_optional_min_length_allows_zero(layout.size.min_width)
            && Self::clean_optional_min_length_allows_zero(layout.size.min_height)
            && Self::clean_optional_max_length_allows_zero(layout.size.max_width)
            && Self::clean_optional_max_length_allows_zero(layout.size.max_height)
            && layout.flex.order == 0
            && layout.flex.grow.abs() <= 0.01
            && layout.flex.align_self.is_none()
            && layout.flex.shrink >= -0.01
            && (Self::clean_length_is_zero_px(layout.flex.basis)
                || matches!(layout.flex.basis, crate::element::Length::Auto))
    }

    fn clean_zero_driver_container_chrome_is_empty(props: crate::element::ContainerProps) -> bool {
        Self::clean_spacing_edges_are_zero_px(props.padding)
            && props.background.is_none()
            && props.background_paint.is_none()
            && props.shadow.is_none()
            && Self::clean_edges_are_zero_px(props.border)
            && props.border_color.is_none()
            && props.border_paint.is_none()
            && props.border_dash.is_none()
            && props.focus_ring.is_none()
            && !props.focus_ring_always_paint
            && props.focus_border_color.is_none()
            && !props.focus_within
            && Self::clean_corners_are_zero_px(props.corner_radii)
            && !props.snap_to_device_pixels
    }

    fn clean_length_is_zero_px(length: crate::element::Length) -> bool {
        matches!(length, crate::element::Length::Px(px) if px.0.abs() <= 0.01)
    }

    fn clean_optional_min_length_allows_zero(length: Option<crate::element::Length>) -> bool {
        match length {
            Some(crate::element::Length::Px(px)) => px.0 <= 0.01,
            Some(crate::element::Length::Auto) | None => true,
            Some(crate::element::Length::Fill | crate::element::Length::Fraction(_)) => false,
        }
    }

    fn clean_optional_max_length_allows_zero(length: Option<crate::element::Length>) -> bool {
        match length {
            Some(crate::element::Length::Px(px)) => px.0 >= -0.01,
            Some(crate::element::Length::Auto) | None => true,
            Some(crate::element::Length::Fill | crate::element::Length::Fraction(_)) => false,
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
                let allow_zero_measured_size =
                    self.clean_geometry_absent_interactivity_gate_leaf(app, window, child);
                out.push((
                    child,
                    Self::clean_absolute_px_inset_child_bounds(
                        child_style,
                        bounds,
                        prev_child,
                        element_kind,
                        allow_zero_measured_size,
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
        allow_zero_measured_size: bool,
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

        let computed_size = Size::new(width, height);
        if prev_child.size == Size::default()
            && !(allow_zero_measured_size && computed_size == Size::default())
        {
            return Err(CleanGeometrySolveSkipRejection::for_kind(
                CleanGeometrySolveSkipRejectionReason::MissingMeasuredSize,
                element_kind,
            ));
        }

        Ok(Rect::new(Point::new(x, y), computed_size))
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
        let align = props.align;
        if !matches!(
            align,
            crate::element::CrossAlign::Stretch | crate::element::CrossAlign::Start
        ) {
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
            let horizontal_auto_margin =
                Self::clean_vertical_flex_horizontal_auto_margin_centered(child_style.margin);
            if !Self::clean_margin_edges_are_px(child_style.margin) && !horizontal_auto_margin {
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
            let width = match align {
                crate::element::CrossAlign::Stretch => {
                    if horizontal_auto_margin {
                        Self::clean_vertical_flex_horizontal_auto_margin_child_width(
                            child_style,
                            next_inner_width,
                            element_kind,
                        )?
                    } else if (prev_child.size.width.0 - prev_inner_width).abs() <= 0.01
                        || matches!(child_style.size.width, crate::element::Length::Fill)
                    {
                        Px(next_inner_width)
                    } else {
                        prev_child.size.width
                    }
                }
                crate::element::CrossAlign::Start => {
                    if horizontal_auto_margin {
                        Self::clean_vertical_flex_horizontal_auto_margin_child_width(
                            child_style,
                            next_inner_width,
                            element_kind,
                        )?
                    } else {
                        prev_child.size.width
                    }
                }
                _ => unreachable!("unsupported vertical flex cross-axis alignment rejected above"),
            };
            let origin_x = if horizontal_auto_margin {
                Px(bounds.origin.x.0 + pad_left + (next_inner_width - width.0).max(0.0) * 0.5)
            } else {
                Px(bounds.origin.x.0 + local_x)
            };
            out.push((
                child,
                Rect::new(
                    Point::new(origin_x, Px(bounds.origin.y.0 + local_y)),
                    Size::new(width, prev_child.size.height),
                ),
            ));
        }
        Ok(out)
    }

    fn clean_vertical_flex_horizontal_auto_margin_centered(
        margin: crate::element::MarginEdges,
    ) -> bool {
        matches!(margin.left, crate::element::MarginEdge::Auto)
            && matches!(margin.right, crate::element::MarginEdge::Auto)
            && matches!(margin.top, crate::element::MarginEdge::Px(_))
            && matches!(margin.bottom, crate::element::MarginEdge::Px(_))
    }

    fn clean_vertical_flex_horizontal_auto_margin_child_width(
        child_style: crate::element::LayoutStyle,
        next_inner_width: f32,
        element_kind: &'static str,
    ) -> Result<Px, CleanGeometrySolveSkipRejection> {
        let mut width = match child_style.size.width {
            crate::element::Length::Fill => next_inner_width,
            crate::element::Length::Px(px) => px.0.max(0.0),
            crate::element::Length::Auto => {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::NonPxMargin,
                    element_kind,
                ));
            }
            crate::element::Length::Fraction(_) => {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::FractionalChildSize,
                    element_kind,
                ));
            }
        };

        match child_style.size.min_width {
            Some(crate::element::Length::Px(px)) => {
                width = width.max(px.0.max(0.0));
            }
            Some(crate::element::Length::Auto) | None => {}
            Some(crate::element::Length::Fill | crate::element::Length::Fraction(_)) => {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::FractionalChildSize,
                    element_kind,
                ));
            }
        }

        match child_style.size.max_width {
            Some(crate::element::Length::Px(px)) => {
                width = width.min(px.0.max(0.0));
            }
            Some(crate::element::Length::Auto) | None => {}
            Some(crate::element::Length::Fill | crate::element::Length::Fraction(_)) => {
                return Err(CleanGeometrySolveSkipRejection::for_kind(
                    CleanGeometrySolveSkipRejectionReason::FractionalChildSize,
                    element_kind,
                ));
            }
        }

        Ok(Px(width.max(0.0)))
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

    fn clean_spacing_edges_are_zero_px(spacing: crate::element::SpacingEdges) -> bool {
        let is_zero = |edge: crate::element::SpacingLength| matches!(edge, crate::element::SpacingLength::Px(px) if px.0.abs() <= 0.01);
        is_zero(spacing.left)
            && is_zero(spacing.right)
            && is_zero(spacing.top)
            && is_zero(spacing.bottom)
    }

    fn clean_edges_are_zero_px(edges: fret_core::Edges) -> bool {
        edges.left.0.abs() <= 0.01
            && edges.right.0.abs() <= 0.01
            && edges.top.0.abs() <= 0.01
            && edges.bottom.0.abs() <= 0.01
    }

    fn clean_corners_are_zero_px(corners: fret_core::Corners) -> bool {
        corners.top_left.0.abs() <= 0.01
            && corners.top_right.0.abs() <= 0.01
            && corners.bottom_right.0.abs() <= 0.01
            && corners.bottom_left.0.abs() <= 0.01
    }

    fn clean_engine_geometry_propagation_supported_element(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        children: &[NodeId],
        bounds: Rect,
        prev_bounds: Rect,
        scale_factor: f32,
    ) -> Option<GlobalElementId> {
        let Some(record) = crate::declarative::frame::element_record_for_node(app, window, node)
        else {
            return None;
        };

        let supported =
            match &record.instance {
                crate::declarative::frame::ElementInstance::Stack(_)
                | crate::declarative::frame::ElementInstance::Pressable(_)
                | crate::declarative::frame::ElementInstance::Semantics(_) => true,
                crate::declarative::frame::ElementInstance::Container(_) => true,
                crate::declarative::frame::ElementInstance::Grid(_) => true,
                crate::declarative::frame::ElementInstance::Flex(props) => {
                    self.clean_engine_geometry_flex_margin_supported(app, window, children, *props)
                }
                crate::declarative::frame::ElementInstance::SemanticFlex(props) => self
                    .clean_engine_geometry_flex_margin_supported(app, window, children, props.flex),
                crate::declarative::frame::ElementInstance::RovingFlex(props) => self
                    .clean_engine_geometry_flex_margin_supported(app, window, children, props.flex),
                crate::declarative::frame::ElementInstance::Spacer(_) => children.is_empty(),
                crate::declarative::frame::ElementInstance::Text(_)
                | crate::declarative::frame::ElementInstance::StyledText(_)
                | crate::declarative::frame::ElementInstance::SelectableText(_) => {
                    children.is_empty()
                        && self
                            .clean_text_cached_metrics_supported(
                                app,
                                window,
                                node,
                                bounds,
                                prev_bounds,
                                &record.instance,
                                record.instance.kind_name(),
                                scale_factor,
                            )
                            .is_ok()
                }
                _ => false,
            };
        supported.then_some(record.element)
    }

    fn clean_engine_geometry_flex_margin_supported(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        children: &[NodeId],
        props: crate::element::FlexProps,
    ) -> bool {
        children.iter().copied().all(|child| {
            let style = crate::declarative::frame::layout_style_for_node(app, window, child);
            Self::clean_margin_edges_are_px(style.margin)
                || (props.direction == fret_core::Axis::Vertical
                    && Self::clean_vertical_flex_horizontal_auto_margin_centered(style.margin))
        })
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
}
