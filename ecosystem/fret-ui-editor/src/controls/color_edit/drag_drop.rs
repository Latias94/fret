mod source;
mod store;

use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use super::model::format_hex;
use super::{ColorEditDragDropComponents, ColorEditDragDropPayload, ColorEditPaletteEntry};

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

pub(in crate::controls::color_edit) fn take_delivered_color_drop<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ColorDragDropStore>,
    target_id: GlobalElementId,
) -> Option<ColorEditDragDropPayload> {
    let current_tick = cx.app.tick_id();
    cx.app
        .models_mut()
        .update(store, |st| {
            let delivered = st.delivered.remove(&target_id)?;
            if current_tick.0 > delivered.tick_id.0.saturating_add(1) {
                return None;
            }
            Some(delivered.payload)
        })
        .ok()
        .flatten()
}

pub(super) struct ColorEditDeliveredDropArgs {
    pub(super) store: Model<ColorDragDropStore>,
    pub(super) target_id: GlobalElementId,
    pub(super) model: Model<Color>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) current: Color,
    pub(super) show_alpha: bool,
    pub(super) enabled: bool,
}

pub(super) fn apply_delivered_color_drop<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: ColorEditDeliveredDropArgs,
) {
    if !args.enabled {
        return;
    }
    let Some(payload) = take_delivered_color_drop(cx, &args.store, args.target_id) else {
        return;
    };

    let current_for_drop = cx
        .get_model_copied(&args.model, Invalidation::Paint)
        .unwrap_or(args.current);
    let next = apply_color_drop_payload(payload, current_for_drop, args.show_alpha);
    let formatted = format_hex(next, args.show_alpha);
    let _ = cx
        .app
        .models_mut()
        .update(&args.model, |color| *color = next);
    let _ = cx
        .app
        .models_mut()
        .update(&args.draft, |s| *s = formatted.as_ref().to_string());
    let _ = cx.app.models_mut().update(&args.error, |e| *e = None);
}

pub(super) fn apply_color_drop_payload(
    payload: ColorEditDragDropPayload,
    current: Color,
    target_show_alpha: bool,
) -> Color {
    let mut next = payload.color();
    if payload.components() == ColorEditDragDropComponents::Rgb || !target_show_alpha {
        next.a = current.a;
    }
    next
}

pub(super) fn palette_slot_drop_from_payload(
    previous: ColorEditPaletteEntry,
    payload: ColorEditDragDropPayload,
) -> ColorEditPaletteEntry {
    previous.with_rgb(fret_ui_kit::colors::hex_rgb_from_linear(payload.color()))
}
