use super::*;

#[track_caller]
pub(super) fn use_uncontrolled_model<T: Clone + 'static, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    default_value: impl FnOnce() -> T,
) -> Model<T> {
    cx.local_model(default_value)
}

pub(super) fn mouse_buttons_contains(
    buttons: fret_core::MouseButtons,
    button: MouseButton,
) -> bool {
    match button {
        MouseButton::Left => buttons.left,
        MouseButton::Right => buttons.right,
        MouseButton::Middle => buttons.middle,
        MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => false,
    }
}

pub(super) fn stable_hash_u64(seed: u64, value: &impl std::hash::Hash) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    seed.hash(&mut hasher);
    value.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn authoritative_surface_boundary_snapshot(
    graph_id: crate::core::GraphId,
    graph_rev: u64,
    view_state: &NodeGraphViewState,
) -> AuthoritativeSurfaceBoundarySnapshot {
    AuthoritativeSurfaceBoundarySnapshot {
        graph_id,
        graph_rev,
        selected_nodes_hash: stable_hash_u64(17, &view_state.selected_nodes),
        selected_edges_hash: stable_hash_u64(19, &view_state.selected_edges),
        selected_groups_hash: stable_hash_u64(23, &view_state.selected_groups),
    }
}

pub(super) fn read_authoritative_view_state_action_host<T>(
    host: &mut dyn fret_ui::action::UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    f: impl FnOnce(&NodeGraphViewState) -> T,
) -> Option<T> {
    read_authoritative_view_state_in_models(host.models_mut(), binding, f)
}

pub(super) fn read_authoritative_view_state_in_models<T>(
    models: &mut fret_runtime::ModelStore,
    binding: &NodeGraphSurfaceBinding,
    f: impl FnOnce(&NodeGraphViewState) -> T,
) -> Option<T> {
    let store = binding.store_model();
    models.read(&store, |store| f(store.view_state())).ok()
}

pub(super) fn read_authoritative_graph_in_models<T>(
    models: &mut fret_runtime::ModelStore,
    binding: &NodeGraphSurfaceBinding,
    f: impl FnOnce(&crate::core::Graph) -> T,
) -> Option<T> {
    let store = binding.store_model();
    models.read(&store, |store| f(store.graph())).ok()
}

pub(super) fn read_authoritative_interaction_config_in_models<T>(
    models: &mut fret_runtime::ModelStore,
    binding: &NodeGraphSurfaceBinding,
    f: impl FnOnce(&crate::io::NodeGraphInteractionConfig) -> T,
) -> Option<T> {
    let store = binding.store_model();
    models.read(&store, |store| f(store.interaction())).ok()
}

pub(super) fn read_authoritative_runtime_tuning_in_models<T>(
    models: &mut fret_runtime::ModelStore,
    binding: &NodeGraphSurfaceBinding,
    f: impl FnOnce(&crate::io::NodeGraphRuntimeTuning) -> T,
) -> Option<T> {
    let store = binding.store_model();
    models.read(&store, |store| f(store.runtime_tuning())).ok()
}

pub(super) fn read_authoritative_interaction_state_in_models<T>(
    models: &mut fret_runtime::ModelStore,
    binding: &NodeGraphSurfaceBinding,
    f: impl FnOnce(&crate::io::NodeGraphInteractionState) -> T,
) -> Option<T> {
    let store = binding.store_model();
    models
        .read(&store, |store| {
            let interaction = store.resolved_interaction_state();
            f(&interaction)
        })
        .ok()
}

pub(super) fn sync_authoritative_surface_boundary_in_models(
    models: &mut fret_runtime::ModelStore,
    boundary: &Model<Option<AuthoritativeSurfaceBoundarySnapshot>>,
    next: AuthoritativeSurfaceBoundarySnapshot,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    hovered_node: &Model<Option<crate::core::NodeId>>,
    hover_anchor_store: &Model<HoverAnchorStore>,
    portal_bounds_store: &Model<PortalBoundsStore>,
) -> bool {
    let previous = models.read(boundary, |state| *state).ok().flatten();
    let _ = models.update(boundary, |state| *state = Some(next));

    let Some(previous) = previous else {
        return false;
    };

    let graph_changed = previous.graph_id != next.graph_id || previous.graph_rev != next.graph_rev;
    let selection_changed = previous.selected_nodes_hash != next.selected_nodes_hash
        || previous.selected_edges_hash != next.selected_edges_hash
        || previous.selected_groups_hash != next.selected_groups_hash;

    if !graph_changed && !selection_changed {
        return false;
    }

    if graph_changed {
        let _ = models.update(drag, |state| *state = None);
    }

    if graph_changed || selection_changed {
        let _ = models.update(marquee, |state| *state = None);
        let _ = models.update(node_drag, |state| *state = None);
        let _ = models.update(pending_selection, |state| *state = None);
    }

    if graph_changed {
        let _ = models.update(hovered_node, |state| *state = None);
        let _ = models.update(hover_anchor_store, |state| {
            *state = HoverAnchorStore::default()
        });
        let _ = models.update(portal_bounds_store, |state| {
            state.nodes_canvas_bounds.clear();
            state.pending_fit_to_portals = false;
        });
    }

    true
}
