use super::super::*;

pub(super) fn target_edge_reconnectable_flags<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &H,
    snapshot: &ViewSnapshot,
    target_edge_id: Option<EdgeId>,
) -> (bool, bool) {
    target_edge_id
        .and_then(|edge_id| {
            canvas
                .graph
                .read_ref(host, |g| {
                    let edge = g.edges.get(&edge_id)?;
                    Some(NodeGraphCanvasWith::<M>::edge_reconnectable_flags(
                        edge,
                        &snapshot.interaction,
                    ))
                })
                .ok()
                .flatten()
        })
        .unwrap_or((false, false))
}

pub(super) fn edge_anchor_endpoint_allowed(
    endpoint: EdgeEndpoint,
    reconnectable: (bool, bool),
) -> bool {
    match endpoint {
        EdgeEndpoint::From => reconnectable.0,
        EdgeEndpoint::To => reconnectable.1,
    }
}

#[cfg(test)]
mod tests;
