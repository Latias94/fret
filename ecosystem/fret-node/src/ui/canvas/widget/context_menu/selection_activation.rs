mod payload;
mod pointer_down;
mod retained_cx;
#[cfg(test)]
mod tests;

use crate::ui::canvas::widget::widget_tail::WidgetHandledCx;
use crate::ui::canvas::widget::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::ui::canvas::widget) enum ContextMenuSelectionActivationOutcome {
    Activated,
    KeepOpen,
}

pub(in crate::ui::canvas::widget) trait ContextMenuSelectionActivationCx<
    H,
    M: NodeGraphCanvasMiddleware,
>
{
    fn activate_context_menu_item(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        target: &ContextMenuTarget,
        invoked_at: Point,
        item: NodeGraphContextMenuItem,
        menu_candidates: &[InsertNodeCandidate],
    );
}

pub(in crate::ui::canvas::widget) trait ContextMenuPointerDownCx<H, M: NodeGraphCanvasMiddleware>:
    WidgetHandledCx<H> + ContextMenuSelectionActivationCx<H, M>
{
}

impl<H, M, T> ContextMenuPointerDownCx<H, M> for T
where
    M: NodeGraphCanvasMiddleware,
    T: WidgetHandledCx<H> + ContextMenuSelectionActivationCx<H, M>,
{
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in crate::ui::canvas::widget) fn activate_context_menu_active_selection<H>(
        &mut self,
        cx: &mut impl ContextMenuSelectionActivationCx<H, M>,
        menu: &ContextMenuState,
    ) -> ContextMenuSelectionActivationOutcome {
        let index = menu.active_item.min(menu.items.len().saturating_sub(1));
        self.activate_context_menu_selection(cx, menu, index)
    }

    pub(in crate::ui::canvas::widget) fn activate_context_menu_selection<H>(
        &mut self,
        cx: &mut impl ContextMenuSelectionActivationCx<H, M>,
        menu: &ContextMenuState,
        index: usize,
    ) -> ContextMenuSelectionActivationOutcome {
        let Some((target, invoked_at, item, candidates)) =
            payload::context_menu_activation_payload(menu, index)
        else {
            return ContextMenuSelectionActivationOutcome::KeepOpen;
        };
        cx.activate_context_menu_item(self, &target, invoked_at, item, &candidates);
        ContextMenuSelectionActivationOutcome::Activated
    }
}

pub(super) fn handle_context_menu_pointer_down_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl ContextMenuPointerDownCx<H, M>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    pointer_down::handle_context_menu_pointer_down_event(canvas, cx, position, button, zoom)
}
