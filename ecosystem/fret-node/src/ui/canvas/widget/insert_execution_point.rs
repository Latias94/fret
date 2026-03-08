use super::*;

pub(super) fn is_reroute_insert_candidate(candidate: &InsertNodeCandidate) -> bool {
    candidate.kind.0 == REROUTE_KIND
}

pub(super) fn insert_candidate_canvas_point<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    candidate: &InsertNodeCandidate,
    invoked_at: Point,
) -> CanvasPoint {
    if is_reroute_insert_candidate(candidate) {
        canvas.reroute_pos_for_invoked_at(invoked_at)
    } else {
        CanvasPoint {
            x: invoked_at.x.0,
            y: invoked_at.y.0,
        }
    }
}
