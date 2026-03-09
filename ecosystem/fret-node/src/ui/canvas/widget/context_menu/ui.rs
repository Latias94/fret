use crate::ui::canvas::widget::*;

pub(super) fn clear_context_menu(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> bool {
    interaction.context_menu.take().is_some()
}

pub(super) fn restore_context_menu(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    menu: ContextMenuState,
) {
    interaction.context_menu = Some(menu);
}

pub(super) fn invalidate_context_menu_paint<H: UiHost>(cx: &mut EventCx<'_, H>) {
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
}

pub(super) fn finish_context_menu_event<H: UiHost>(cx: &mut EventCx<'_, H>) -> bool {
    cx.stop_propagation();
    invalidate_context_menu_paint(cx);
    true
}

pub(super) fn restore_context_menu_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    menu: ContextMenuState,
) -> bool {
    restore_context_menu(&mut canvas.interaction, menu);
    finish_context_menu_event(cx)
}

pub(super) fn dismiss_context_menu_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) -> bool {
    if !clear_context_menu(&mut canvas.interaction) {
        return false;
    }

    finish_context_menu_event(cx)
}

pub(super) fn handle_context_menu_escape_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) -> bool {
    dismiss_context_menu_event(canvas, cx)
}
