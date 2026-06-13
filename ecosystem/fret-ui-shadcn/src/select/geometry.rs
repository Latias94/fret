use fret_core::{Px, Rect};
use fret_ui_kit::primitives::select as radix_select;

use super::SelectPosition;

pub(super) fn select_list_desired_height_from_content_height(
    min_row_height: Px,
    content_height: Px,
    max_height: Px,
    outer_height: Px,
) -> Px {
    let outer_height = Px(outer_height.0.max(0.0));
    let max_height = Px(max_height.0.max(0.0).min(outer_height.0));

    let min_height = Px(min_row_height.0.max(0.0).min(max_height.0));
    let content_height = Px(content_height.0.max(0.0));

    Px(content_height.0.min(max_height.0).max(min_height.0))
}

pub(super) fn select_content_desired_width_with_probe(
    outer: Rect,
    anchor: Rect,
    min_width: Px,
    border_width: Px,
    width_probe_w: Option<Px>,
    position: SelectPosition,
) -> Px {
    let base = radix_select::select_popper_desired_width(outer, anchor, min_width);
    let probe = width_probe_w.map(|probe_w| {
        let border_extra = Px(border_width.0 * 2.0);
        Px(probe_w.0 + border_extra.0)
    });

    let desired = match (position, probe) {
        // Upstream shadcn/radix `position="popper"` keeps the content at least as wide as the
        // trigger (`min-w-[var(--radix-select-trigger-width)]`). Do not let the width probe shrink
        // the popup below the trigger-derived base width once it becomes available a frame later.
        (SelectPosition::Popper, Some(probe)) => Px(base.0.max(probe.0)),
        (_, Some(probe)) => probe,
        (_, None) => base,
    };

    Px(desired.0.max(min_width.0).min(outer.size.width.0))
}
