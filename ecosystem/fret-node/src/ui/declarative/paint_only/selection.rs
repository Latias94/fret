use super::*;

fn update_selection_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    f: impl FnOnce(
        &mut Vec<crate::core::NodeId>,
        &mut Vec<crate::core::EdgeId>,
        &mut Vec<crate::core::GroupId>,
    ),
) -> bool {
    let view_state = binding.view_state_model();
    let Ok(state) = host.models_mut().read(&view_state, |state| state.clone()) else {
        return false;
    };
    let mut selected_nodes = state.selected_nodes;
    let mut selected_edges = state.selected_edges;
    let mut selected_groups = state.selected_groups;
    f(
        &mut selected_nodes,
        &mut selected_edges,
        &mut selected_groups,
    );

    binding
        .set_selection_action_host(host, selected_nodes, selected_edges, selected_groups)
        .is_ok()
}

fn compute_marquee_candidate_nodes(
    rect_canvas: Rect,
    selection_mode: crate::io::NodeGraphSelectionMode,
    geom: &CanvasGeometry,
    index: &CanvasSpatialDerived,
) -> Vec<crate::core::NodeId> {
    let mut candidates = Vec::<crate::core::NodeId>::new();
    index.query_nodes_in_rect(rect_canvas, &mut candidates);
    candidates.retain(|id| {
        let Some(node) = geom.nodes.get(id) else {
            return false;
        };
        match selection_mode {
            crate::io::NodeGraphSelectionMode::Full => rect_contains_rect(rect_canvas, node.rect),
            crate::io::NodeGraphSelectionMode::Partial => rects_intersect(rect_canvas, node.rect),
        }
    });
    candidates.sort();
    candidates.dedup();
    candidates
}

pub(super) fn build_marquee_preview_selected_nodes(
    marquee: &MarqueeDragState,
    rect_canvas: Rect,
    selection_mode: crate::io::NodeGraphSelectionMode,
    geom: &CanvasGeometry,
    index: &CanvasSpatialDerived,
) -> Arc<[crate::core::NodeId]> {
    let candidates = compute_marquee_candidate_nodes(rect_canvas, selection_mode, geom, index);
    if !marquee.toggle {
        return Arc::from(candidates.into_boxed_slice());
    }

    let mut selected_nodes = marquee.base_selected_nodes.to_vec();
    for id in candidates {
        if let Some(ix) = selected_nodes.iter().position(|value| *value == id) {
            selected_nodes.remove(ix);
        } else {
            selected_nodes.push(id);
        }
    }
    selected_nodes.sort();
    selected_nodes.dedup();
    Arc::from(selected_nodes.into_boxed_slice())
}

/// Resolves the effective node selection for paint/layout only.
///
/// Boundary contract:
/// - committed selection lives in `NodeGraphViewState`,
/// - local preview state may temporarily override it,
/// - precedence is: active marquee preview > pending click-selection preview > committed selection.
pub(super) fn effective_selected_nodes_for_paint(
    view_state: &NodeGraphViewState,
    marquee: Option<&MarqueeDragState>,
    pending_selection: Option<&PendingSelectionState>,
) -> Vec<crate::core::NodeId> {
    marquee
        .filter(|marquee| marquee.active)
        .map(|marquee| marquee.preview_selected_nodes.to_vec())
        .or_else(|| pending_selection.map(|pending| pending.nodes.to_vec()))
        .unwrap_or_else(|| view_state.selected_nodes.clone())
}

pub(super) fn build_click_selection_preview_nodes(
    base_selected_nodes: &[crate::core::NodeId],
    hit: crate::core::NodeId,
    multi: bool,
) -> Arc<[crate::core::NodeId]> {
    let mut selected_nodes = base_selected_nodes.to_vec();
    let already_selected = selected_nodes.contains(&hit);
    if multi {
        if let Some(ix) = selected_nodes.iter().position(|id| *id == hit) {
            selected_nodes.remove(ix);
        } else {
            selected_nodes.push(hit);
        }
    } else if !already_selected {
        selected_nodes.clear();
        selected_nodes.push(hit);
    }
    selected_nodes.sort();
    selected_nodes.dedup();
    Arc::from(selected_nodes.into_boxed_slice())
}

pub(super) fn commit_pending_selection_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    pending: &PendingSelectionState,
) -> bool {
    let nodes = pending.nodes.clone();
    let clear_edges = pending.clear_edges;
    let clear_groups = pending.clear_groups;
    update_selection_action_host(
        host,
        binding,
        move |selected_nodes, selected_edges, selected_groups| {
            selected_nodes.clear();
            selected_nodes.extend(nodes.iter().copied());
            if clear_edges {
                selected_edges.clear();
            }
            if clear_groups {
                selected_groups.clear();
            }
        },
    )
}

pub(super) fn commit_marquee_selection_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    binding: &NodeGraphSurfaceBinding,
    marquee: &MarqueeDragState,
) -> bool {
    let pending = PendingSelectionState {
        nodes: marquee.preview_selected_nodes.clone(),
        clear_edges: !marquee.toggle,
        clear_groups: !marquee.toggle,
    };
    commit_pending_selection_action_host(host, binding, &pending)
}
