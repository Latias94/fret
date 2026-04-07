use fret_ui::UiHost;

use super::super::*;
use super::{ContextMenuHoverEdgePolicy, overlay};

pub(super) fn invalidate_context_menu_paint<H: UiHost>(cx: &mut EventCx<'_, H>) {
    super::super::retained_widget_runtime_shared::invalidate_widget_paint(cx);
}

pub(super) fn finish_context_menu_event<H: UiHost>(cx: &mut EventCx<'_, H>) -> bool {
    super::super::retained_widget_runtime_shared::finish_middleware_handled(cx);
    true
}

pub(super) fn open_context_menu_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    menu: ContextMenuState,
    hover_edge_policy: ContextMenuHoverEdgePolicy,
) -> bool {
    overlay::apply_context_menu_open_state(&mut canvas.interaction, menu, hover_edge_policy);
    cx.request_focus(cx.node);
    finish_context_menu_event(cx)
}

pub(super) fn restore_context_menu_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    menu: ContextMenuState,
) -> bool {
    overlay::restore_context_menu(&mut canvas.interaction, menu);
    finish_context_menu_event(cx)
}

pub(super) fn dismiss_context_menu_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) -> bool {
    if !overlay::clear_context_menu(&mut canvas.interaction) {
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
