use fret_ui::{UiHost, retained_bridge::EventCx};

use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

use super::super::ContextMenuState;
use super::super::selection_activation::ContextMenuSelectionActivationOutcome;
use super::ContextMenuKeyDownCx;

impl<H: UiHost, M: NodeGraphCanvasMiddleware> ContextMenuKeyDownCx<H, M> for EventCx<'_, H> {
    fn activate_context_menu_active_selection(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        menu: &ContextMenuState,
    ) -> ContextMenuSelectionActivationOutcome {
        canvas.activate_context_menu_active_selection(self, menu)
    }
}
