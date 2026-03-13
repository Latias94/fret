use super::*;

pub(super) fn focus_next_edge<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    forward: bool,
) -> bool {
    let snapshot = canvas.sync_view_state(host);
    if !snapshot.interaction.elements_selectable
        || !snapshot.interaction.edges_selectable
        || !snapshot.interaction.edges_focusable
    {
        return false;
    }

    let mut edges: Vec<EdgeId> = canvas
        .graph
        .read_ref(host, |g| {
            g.edges
                .keys()
                .copied()
                .filter(|id| {
                    NodeGraphCanvasWith::<M>::edge_is_selectable(g, &snapshot.interaction, *id)
                })
                .collect()
        })
        .ok()
        .unwrap_or_default();
    if edges.is_empty() {
        return false;
    }
    edges.sort_unstable();

    let current = canvas
        .interaction
        .focused_edge
        .or_else(|| snapshot.selected_edges.first().copied());

    let next = match current.and_then(|id| edges.iter().position(|e| *e == id)) {
        Some(ix) => {
            let len = edges.len();
            let next_ix = if forward {
                (ix + 1) % len
            } else {
                (ix + len - 1) % len
            };
            edges[next_ix]
        }
        None => {
            if forward {
                edges[0]
            } else {
                edges[edges.len() - 1]
            }
        }
    };

    super::focus_session::focus_edge(&mut canvas.interaction, next);
    canvas.update_view_state(host, |s| {
        super::focus_session::select_only_edge(s, next);
    });
    true
}
