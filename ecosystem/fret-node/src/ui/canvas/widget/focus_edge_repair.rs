use super::*;

pub(super) fn repair_focused_edge_after_graph_change<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    preferred: Option<EdgeId>,
) {
    if preferred.is_none() && canvas.interaction.focused_edge.is_none() {
        return;
    }

    let snapshot = canvas.sync_view_state(host);
    if !snapshot.interaction.edges_focusable && !snapshot.interaction.edges_reconnectable {
        canvas.interaction.focused_edge = None;
        return;
    }

    let (edges, current_valid) = canvas
        .graph
        .read_ref(host, |g| {
            let mut edges: Vec<EdgeId> = g.edges.keys().copied().collect();
            edges.sort_unstable();
            let current = canvas.interaction.focused_edge;
            let current_valid = current.is_some_and(|id| g.edges.contains_key(&id));
            (edges, current_valid)
        })
        .ok()
        .unwrap_or_default();

    if edges.is_empty() {
        canvas.interaction.focused_edge = None;
        return;
    }

    if current_valid {
        return;
    }

    let base = preferred.or(canvas.interaction.focused_edge);
    let next = match base {
        Some(id) => match edges.binary_search(&id) {
            Ok(ix) => edges.get(ix).copied(),
            Err(ix) => edges.get(ix).copied().or_else(|| edges.first().copied()),
        },
        None => edges.first().copied(),
    };
    canvas.interaction.focused_edge = next;
}
