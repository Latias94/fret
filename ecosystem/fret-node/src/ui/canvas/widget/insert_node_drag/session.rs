use super::prelude::*;

pub(super) fn invalidate_insert_node_drag_preview<H: UiHost>(cx: &mut EventCx<'_, H>) {
    super::super::paint_invalidation::invalidate_paint(cx);
}

pub(super) fn set_insert_node_drag_preview<H: UiHost>(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    cx: &mut EventCx<'_, H>,
    preview: InsertNodeDragPreview,
) {
    interaction.insert_node_drag_preview = Some(preview);
    invalidate_insert_node_drag_preview(cx);
}

pub(super) fn clear_insert_node_drag_preview<H: UiHost>(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    cx: &mut EventCx<'_, H>,
) -> bool {
    if interaction.insert_node_drag_preview.take().is_none() {
        return false;
    }

    invalidate_insert_node_drag_preview(cx);
    true
}

pub(super) fn clear_insert_node_drag_state(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> bool {
    let mut cleared = false;
    if interaction.pending_insert_node_drag.take().is_some() {
        cleared = true;
    }
    if interaction.insert_node_drag_preview.take().is_some() {
        cleared = true;
    }
    cleared
}

pub(super) fn finish_insert_node_drag_event<H: UiHost>(cx: &mut EventCx<'_, H>) -> bool {
    cx.stop_propagation();
    true
}

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
    invalidate_insert_node_drag_preview(cx);
}
