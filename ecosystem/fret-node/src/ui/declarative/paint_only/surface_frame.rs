use std::sync::Arc;

use fret_canvas::view::PanZoom2D;
use fret_ui::{ElementContext, Invalidation, Theme, ThemeSnapshot, UiHost};

use crate::core::NodeId;
use crate::ui::MeasuredGeometryStore;
use crate::ui::geometry_overrides::NodeGraphGeometryOverridesRef;
use crate::ui::paint_overrides::{NodeGraphPaintOverridesMap, NodeGraphPaintOverridesRef};
use crate::ui::style::NodeGraphStyle;

use super::{
    PaintOnlySurfaceModels, PortalBoundsStore, PortalMeasuredGeometryFlushOutcome,
    SurfaceSemanticsParams, authoritative_surface_boundary_snapshot,
    collect_edge_paint_diagnostics, collect_portal_diagnostics, declarative_presenter_revision,
    effective_selected_nodes_for_paint, flush_portal_measured_geometry_state,
    read_authoritative_graph_in_models, read_authoritative_view_state_in_models, stable_hash_u64,
    sync_authoritative_surface_boundary_in_models, sync_derived_cache, sync_edges_cache,
    sync_grid_cache, sync_nodes_cache, view_from_state,
};
use super::surface_support::{
    read_authoritative_interaction_config_in_models, read_authoritative_runtime_tuning_in_models,
};

#[derive(Clone)]
pub(super) struct PreparedPaintOnlySurfaceFrame {
    pub(super) view_for_paint: PanZoom2D,
    pub(super) theme: ThemeSnapshot,
    pub(super) style_tokens: NodeGraphStyle,
    pub(super) diag_keys_enabled: bool,
    pub(super) diag_paint_overrides_value: Arc<NodeGraphPaintOverridesMap>,
    pub(super) paint_overrides_ref: Option<NodeGraphPaintOverridesRef>,
    pub(super) panning: bool,
    pub(super) marquee_value: Option<super::MarqueeDragState>,
    pub(super) marquee_active: bool,
    pub(super) node_drag_value: Option<super::NodeDragState>,
    pub(super) node_dragging: bool,
    pub(super) grid_cache_value: super::GridPaintCacheState,
    pub(super) derived_cache_value: super::DerivedGeometryCacheState,
    pub(super) nodes_cache_value: super::NodePaintCacheState,
    pub(super) edges_cache_value: super::EdgePaintCacheState,
    pub(super) hovered_node_value: Option<NodeId>,
    pub(super) effective_selected_nodes: Vec<NodeId>,
    pub(super) portals_disabled: bool,
    pub(super) semantics_value: Arc<str>,
    pub(super) test_id: Arc<str>,
}

pub(super) struct PrepareSurfaceFrameParams<'a> {
    pub(super) binding: &'a crate::ui::NodeGraphSurfaceBinding,
    pub(super) surface_models: &'a PaintOnlySurfaceModels,
    pub(super) geometry_overrides: Option<NodeGraphGeometryOverridesRef>,
    pub(super) paint_overrides: Option<NodeGraphPaintOverridesRef>,
    pub(super) measured_geometry: Option<Arc<MeasuredGeometryStore>>,
    pub(super) cull_margin_screen_px: f32,
    pub(super) test_id: Option<Arc<str>>,
}

