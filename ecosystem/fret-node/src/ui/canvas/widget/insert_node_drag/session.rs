use super::prelude::*;

pub(super) fn abort_pending_insert_node_drag<H: UiHost>(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    cx: &mut EventCx<'_, H>,
) -> bool {
    if interaction.pending_insert_node_drag.take().is_none() {
        return false;
    }
    cx.release_pointer_capture();
    false
}

pub(super) fn abort_pending_insert_node_drag_and_clear_dnd<H: UiHost>(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    cx: &mut EventCx<'_, H>,
    drag_kind: DragKindId,
    pointer_id: fret_core::PointerId,
) -> bool {
    if interaction.pending_insert_node_drag.take().is_none() {
        return false;
    }
    if let Some(window) = cx.window {
        let dnd = ui_dnd::dnd_service_model_global(cx.app);
        ui_dnd::clear_pointer(cx.app.models_mut(), &dnd, window, drag_kind, pointer_id);
    }
    cx.release_pointer_capture();
    false
}

pub(super) fn finish_pending_insert_node_drag<H: UiHost>(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    cx: &mut EventCx<'_, H>,
) {
    interaction.searcher = None;
    interaction.pending_insert_node_drag = None;
    cx.release_pointer_capture();
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
}
