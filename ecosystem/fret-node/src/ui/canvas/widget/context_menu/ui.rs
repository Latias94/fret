mod event;
mod event_retained_cx;
mod overlay;

pub(in crate::ui::canvas::widget) use event::ContextMenuFocusCx;
pub(in crate::ui::canvas::widget) use overlay::ContextMenuHoverEdgePolicy;

use crate::ui::canvas::widget::widget_tail::{WidgetHandledCx, WidgetPaintInvalidationCx};
use crate::ui::canvas::widget::*;

pub(super) fn apply_context_menu_open_state(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    menu: ContextMenuState,
    hover_edge_policy: ContextMenuHoverEdgePolicy,
) {
    overlay::apply_context_menu_open_state(interaction, menu, hover_edge_policy);
}

pub(super) fn clear_context_menu(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> bool {
    overlay::clear_context_menu(interaction)
}

pub(super) fn take_context_menu(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> Option<ContextMenuState> {
    overlay::take_context_menu(interaction)
}

pub(super) fn restore_context_menu(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    menu: ContextMenuState,
) {
    overlay::restore_context_menu(interaction, menu);
}

pub(super) fn invalidate_context_menu_paint<H>(cx: &mut impl WidgetPaintInvalidationCx<H>) {
    event::invalidate_context_menu_paint(cx);
}

pub(super) fn finish_context_menu_event<H>(cx: &mut impl WidgetHandledCx<H>) -> bool {
    event::finish_context_menu_event(cx)
}

pub(super) fn open_context_menu_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl event::ContextMenuFocusCx<H>,
    menu: ContextMenuState,
    hover_edge_policy: ContextMenuHoverEdgePolicy,
) -> bool {
    event::open_context_menu_event(canvas, cx, menu, hover_edge_policy)
}

pub(super) fn restore_context_menu_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WidgetHandledCx<H>,
    menu: ContextMenuState,
) -> bool {
    event::restore_context_menu_event(canvas, cx, menu)
}

pub(super) fn dismiss_context_menu_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WidgetHandledCx<H>,
) -> bool {
    event::dismiss_context_menu_event(canvas, cx)
}

pub(super) fn handle_context_menu_escape_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WidgetHandledCx<H>,
) -> bool {
    event::handle_context_menu_escape_event(canvas, cx)
}
