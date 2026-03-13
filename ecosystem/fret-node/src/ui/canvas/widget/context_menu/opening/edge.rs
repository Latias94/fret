use crate::ui::canvas::widget::*;

pub(super) fn try_show_edge_context_menu<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    let Some(edge) = canvas.hit_edge_context_target(cx.app, snapshot, position, zoom) else {
        return false;
    };

    let items = canvas.build_edge_context_menu_items(cx.app, edge);
    canvas.select_edge_context_target(cx.app, edge);
    canvas.show_context_menu(
        cx,
        snapshot,
        position,
        ContextMenuTarget::Edge(edge),
        items,
        Vec::new(),
        true,
    )
}
