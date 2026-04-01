use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::element::CanvasProps;
use fret_ui::{ElementContext, Invalidation, ThemeSnapshot, UiHost};

use crate::core::{Graph, NodeId};
use crate::ui::paint_overrides::NodeGraphPaintOverridesRef;
use crate::ui::style::NodeGraphStyle;

use super::{
    HoverAnchorStore, HoverTooltipOverlayParams, MarqueeDragState, NodeDragState,
    PortalMeasuredGeometryState, apply_pending_fit_to_portals, host_visible_portal_labels,
    paint_debug_grid_cached, paint_edges_cached, paint_nodes_cached,
    push_hover_tooltip_overlay_if_needed, push_marquee_overlay_if_active,
    push_overlay_layer_if_needed, sync_hover_anchor_store_in_models,
};

pub(super) struct SurfaceRegionChildrenParams {
    pub(super) canvas: CanvasProps,
    pub(super) binding: crate::ui::NodeGraphSurfaceBinding,
    pub(super) graph: Model<Graph>,
    pub(super) hovered_node_model: Model<Option<NodeId>>,
    pub(super) node_drag_model: Model<Option<NodeDragState>>,
    pub(super) marquee_drag_model: Model<Option<MarqueeDragState>>,
    pub(super) hover_anchor_store: Model<HoverAnchorStore>,
    pub(super) portal_bounds_store: Model<super::PortalBoundsStore>,
    pub(super) portal_measured_geometry_state: Model<PortalMeasuredGeometryState>,
    pub(super) measured_geometry_present: bool,
    pub(super) portals_enabled: bool,
    pub(super) portals_disabled: bool,
    pub(super) portal_max_nodes: usize,
    pub(super) cull_margin_screen_px: f32,
    pub(super) min_zoom: f32,
    pub(super) max_zoom: f32,
    pub(super) diag_keys_enabled: bool,
    pub(super) panning: bool,
    pub(super) marquee_active: bool,
    pub(super) node_dragging: bool,
    pub(super) view_for_paint: fret_canvas::view::PanZoom2D,
    pub(super) grid_bounds: fret_core::Rect,
    pub(super) grid_ops: Option<Arc<Vec<fret_core::SceneOp>>>,
    pub(super) node_draws: Option<Arc<Vec<super::NodeRectDraw>>>,
    pub(super) edge_draws: Option<Arc<Vec<super::cache::EdgePathDraw>>>,
    pub(super) geom_for_paint: Option<Arc<crate::ui::canvas::CanvasGeometry>>,
    pub(super) style_tokens: NodeGraphStyle,
    pub(super) theme: ThemeSnapshot,
    pub(super) hovered_node_value: Option<NodeId>,
    pub(super) selected_nodes: Vec<NodeId>,
    pub(super) marquee_value: Option<MarqueeDragState>,
    pub(super) node_drag_value: Option<NodeDragState>,
    pub(super) paint_overrides_ref: Option<NodeGraphPaintOverridesRef>,
}

