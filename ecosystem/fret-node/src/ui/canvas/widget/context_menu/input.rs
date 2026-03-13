use super::{key_navigation, ui};
use crate::ui::canvas::widget::*;

pub(super) fn handle_context_menu_escape<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    ui::handle_context_menu_escape_event(canvas, cx)
}

pub(super) fn handle_context_menu_key_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    key: fret_core::KeyCode,
) -> bool {
    key_navigation::handle_context_menu_key_down_event(canvas, cx, key)
}
