use fret_ui::{EventCx, UiHost};

use crate::ui::canvas::widget::*;

use super::ContextMenuSelectionActivationCx;

impl<H: UiHost, M: NodeGraphCanvasMiddleware> ContextMenuSelectionActivationCx<H, M>
    for EventCx<'_, H>
{
    fn activate_context_menu_item(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        target: &ContextMenuTarget,
        invoked_at: Point,
        item: NodeGraphContextMenuItem,
        menu_candidates: &[InsertNodeCandidate],
    ) {
        canvas.activate_context_menu_item(self, target, invoked_at, item, menu_candidates);
    }
}
