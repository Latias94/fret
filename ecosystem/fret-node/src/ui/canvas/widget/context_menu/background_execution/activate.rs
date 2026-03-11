use crate::ui::canvas::widget::*;

pub(super) fn activate_background_context_action<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    at: CanvasPoint,
    action: NodeGraphContextMenuAction,
    menu_candidates: &[InsertNodeCandidate],
) -> bool {
    match action {
        NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix) => {
            let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                return true;
            };
            canvas.record_recent_kind(&candidate.kind);
            let plan = canvas.plan_background_insert_menu_candidate(cx.app, &candidate, at);
            canvas.apply_background_insert_menu_plan(cx, plan);
            true
        }
        _ => false,
    }
}
