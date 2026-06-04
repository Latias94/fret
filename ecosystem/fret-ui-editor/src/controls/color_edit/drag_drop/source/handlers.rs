use fret_core::{Point, Px};
use fret_runtime::{DragKindId, Model};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod down;
mod move_phase;
mod up;

use super::super::ColorDragDropStore;
use crate::controls::color_edit::{ColorEditDragDropOptions, ColorEditDragDropPayload};

const COLOR_DRAG_KIND_MASK: u64 = 0x4000_0000_0000_0000;

pub(in crate::controls::color_edit) fn install_color_drag_source<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    source_id: GlobalElementId,
    store: Model<ColorDragDropStore>,
    payload: ColorEditDragDropPayload,
    options: ColorEditDragDropOptions,
    threshold: Px,
) {
    if !options.enabled {
        return;
    }

    let kind = color_drag_kind_for_element(source_id);
    down::install_color_drag_pointer_down(cx, source_id, store.clone(), options, kind);
    move_phase::install_color_drag_pointer_move(
        cx,
        source_id,
        store.clone(),
        payload,
        threshold,
        kind,
    );
    up::install_color_drag_pointer_up(cx, source_id, store, kind);
}

fn color_drag_kind_for_element(element: GlobalElementId) -> DragKindId {
    DragKindId(COLOR_DRAG_KIND_MASK ^ element.0)
}

fn color_drag_threshold_exceeded(start: Point, position: Point, threshold: Px) -> bool {
    let dx = position.x.0 - start.x.0;
    let dy = position.y.0 - start.y.0;
    let distance_sq = dx * dx + dy * dy;
    distance_sq >= threshold.0 * threshold.0
}
