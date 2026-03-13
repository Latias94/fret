use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(in super::super) fn handle_edge_insert_double_click<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    click_count: u8,
    modifiers: Modifiers,
) -> bool {
    if !should_open_edge_insert_picker(click_count, modifiers) {
        return false;
    }

    let Some(edge_drag) = canvas.interaction.edge_drag.take() else {
        return false;
    };

    canvas.open_edge_insert_node_picker(cx.app, cx.window, edge_drag.edge, position);
    canvas.interaction.hover_edge = None;
    cx.release_pointer_capture();
    super::super::paint_invalidation::invalidate_paint(cx);
    true
}

fn should_open_edge_insert_picker(click_count: u8, modifiers: Modifiers) -> bool {
    click_count == 2 && !(modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr)
}

#[cfg(test)]
mod tests;
