use fret_core::Rect;
use fret_runtime::Model;

use crate::core::Graph;
use crate::ui::MeasuredGeometryStore;
use crate::ui::canvas::{node_ports, node_size_default_px};
use crate::ui::measured::{MEASURED_GEOMETRY_EPSILON_PX, MeasuredGeometryBatch};
use crate::ui::style::NodeGraphStyle;

#[derive(Debug, Default, Clone)]
pub(super) struct PortalBoundsStore {
    /// Last-known bounds for portal node subtrees, mapped into canvas space under the current view.
    pub(super) nodes_canvas_bounds: std::collections::BTreeMap<crate::core::NodeId, Rect>,
    /// Counter for diagnostics gates (fit-to-portals triggered).
    pub(super) fit_to_portals_count: u64,
    /// Diagnostics-only: when true, a Ctrl+9 fit-to-portals request is armed and will be applied
    /// once portal bounds arrive via `LayoutQueryRegion` (frame-lagged by contract).
    pub(super) pending_fit_to_portals: bool,
}

#[derive(Debug, Default, Clone)]
pub(super) struct PortalMeasuredGeometryState {
    /// Frame-local pending subtree measurements harvested from `LayoutQueryRegion`.
    pub(super) pending_node_sizes_px: std::collections::BTreeMap<crate::core::NodeId, (f32, f32)>,
    /// Nodes previously published into the shared measured-geometry store.
    pub(super) published_nodes: Vec<crate::core::NodeId>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct PortalMeasuredGeometryFlushOutcome {
    pub(super) state_changed: bool,
    pub(super) store_changed: bool,
}

pub(super) fn sync_portal_canvas_bounds_in_models(
    models: &mut fret_runtime::ModelStore,
    portal_bounds_store: &Model<PortalBoundsStore>,
    node_id: crate::core::NodeId,
    canvas_bounds: Rect,
) -> bool {
    let should_update = models
        .read(portal_bounds_store, |st| {
            let Some(prev) = st.nodes_canvas_bounds.get(&node_id) else {
                return true;
            };
            !super::rect_approx_eq(*prev, canvas_bounds, 0.25)
        })
        .unwrap_or(true);

    if !should_update {
        return false;
    }

    let _ = models.update(portal_bounds_store, |st| {
        st.nodes_canvas_bounds.insert(node_id, canvas_bounds);
    });
    true
}

pub(super) fn record_portal_measured_node_size_in_state(
    models: &mut fret_runtime::ModelStore,
    state: &Model<PortalMeasuredGeometryState>,
    node_id: crate::core::NodeId,
    size_px: (f32, f32),
) -> bool {
    if !size_px.0.is_finite() || !size_px.1.is_finite() || size_px.0 <= 0.0 || size_px.1 <= 0.0 {
        return false;
    }

    let should_update = models
        .read(state, |st| {
            let Some(prev) = st.pending_node_sizes_px.get(&node_id) else {
                return true;
            };
            (prev.0 - size_px.0).abs() > MEASURED_GEOMETRY_EPSILON_PX
                || (prev.1 - size_px.1).abs() > MEASURED_GEOMETRY_EPSILON_PX
        })
        .unwrap_or(true);
    if !should_update {
        return false;
    }

    let _ = models.update(state, |st| {
        st.pending_node_sizes_px.insert(node_id, size_px);
    });
    true
}

pub(super) fn flush_portal_measured_geometry_state(
    graph: &Graph,
    style: &NodeGraphStyle,
    measured_geometry: &MeasuredGeometryStore,
    state: &mut PortalMeasuredGeometryState,
) -> PortalMeasuredGeometryFlushOutcome {
    let pending_before = state.pending_node_sizes_px.len();
    let published_before = state.published_nodes.clone();
    let graph_nodes: std::collections::BTreeSet<crate::core::NodeId> =
        graph.nodes.keys().copied().collect();

    let mut publish: Vec<(crate::core::NodeId, (f32, f32))> = Vec::new();
    for (node_id, measured_px) in state.pending_node_sizes_px.iter() {
        let Some(node) = graph.nodes.get(node_id) else {
            continue;
        };
        if node.size.is_some() {
            continue;
        }

        let (inputs, outputs) = node_ports(graph, *node_id);
        let min = node_size_default_px(inputs.len(), outputs.len(), style);
        let prev_px = measured_geometry.node_size_px(*node_id).unwrap_or(min);
        publish.push((
            *node_id,
            (
                measured_px.0.max(min.0).max(prev_px.0),
                measured_px.1.max(min.1).max(prev_px.1),
            ),
        ));
    }

    let remove_nodes: Vec<crate::core::NodeId> = published_before
        .iter()
        .copied()
        .filter(|id| !graph_nodes.contains(id))
        .collect();

    let store_changed = measured_geometry
        .apply_batch_if_changed(
            MeasuredGeometryBatch {
                node_sizes_px: publish.clone(),
                port_anchors_px: Vec::new(),
                remove_nodes,
                remove_ports: Vec::new(),
            },
            Default::default(),
        )
        .is_some();

    let mut next_published = published_before;
    for (node_id, _) in &publish {
        if !next_published.contains(node_id) {
            next_published.push(*node_id);
        }
    }
    next_published.retain(|id| graph_nodes.contains(id));

    state.pending_node_sizes_px.clear();
    let state_changed = pending_before > 0 || next_published != state.published_nodes;
    state.published_nodes = next_published;

    PortalMeasuredGeometryFlushOutcome {
        state_changed,
        store_changed,
    }
}
