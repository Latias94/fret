use super::*;

pub(super) struct PaintOnlySurfaceModels {
    pub(super) drag: Model<Option<DragState>>,
    pub(super) marquee_drag: Model<Option<MarqueeDragState>>,
    pub(super) node_drag: Model<Option<NodeDragState>>,
    pub(super) pending_selection: Model<Option<PendingSelectionState>>,
    pub(super) hovered_node: Model<Option<crate::core::NodeId>>,
    pub(super) hit_scratch: Model<Vec<crate::core::NodeId>>,
    pub(super) diag_paint_overrides: Model<Arc<NodeGraphPaintOverridesMap>>,
    pub(super) diag_paint_overrides_enabled: Model<bool>,
    pub(super) grid_cache: Model<GridPaintCacheState>,
    pub(super) derived_cache: Model<DerivedGeometryCacheState>,
    pub(super) edges_cache: Model<EdgePaintCacheState>,
    pub(super) nodes_cache: Model<NodePaintCacheState>,
    pub(super) portal_bounds_store: Model<PortalBoundsStore>,
    pub(super) portal_measured_geometry_state: Model<PortalMeasuredGeometryState>,
    pub(super) portal_debug_flags: Model<PortalDebugFlags>,
    pub(super) hover_anchor_store: Model<HoverAnchorStore>,
    pub(super) authoritative_surface_boundary: Model<Option<AuthoritativeSurfaceBoundarySnapshot>>,
}

pub(super) fn use_paint_only_surface_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> PaintOnlySurfaceModels {
    PaintOnlySurfaceModels {
        drag: use_uncontrolled_model(cx, || None),
        marquee_drag: use_uncontrolled_model(cx, || None),
        node_drag: use_uncontrolled_model(cx, || None),
        pending_selection: use_uncontrolled_model(cx, || None),
        hovered_node: use_uncontrolled_model(cx, || None),
        hit_scratch: use_uncontrolled_model(cx, Vec::new),
        diag_paint_overrides: use_uncontrolled_model(cx, || {
            Arc::new(NodeGraphPaintOverridesMap::default())
        }),
        diag_paint_overrides_enabled: use_uncontrolled_model(cx, || false),
        grid_cache: use_uncontrolled_model(cx, GridPaintCacheState::default),
        derived_cache: use_uncontrolled_model(cx, DerivedGeometryCacheState::default),
        edges_cache: use_uncontrolled_model(cx, EdgePaintCacheState::default),
        nodes_cache: use_uncontrolled_model(cx, NodePaintCacheState::default),
        portal_bounds_store: use_uncontrolled_model(cx, PortalBoundsStore::default),
        portal_measured_geometry_state: use_uncontrolled_model(
            cx,
            PortalMeasuredGeometryState::default,
        ),
        portal_debug_flags: use_uncontrolled_model(cx, PortalDebugFlags::default),
        hover_anchor_store: use_uncontrolled_model(cx, HoverAnchorStore::default),
        authoritative_surface_boundary: use_uncontrolled_model(cx, || None),
    }
}
