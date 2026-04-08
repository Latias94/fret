mod payload;
mod pointer_down;

use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in crate::ui::canvas::widget) fn activate_context_menu_active_selection<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        menu: &ContextMenuState,
    ) -> bool {
        let index = menu.active_item.min(menu.items.len().saturating_sub(1));
        self.activate_context_menu_selection(cx, menu, index)
    }

    pub(in crate::ui::canvas::widget) fn activate_context_menu_selection<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        menu: &ContextMenuState,
        index: usize,
    ) -> bool {
        let Some((target, invoked_at, item, candidates)) =
            payload::context_menu_activation_payload(menu, index)
        else {
            return false;
        };
        self.activate_context_menu_item(cx, &target, invoked_at, item, &candidates);
        true
    }
}

pub(super) fn handle_context_menu_pointer_down_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    pointer_down::handle_context_menu_pointer_down_event(canvas, cx, position, button, zoom)
}