pub(super) fn prepare_surface_frame<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    params: PrepareSurfaceFrameParams<'_>,
) -> PreparedPaintOnlySurfaceFrame {
    let PrepareSurfaceFrameParams {
        binding,
        surface_models,
        geometry_overrides,
        paint_overrides,
        measured_geometry,
        cull_margin_screen_px,
        test_id,
    } = params;
    let PaintOnlySurfaceModels {
        drag,
        marquee_drag,
        node_drag,
        pending_selection,
        hovered_node,
        hit_scratch: _,
        diag_paint_overrides,
        diag_paint_overrides_enabled: _,
        grid_cache,
        derived_cache,
        edges_cache,
        nodes_cache,
        portal_bounds_store,
        portal_measured_geometry_state,
        portal_debug_flags,
        hover_anchor_store,
        authoritative_surface_boundary,
    } = surface_models;

    cx.observe_model(&binding.store_model(), Invalidation::Layout);

    let view_value =
        read_authoritative_view_state_in_models(cx.app.models_mut(), binding, |state| {
            state.clone()
        })
        .unwrap_or_default();
    let graph_meta = cx
        .app
        .models()
        .read(&binding.store_model(), |store| {
            (store.graph_revision(), store.graph().graph_id)
        })
        .ok()
        .unwrap_or((0, crate::core::GraphId::from_u128(0)));
    let graph_rev = graph_meta.0;
    let graph_id = graph_meta.1;
    let authoritative_boundary =
        authoritative_surface_boundary_snapshot(graph_id, graph_rev, &view_value);
    let _ = sync_authoritative_surface_boundary_in_models(
        cx.app.models_mut(),
        authoritative_surface_boundary,
        authoritative_boundary,
        drag,
        marquee_drag,
        node_drag,
        pending_selection,
        hovered_node,
        hover_anchor_store,
        portal_bounds_store,
    );

    let drag_value = cx
        .get_model_copied(drag, Invalidation::Layout)
        .unwrap_or(None);
    let panning = drag_value.is_some();

    let marquee_value = cx
        .get_model_cloned(marquee_drag, Invalidation::Layout)
        .unwrap_or(None);
    let marquee_active = marquee_value.as_ref().is_some_and(|state| state.active);

    let node_drag_value = cx
        .get_model_cloned(node_drag, Invalidation::Layout)
        .unwrap_or(None);
    let node_drag_armed = node_drag_value
        .as_ref()
        .is_some_and(super::NodeDragState::is_armed);
    let node_dragging = node_drag_value
        .as_ref()
        .is_some_and(super::NodeDragState::is_active);
    let pending_selection_value = cx
        .get_model_cloned(pending_selection, Invalidation::Layout)
        .unwrap_or(None);

    let view_for_paint = view_from_state(&view_value);
    let theme = Theme::global(&*cx.app).snapshot();
    let style_tokens = NodeGraphStyle::from_snapshot(theme.clone());
    let diag_keys_enabled = std::env::var("FRET_DIAG")
        .ok()
        .is_some_and(|value| !value.trim().is_empty() && value.trim() != "0");
    let geometry_overrides = geometry_overrides.as_deref();
    let geometry_overrides_rev = geometry_overrides
        .map(|overrides| overrides.revision())
        .unwrap_or(0);
    let max_edge_interaction_width_override_px = geometry_overrides
        .map(|overrides| overrides.max_edge_interaction_width_override_px())
        .filter(|width| width.is_finite() && *width >= 0.0)
        .unwrap_or(0.0);
    let diag_paint_overrides_value = cx
        .get_model_cloned(diag_paint_overrides, Invalidation::Paint)
        .unwrap_or_else(|| Arc::new(NodeGraphPaintOverridesMap::default()));
    let diag_paint_overrides_ref: NodeGraphPaintOverridesRef = diag_paint_overrides_value.clone();
    let paint_overrides_ref =
        paint_overrides.or_else(|| diag_keys_enabled.then_some(diag_paint_overrides_ref));
    let paint_overrides_rev = paint_overrides_ref
        .as_deref()
        .map(|overrides| overrides.revision())
        .unwrap_or(0);

    let draw_order_hash = stable_hash_u64(2, &view_value.draw_order);
    let interaction_config = read_authoritative_interaction_config_in_models(
        cx.app.models_mut(),
        binding,
        Clone::clone,
    )
    .unwrap_or_default();
    let runtime_tuning = read_authoritative_runtime_tuning_in_models(
        cx.app.models_mut(),
        binding,
        |state| *state,
    )
    .unwrap_or_default();
    let interaction_state =
        crate::io::NodeGraphInteractionState::from_parts(&interaction_config, &runtime_tuning);
    let node_origin = interaction_config.node_origin;

    let mut portal_measured_geometry_state_value = cx
        .get_model_cloned(portal_measured_geometry_state, Invalidation::Paint)
        .unwrap_or_default();
    let portal_measured_flush_outcome = if let Some(measured_geometry) = measured_geometry.as_ref()
    {
        read_authoritative_graph_in_models(cx.app.models_mut(), binding, |graph_value| {
            flush_portal_measured_geometry_state(
                graph_value,
                &style_tokens,
                measured_geometry.as_ref(),
                &mut portal_measured_geometry_state_value,
            )
        })
        .unwrap_or_default()
    } else {
        PortalMeasuredGeometryFlushOutcome::default()
    };
    if portal_measured_flush_outcome.state_changed {
        let next_state = portal_measured_geometry_state_value.clone();
        let _ = cx
            .app
            .models_mut()
            .update(portal_measured_geometry_state, |state| *state = next_state);
    }
    if portal_measured_flush_outcome.store_changed {
        cx.request_frame();
    }
    let presenter_rev = declarative_presenter_revision(measured_geometry.as_ref());

    let grid_cache_value = sync_grid_cache(cx, grid_cache, view_for_paint, &style_tokens);
    let grid_cached = grid_cache_value.ops.is_some();

    let derived_cache_value = sync_derived_cache(
        cx,
        binding,
        derived_cache,
        graph_rev,
        view_for_paint,
        &view_value,
        &interaction_config,
        &interaction_state,
        runtime_tuning,
        &style_tokens,
        presenter_rev,
        measured_geometry.as_ref(),
        geometry_overrides,
        geometry_overrides_rev,
        max_edge_interaction_width_override_px,
    );
    let geom_cached = derived_cache_value.geom.is_some();

    let nodes_cache_value = sync_nodes_cache(
        cx,
        binding,
        nodes_cache,
        &derived_cache_value,
        graph_rev,
        view_for_paint,
        node_origin,
        draw_order_hash,
    );
    let nodes_cached = nodes_cache_value.draws.is_some();

    let edges_cache_value = sync_edges_cache(
        cx,
        binding,
        edges_cache,
        &derived_cache_value,
        graph_rev,
        view_for_paint,
        node_origin,
        draw_order_hash,
        &style_tokens,
    );
    let edges_cached = edges_cache_value.draws.is_some();

    let hovered_node_value = cx
        .get_model_copied(hovered_node, Invalidation::Paint)
        .unwrap_or(None);
    let hovered = hovered_node_value.is_some();
    let effective_selected_nodes = effective_selected_nodes_for_paint(
        &view_value,
        marquee_value.as_ref(),
        pending_selection_value.as_ref(),
    );
    let selected_nodes_len = effective_selected_nodes.len();
    let portals_disabled = cx
        .get_model_copied(portal_debug_flags, Invalidation::Paint)
        .unwrap_or_default()
        .disable_portals;
    let portal_diagnostics = cx
        .app
        .models()
        .read(portal_bounds_store, |state| {
            collect_portal_diagnostics(state, portals_disabled)
        })
        .unwrap_or_else(|_| {
            collect_portal_diagnostics(&PortalBoundsStore::default(), portals_disabled)
        });

    let edge_paint_diagnostics = collect_edge_paint_diagnostics(
        &edges_cache_value,
        &grid_cache_value,
        &derived_cache_value,
        &view_value,
        cull_margin_screen_px,
        node_drag_value.as_ref(),
    );
    let semantics_value = super::build_surface_semantics_value(SurfaceSemanticsParams {
        panning,
        marquee_active,
        node_drag_armed,
        node_dragging,
        hovered,
        selected_nodes_len,
        grid_cached,
        geom_cached,
        nodes_cached,
        edges_cached,
        grid_rebuilds: grid_cache_value.rebuilds,
        geom_rebuilds: derived_cache_value.rebuilds,
        nodes_rebuilds: nodes_cache_value.rebuilds,
        edges_rebuilds: edges_cache_value.rebuilds,
        edges: edge_paint_diagnostics,
        paint_overrides_rev,
        view_state: &view_value,
        portal: portal_diagnostics,
    });
    let test_id = test_id.unwrap_or_else(|| Arc::<str>::from("node_graph.canvas"));

    PreparedPaintOnlySurfaceFrame {
        view_for_paint,
        theme,
        style_tokens,
        diag_keys_enabled,
        diag_paint_overrides_value,
        paint_overrides_ref,
        panning,
        marquee_value,
        marquee_active,
        node_drag_value,
        node_dragging,
        grid_cache_value,
        derived_cache_value,
        nodes_cache_value,
        edges_cache_value,
        hovered_node_value,
        effective_selected_nodes,
        portals_disabled,
        semantics_value,
        test_id,
    }
}
