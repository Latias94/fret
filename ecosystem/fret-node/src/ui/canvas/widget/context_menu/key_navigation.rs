mod active_item;
mod hover;
mod key_down;
mod pointer_move;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
mod typeahead;

use fret_ui::UiHost;

use crate::ui::canvas::widget::*;

pub(super) fn handle_context_menu_key_down_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    key: fret_core::KeyCode,
) -> bool {
    key_down::handle_context_menu_key_down_event(canvas, cx, key)
}

pub(super) fn handle_context_menu_pointer_move_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    zoom: f32,
) -> bool {
    pointer_move::handle_context_menu_pointer_move_event(canvas, cx, position, zoom)
}
