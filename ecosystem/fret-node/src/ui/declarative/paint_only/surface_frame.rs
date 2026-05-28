use std::sync::Arc;

use fret_canvas::view::PanZoom2D;
use fret_canvas::wires as canvas_wires;
use fret_ui::{ElementContext, Invalidation, Theme, ThemeSnapshot, UiHost};

use crate::core::{EdgeId, NodeId, PortId};
use crate::ui::geometry_overrides::NodeGraphGeometryOverridesRef;
use crate::ui::paint_overrides::{NodeGraphPaintOverridesMap, NodeGraphPaintOverridesRef};
use crate::ui::presenter::{DefaultNodeGraphPresenter, NodeGraphPresenter};
use crate::ui::style::NodeGraphStyle;
use crate::ui::{
    MeasuredGeometryStore, NodeGraphCanvasTransform, NodeGraphEdgeTypesRef,
    NodeGraphInternalsSnapshot, NodeGraphSkinRef,
};

use super::surface_support::{
    read_authoritative_interaction_config_in_models, read_authoritative_runtime_tuning_in_models,
};
use super::{
    PaintOnlyInteractionFrameInputs, PaintOnlySurfaceModels, PortalBoundsStore,
    PortalMeasuredGeometryFlushOutcome, SurfaceSemanticsParams,
    authoritative_surface_boundary_snapshot, collect_edge_paint_diagnostics,
    collect_portal_diagnostics, declarative_presenter_revision,
    flush_portal_measured_geometry_state, plan_paint_only_interaction_frame,
    read_authoritative_graph_in_models, read_authoritative_view_state_in_models, stable_hash_u64,
    sync_authoritative_surface_boundary_in_models, sync_derived_cache, sync_edges_cache,
    sync_grid_cache, sync_nodes_cache, view_from_state,
};

#[derive(Clone)]
pub(super) struct PreparedPaintOnlySurfaceFrame {
    pub(super) view_for_paint: PanZoom2D,
    pub(super) theme: ThemeSnapshot,
    pub(super) style_tokens: NodeGraphStyle,
    pub(super) diagnostics: super::NodeGraphDiagnosticsConfig,
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

fn sync_binding_internals_for_surface(
    models: &mut fret_runtime::ModelStore,
    binding: &crate::ui::NodeGraphSurfaceBinding,
    bounds: fret_core::Rect,
    view: PanZoom2D,
    view_state: &crate::io::NodeGraphViewState,
    derived_cache: &super::DerivedGeometryCacheState,
    edges_cache: &super::EdgePaintCacheState,
    style_tokens: &NodeGraphStyle,
) {
    let Some(geom) = derived_cache.geom.as_deref() else {
        binding
            .internals_store()
            .update(NodeGraphInternalsSnapshot::default());
        return;
    };

    let transform = NodeGraphCanvasTransform {
        bounds_origin: bounds.origin,
        bounds_size: bounds.size,
        pan: crate::core::CanvasPoint {
            x: view.pan.x.0,
            y: view.pan.y.0,
        },
        zoom: view.zoom,
    };

    let keyboard_a11y_disabled =
        read_authoritative_interaction_config_in_models(models, binding, |config| {
            config.disable_keyboard_a11y
        })
        .unwrap_or(false);
    let focused_node = if keyboard_a11y_disabled {
        None
    } else {
        view_state
            .selected_nodes
            .iter()
            .copied()
            .find(|node| geom.nodes.contains_key(node))
    };
    let focused_edge = if keyboard_a11y_disabled {
        None
    } else {
        view_state.selected_edges.iter().copied().find(|edge| {
            edges_cache
                .draws
                .as_deref()
                .is_some_and(|draws| draws.iter().any(|draw| draw.edge == *edge))
        })
    };
    let focused_port = focused_node.and_then(|node| {
        geom.ports
            .iter()
            .find_map(|(&port, handle)| (handle.node == node).then_some(port))
    });

    let mut next = NodeGraphInternalsSnapshot {
        transform,
        focused_node,
        focused_port,
        focused_edge,
        ..NodeGraphInternalsSnapshot::default()
    };

    for (&node, node_geom) in &geom.nodes {
        next.nodes_window
            .insert(node, transform.canvas_rect_to_window(node_geom.rect));
    }
    for (&port, handle) in &geom.ports {
        next.ports_window
            .insert(port, transform.canvas_rect_to_window(handle.bounds));
        next.port_centers_window
            .insert(port, transform.canvas_point_to_window(handle.center));
    }

    sync_a11y_labels(
        models,
        binding,
        &mut next,
        focused_node,
        focused_port,
        focused_edge,
        style_tokens,
    );

    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    if let Some(edge_draws) = edges_cache.draws.as_deref() {
        for edge in edge_draws.iter() {
            if let (Some(from), Some(to)) = (geom.port_center(edge.from), geom.port_center(edge.to))
            {
                let (ctrl1, ctrl2) = canvas_wires::wire_ctrl_points(from, to, zoom);
                let center = canvas_wires::cubic_bezier(from, ctrl1, ctrl2, to, 0.5);
                next.edge_centers_window
                    .insert(edge.edge, transform.canvas_point_to_window(center));
            }
        }
    }

    next.a11y_active_descendant_label = next
        .a11y_focused_port_label
        .clone()
        .or_else(|| next.a11y_focused_edge_label.clone())
        .or_else(|| next.a11y_focused_node_label.clone());

    binding.internals_store().update(next);
}

fn sync_a11y_labels(
    models: &mut fret_runtime::ModelStore,
    binding: &crate::ui::NodeGraphSurfaceBinding,
    next: &mut NodeGraphInternalsSnapshot,
    focused_node: Option<NodeId>,
    focused_port: Option<PortId>,
    focused_edge: Option<EdgeId>,
    style_tokens: &NodeGraphStyle,
) {
    let labels = read_authoritative_graph_in_models(models, binding, |graph| {
        let presenter = DefaultNodeGraphPresenter::default();
        let node_label = focused_node
            .and_then(|node| presenter.a11y_node_label(graph, node))
            .map(|label| label.to_string())
            .or_else(|| focused_node.map(|node| format!("{node:?}")));
        let port_label = focused_port
            .and_then(|port| presenter.a11y_port_label(graph, port))
            .map(|label| label.to_string())
            .or_else(|| focused_port.map(|port| format!("{port:?}")));
        let edge_label = focused_edge
            .and_then(|edge| presenter.a11y_edge_label(graph, edge, style_tokens))
            .map(|label| label.to_string())
            .or_else(|| focused_edge.map(|edge| format!("{edge:?}")));

        (node_label, port_label, edge_label)
    })
    .unwrap_or_default();

    next.a11y_focused_node_label = labels.0.map(|label| format!("Node {label}"));
    next.a11y_focused_port_label = labels.1.map(|label| format!("Port {label}"));
    next.a11y_focused_edge_label = labels.2.map(|label| format!("Edge {label}"));
}

pub(super) struct PrepareSurfaceFrameParams<'a> {
    pub(super) binding: &'a crate::ui::NodeGraphSurfaceBinding,
    pub(super) surface_models: &'a PaintOnlySurfaceModels,
    pub(super) geometry_overrides: Option<NodeGraphGeometryOverridesRef>,
    pub(super) paint_overrides: Option<NodeGraphPaintOverridesRef>,
    pub(super) edge_types: Option<NodeGraphEdgeTypesRef>,
    pub(super) skin: Option<NodeGraphSkinRef>,
    pub(super) measured_geometry: Option<Arc<MeasuredGeometryStore>>,
    pub(super) diagnostics: super::NodeGraphDiagnosticsConfig,
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
        edge_types,
        skin,
        measured_geometry,
        diagnostics,
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