pub(super) fn build_surface_region_children<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    params: SurfaceRegionChildrenParams,
) -> Vec<AnyElement> {
    let SurfaceRegionChildrenParams {
        canvas,
        binding,
        graph,
        hovered_node_model,
        node_drag_model,
        marquee_drag_model,
        hover_anchor_store,
        portal_bounds_store,
        portal_measured_geometry_state,
        measured_geometry_present,
        portals_enabled,
        portals_disabled,
        portal_max_nodes,
        cull_margin_screen_px,
        min_zoom,
        max_zoom,
        diag_keys_enabled,
        panning,
        marquee_active,
        node_dragging,
        view_for_paint,
        grid_bounds,
        grid_ops,
        node_draws,
        edge_draws,
        geom_for_paint,
        style_tokens,
        theme,
        hovered_node_value,
        selected_nodes,
        marquee_value,
        node_drag_value,
        paint_overrides_ref,
    } = params;

    let node_draws_for_paint = node_draws.clone();
    let edge_draws_for_paint = edge_draws.clone();
    let style_tokens_for_paint = style_tokens.clone();
    let selected_nodes_for_paint = selected_nodes.clone();
    let node_drag_for_paint = node_drag_value.clone();
    let paint_overrides_for_paint = paint_overrides_ref.clone();

    let graph_model_id = graph.id();
    let view_state = binding.view_state_model();
    let view_state_model_id = view_state.id();
    let hovered_node_model_id = hovered_node_model.id();
    let node_drag_model_id = node_drag_model.id();
    let marquee_drag_model_id = marquee_drag_model.id();
    let canvas = cx.canvas(canvas, move |p| {
        p.observe_model_id(graph_model_id, Invalidation::Paint);
        p.observe_model_id(view_state_model_id, Invalidation::Paint);
        p.observe_model_id(hovered_node_model_id, Invalidation::Paint);
        p.observe_model_id(node_drag_model_id, Invalidation::Paint);
        p.observe_model_id(marquee_drag_model_id, Invalidation::Paint);

        paint_debug_grid_cached(p, view_for_paint, grid_ops.clone(), &style_tokens_for_paint);
        paint_nodes_cached(
            p,
            view_for_paint,
            cull_margin_screen_px,
            node_draws_for_paint.clone(),
            &style_tokens_for_paint,
            hovered_node_value,
            &selected_nodes_for_paint,
            node_drag_for_paint.as_ref(),
            paint_overrides_for_paint.as_deref(),
        );
        paint_edges_cached(
            p,
            view_for_paint,
            cull_margin_screen_px,
            edge_draws_for_paint.clone(),
            geom_for_paint.clone(),
            node_drag_for_paint.as_ref(),
            &style_tokens_for_paint,
            paint_overrides_for_paint.as_deref(),
        );
    });

    let mut out: Vec<AnyElement> = vec![canvas];
    let mut overlay_children: Vec<AnyElement> = Vec::new();
    let hovered_portal_hosted = if portals_enabled && !portals_disabled {
        host_visible_portal_labels(
            cx,
            &mut overlay_children,
            &graph,
            node_draws.as_ref(),
            grid_bounds,
            view_for_paint,
            cull_margin_screen_px,
            portal_max_nodes,
            hovered_node_value,
            &selected_nodes,
            node_drag_value.as_ref(),
            &portal_bounds_store,
            &portal_measured_geometry_state,
            measured_geometry_present,
            &style_tokens,
            &theme,
        )
    } else {
        false
    };

    if sync_hover_anchor_store_in_models(
        cx.app.models_mut(),
        &hover_anchor_store,
        hovered_node_value,
        node_draws.as_deref().map(|draws| draws.as_slice()),
        view_for_paint,
        node_drag_value.as_ref(),
    ) {
        cx.request_frame();
    }

    push_hover_tooltip_overlay_if_needed(
        cx,
        &mut overlay_children,
        HoverTooltipOverlayParams {
            graph: &graph,
            portal_bounds_store: &portal_bounds_store,
            hover_anchor_store: &hover_anchor_store,
            style_tokens: &style_tokens,
            diag_keys_enabled,
            panning,
            marquee_active,
            node_dragging,
            hovered_node: hovered_node_value,
            hovered_portal_hosted,
            portals_disabled,
            bounds: grid_bounds,
            view: view_for_paint,
        },
    );

    apply_pending_fit_to_portals(
        cx,
        &binding,
        &portal_bounds_store,
        portals_enabled,
        portals_disabled,
        grid_bounds,
        min_zoom,
        max_zoom,
    );

    push_marquee_overlay_if_active(
        cx,
        &mut overlay_children,
        marquee_value.as_ref(),
        grid_bounds,
        &style_tokens,
    );
    push_overlay_layer_if_needed(cx, &mut out, overlay_children);

    out
}
