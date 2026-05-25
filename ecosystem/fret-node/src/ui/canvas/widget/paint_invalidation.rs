use fret_ui::UiHost;

use super::low_level_adapter::{CanvasPaintInvalidationCx, invalidate_canvas_paint};

pub(super) fn invalidate_paint<H: UiHost>(cx: &mut impl CanvasPaintInvalidationCx<H>) {
    invalidate_canvas_paint(cx);
}
