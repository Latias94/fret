use crate::ui::canvas::widget::context_menu::ui::ContextMenuHoverEdgePolicy;
use crate::ui::canvas::widget::*;

use super::super::item_builders;

pub(super) fn show_background_context_menu<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
) -> bool {
    let has_selection = !snapshot.selected_nodes.is_empty()
        || !snapshot.selected_edges.is_empty()
        || !snapshot.selected_groups.is_empty();
    let items =
        item_builders::build_background_context_menu_items(cx.window.is_some(), has_selection);

    canvas.show_context_menu(
        cx,
        snapshot,
        position,
        ContextMenuTarget::Background,
        items,
        Vec::new(),
        ContextMenuHoverEdgePolicy::Preserve,
    )
}
