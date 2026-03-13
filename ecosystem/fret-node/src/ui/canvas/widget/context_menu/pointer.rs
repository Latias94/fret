use super::{key_navigation, selection_activation};
use crate::ui::canvas::widget::*;

pub(super) fn handle_context_menu_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    selection_activation::handle_context_menu_pointer_down_event(canvas, cx, position, button, zoom)
}

pub(super) fn handle_context_menu_pointer_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    zoom: f32,
) -> bool {
    key_navigation::handle_context_menu_pointer_move_event(canvas, cx, position, zoom)
}
