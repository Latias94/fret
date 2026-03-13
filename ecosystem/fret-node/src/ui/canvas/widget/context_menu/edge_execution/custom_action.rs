use crate::ui::canvas::widget::*;

pub(super) fn apply_custom_edge_context_action<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    edge_id: EdgeId,
    action_id: u64,
) {
    let ops = {
        let presenter = &mut *canvas.presenter;
        canvas
            .graph
            .read_ref(cx.app, |graph| {
                presenter.on_edge_context_menu_action(graph, edge_id, action_id)
            })
            .ok()
            .flatten()
            .unwrap_or_default()
    };
    if !ops.is_empty() {
        canvas.apply_ops(cx.app, cx.window, ops);
    }
}
