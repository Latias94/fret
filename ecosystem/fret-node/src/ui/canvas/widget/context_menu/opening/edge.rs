use crate::ui::canvas::widget::context_menu::ui::ContextMenuHoverEdgePolicy;
use crate::ui::canvas::widget::*;

pub(super) fn show_edge_context_menu<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::ContextMenuOpeningCx<H>,
    snapshot: &ViewSnapshot,
    position: Point,
    edge: EdgeId,
) -> bool {
    let items = canvas.build_edge_context_menu_items(cx.host(), edge);
    canvas.select_edge_context_target(cx.host(), edge);
    canvas.show_context_menu(
        cx,
        snapshot,
        position,
        ContextMenuTarget::Edge(edge),
        items,
        Vec::new(),
        ContextMenuHoverEdgePolicy::Clear,
    )
}
