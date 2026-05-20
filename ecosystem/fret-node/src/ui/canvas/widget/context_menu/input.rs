use super::{key_navigation, ui};
use crate::ui::canvas::widget::widget_tail::WidgetHandledCx;
use crate::ui::canvas::widget::*;

pub(super) fn handle_context_menu_escape<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WidgetHandledCx<H>,
) -> bool {
    ui::handle_context_menu_escape_event(canvas, cx)
}

pub(super) fn handle_context_menu_key_down<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl key_navigation::ContextMenuKeyDownCx<H, M>,
    key: fret_core::KeyCode,
) -> bool {
    key_navigation::handle_context_menu_key_down_event(canvas, cx, key)
}
