use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::super::super::paint_invalidation::invalidate_paint;

pub(super) fn finish_edge_insert_drag_move<H: UiHost>(cx: &mut EventCx<'_, H>) {
    invalidate_paint(cx);
}
