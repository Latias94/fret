use crate::ui::canvas::widget::*;

pub(super) fn activate_connection_conversion_picker_action<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    from: PortId,
    to: PortId,
    at: CanvasPoint,
    invoked_at: Point,
    action: NodeGraphContextMenuAction,
    menu_candidates: &[InsertNodeCandidate],
) -> bool {
    match action {
        NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix) => {
            let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                return true;
            };
            canvas.record_recent_kind(&candidate.kind);
            let plan =
                canvas.plan_connection_conversion_menu_candidate(cx.app, from, to, at, &candidate);
            canvas.apply_connection_conversion_menu_plan(cx, from, invoked_at, plan);
            true
        }
        _ => false,
    }
}
