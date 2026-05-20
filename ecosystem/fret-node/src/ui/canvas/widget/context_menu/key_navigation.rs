mod active_item;
mod hover;
mod key_down;
mod key_down_retained_cx;
mod pointer_move;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
mod typeahead;

use crate::ui::canvas::widget::widget_tail::{WidgetHandledCx, WidgetPaintInvalidationCx};
use crate::ui::canvas::widget::*;

pub(in crate::ui::canvas::widget) trait ContextMenuKeyDownCx<H, M: NodeGraphCanvasMiddleware>:
    WidgetHandledCx<H>
{
    fn activate_context_menu_active_selection(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        menu: &ContextMenuState,
    ) -> super::selection_activation::ContextMenuSelectionActivationOutcome;
}

pub(super) fn handle_context_menu_key_down_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl ContextMenuKeyDownCx<H, M>,
    key: fret_core::KeyCode,
) -> bool {
    key_down::handle_context_menu_key_down_event(canvas, cx, key)
}

pub(super) fn handle_context_menu_pointer_move_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WidgetPaintInvalidationCx<H>,
    position: Point,
    zoom: f32,
) -> bool {
    pointer_move::handle_context_menu_pointer_move_event(canvas, cx, position, zoom)
}