    let marquee_value = cx
        .get_model_cloned(marquee_drag, Invalidation::Layout)
        .unwrap_or(None);

    let node_drag_value = cx
        .get_model_cloned(node_drag, Invalidation::Layout)
        .unwrap_or(None);
    let pending_selection_value = cx
        .get_model_cloned(pending_selection, Invalidation::Layout)
        .unwrap_or(None);

    let view_for_paint = view_from_state(&view_value);
    let theme = Theme::global(&*cx.app).snapshot();
    let style_tokens = NodeGraphStyle::from_snapshot(theme.clone());
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
    let paint_overrides_ref = paint_overrides.or_else(|| {
        diagnostics
            .key_actions_enabled
            .then_some(diag_paint_overrides_ref)
    });
    let paint_overrides_rev = paint_overrides_ref
        .as_deref()
        .map(|overrides| overrides.revision())
        .unwrap_or(0);
    let edge_types_rev = edge_types
        .as_ref()
        .map(|edge_types| edge_types.revision())
        .unwrap_or(0);
    let skin_rev = skin.as_ref().map(|skin| skin.revision()).unwrap_or(0);

    let draw_order_hash = stable_hash_u64(2, &view_value.draw_order);
    let interaction_config =
        read_authoritative_interaction_config_in_models(cx.app.models_mut(), binding, Clone::clone)
            .unwrap_or_default();
    let runtime_tuning =
        read_authoritative_runtime_tuning_in_models(cx.app.models_mut(), binding, |state| *state)
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
        edge_types_rev,
        skin_rev,
        edge_types.as_deref(),
        skin.as_deref(),
    );
    let edges_cached = edges_cache_value.draws.is_some();

    sync_binding_internals_for_surface(
        cx.app.models_mut(),
        binding,
        grid_cache_value.bounds,
        view_for_paint,
        &view_value,
        &derived_cache_value,
        &edges_cache_value,
        &style_tokens,
    );

    let hovered_node_value = cx
        .get_model_copied(hovered_node, Invalidation::Paint)
        .unwrap_or(None);
    let interaction_plan = plan_paint_only_interaction_frame(PaintOnlyInteractionFrameInputs {
        view_state: &view_value,
        drag: drag_value,
        marquee: marquee_value.as_ref(),
        node_drag: node_drag_value.as_ref(),
        pending_selection: pending_selection_value.as_ref(),
        hovered_node: hovered_node_value,
    });
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
        panning: interaction_plan.panning,
        marquee_active: interaction_plan.marquee_active,
        node_drag_armed: interaction_plan.node_drag_armed,
        node_dragging: interaction_plan.node_dragging,
        hovered: interaction_plan.hovered,
        selected_nodes_len: interaction_plan.selected_nodes_len(),
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
        diagnostics,
        diag_paint_overrides_value,
        paint_overrides_ref,
        panning: interaction_plan.panning,
        marquee_active: interaction_plan.marquee_active,
        marquee_value,
        node_drag_value,
        node_dragging: interaction_plan.node_dragging,
        grid_cache_value,
        derived_cache_value,
        nodes_cache_value,
        edges_cache_value,
        hovered_node_value: interaction_plan.hovered_node,
        effective_selected_nodes: interaction_plan.effective_selected_nodes,
        portals_disabled,
        semantics_value,
        test_id,
    }
}
