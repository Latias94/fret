use crate::ui::canvas::widget::*;

use super::{finish, target};

pub(super) fn handle_edge_insert_picker_double_click<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) -> bool {
    if click_count != 2
        || !(modifiers.alt || modifiers.alt_gr)
        || canvas.interaction.searcher.is_some()
        || canvas.interaction.context_menu.is_some()
    {
        return false;
    }

    let Some(edge_id) = target::edge_double_click_target(canvas, cx, snapshot, position, zoom)
    else {
        return false;
    };

    canvas.select_edge_context_target(cx.app, edge_id);
    canvas.open_edge_insert_node_picker(cx.app, cx.window, edge_id, position);
    finish::finish_double_click(cx);
    true
}
