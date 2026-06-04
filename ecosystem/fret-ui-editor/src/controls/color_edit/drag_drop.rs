mod delivery;
mod source;
mod store;

use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

#[cfg(test)]
pub(super) use delivery::apply_color_drop_payload;
pub(in crate::controls::color_edit) use delivery::take_delivered_color_drop;
pub(super) use delivery::{
    ColorEditDeliveredDropArgs, apply_delivered_color_drop, palette_slot_drop_from_payload,
};
pub(in crate::controls::color_edit) use source::install_color_drag_source;
pub(super) use source::resolve_color_drag_threshold;
pub(in crate::controls::color_edit) use store::{
    ActiveColorDrag, ColorDragDropStore, DeliveredColorDrop,
};
pub(super) use store::{color_drag_drop_store_for, prune_color_drag_drop_store};

pub(in crate::controls::color_edit) fn update_color_drop_target<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ColorDragDropStore>,
    target_id: GlobalElementId,
    hovered: bool,
    enabled: bool,
) -> bool {
    if !enabled {
        return false;
    }

    let active_sessions = cx
        .read_model(store, Invalidation::Paint, |app, st| {
            st.active
                .iter()
                .filter_map(|(session_id, active)| {
                    let drag = app.drag(active.pointer_id)?;
                    (drag.session_id == *session_id && drag.kind == active.kind && drag.dragging)
                        .then_some(*session_id)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if active_sessions.is_empty() {
        return false;
    }

    let mut over = false;
    let _ = cx.app.models_mut().update(store, |st| {
        for session_id in active_sessions {
            let Some(active) = st.active.get_mut(&session_id) else {
                continue;
            };
            if hovered {
                active.hovered_target = Some(target_id);
                over = true;
            } else if active.hovered_target == Some(target_id) {
                active.hovered_target = None;
            }
        }
    });
    over
}
