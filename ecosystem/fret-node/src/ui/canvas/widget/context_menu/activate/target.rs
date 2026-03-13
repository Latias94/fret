use crate::ui::canvas::widget::*;

pub(super) fn activate_target_context_action<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    target: &ContextMenuTarget,
    invoked_at: Point,
    action: NodeGraphContextMenuAction,
    menu_candidates: &[InsertNodeCandidate],
) {
    match target {
        ContextMenuTarget::BackgroundInsertNodePicker { at } => {
            let _ = canvas.activate_background_context_action(cx, *at, action, menu_candidates);
        }
        ContextMenuTarget::ConnectionInsertNodePicker { from, at } => {
            let _ = canvas.activate_connection_insert_picker_action(
                cx,
                *from,
                *at,
                invoked_at,
                action,
                menu_candidates,
            );
        }
        ContextMenuTarget::Edge(edge_id) => {
            let _ = canvas.activate_edge_context_action(cx, *edge_id, invoked_at, action);
        }
        ContextMenuTarget::EdgeInsertNodePicker(edge_id) => {
            let _ = edge_insert::activate_edge_insert_picker_action(
                canvas,
                cx,
                *edge_id,
                invoked_at,
                action,
                menu_candidates,
            );
        }
        ContextMenuTarget::ConnectionConvertPicker { from, to, at } => {
            let _ = canvas.activate_connection_conversion_picker_action(
                cx,
                *from,
                *to,
                *at,
                invoked_at,
                action,
                menu_candidates,
            );
        }
        _ => {}
    }
}
