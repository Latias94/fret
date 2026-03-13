use crate::ui::canvas::widget::*;

pub(super) fn activate_connection_insert_picker_action<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    from: PortId,
    at: CanvasPoint,
    invoked_at: Point,
    action: NodeGraphContextMenuAction,
    menu_candidates: &[InsertNodeCandidate],
) -> bool {
    match action {
        NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix) => {
            let mode = canvas.sync_view_state(cx.app).interaction.connection_mode;
            let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                return true;
            };
            canvas.record_recent_kind(&candidate.kind);
            let plan =
                canvas.plan_connection_insert_menu_candidate(cx.app, from, at, mode, &candidate);
            canvas.apply_connection_insert_menu_plan(cx, from, invoked_at, plan);
            true
        }
        _ => false,
    }
}
