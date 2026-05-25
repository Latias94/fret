mod active_item;
mod hover;
mod key_down;
mod pointer_move;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
mod typeahead;

use crate::ui::canvas::widget::low_level_adapter::{CanvasHandledCx, CanvasPaintInvalidationCx};
use crate::ui::canvas::widget::*;

pub(in crate::ui::canvas::widget) trait ContextMenuKeyDownCx<H, M: NodeGraphCanvasMiddleware>:
    CanvasHandledCx<H> + super::selection_activation::ContextMenuSelectionActivationCx<H, M>
{
}

impl<H, M, T> ContextMenuKeyDownCx<H, M> for T
where
    M: NodeGraphCanvasMiddleware,
    T: CanvasHandledCx<H> + super::selection_activation::ContextMenuSelectionActivationCx<H, M>,
{
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
    cx: &mut impl CanvasPaintInvalidationCx<H>,
    position: Point,
    zoom: f32,
) -> bool {
    pointer_move::handle_context_menu_pointer_move_event(canvas, cx, position, zoom)
}
