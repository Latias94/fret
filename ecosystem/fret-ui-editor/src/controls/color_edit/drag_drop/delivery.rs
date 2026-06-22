use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use super::super::model::format_hex;
use super::super::{ColorEditDragDropComponents, ColorEditDragDropPayload, ColorEditPaletteEntry};
use super::ColorDragDropStore;

pub(in crate::controls::color_edit) fn take_delivered_color_drop<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ColorDragDropStore>,
    target_id: GlobalElementId,
) -> Option<ColorEditDragDropPayload> {
    let current_tick = cx.app.tick_id();
    let has_target = cx
        .read_model_ref(store, Invalidation::Paint, |st| {
            st.delivered.contains_key(&target_id)
        })
        .unwrap_or(true);
    if !has_target {
        return None;
    }

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

pub(in crate::controls::color_edit) struct ColorEditDeliveredDropArgs {
    pub(in crate::controls::color_edit) store: Model<ColorDragDropStore>,
    pub(in crate::controls::color_edit) target_id: GlobalElementId,
    pub(in crate::controls::color_edit) model: Model<Color>,
    pub(in crate::controls::color_edit) draft: Model<String>,
    pub(in crate::controls::color_edit) error: Model<Option<Arc<str>>>,
    pub(in crate::controls::color_edit) current: Color,
    pub(in crate::controls::color_edit) show_alpha: bool,
    pub(in crate::controls::color_edit) enabled: bool,
}

pub(in crate::controls::color_edit) fn apply_delivered_color_drop<H: UiHost>(
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

pub(in crate::controls::color_edit) fn apply_color_drop_payload(
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

pub(in crate::controls::color_edit) fn palette_slot_drop_from_payload(
    previous: ColorEditPaletteEntry,
    payload: ColorEditDragDropPayload,
) -> ColorEditPaletteEntry {
    previous.with_rgb(fret_ui_kit::colors::hex_rgb_from_linear(payload.color()))
}
