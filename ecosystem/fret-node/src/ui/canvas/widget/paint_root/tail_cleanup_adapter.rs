//! Paint-root tail cleanup adapter contract.
//!
//! This module keeps the root frame tail cleanup clip-pop emission behind a named seam. Concrete
//! scene sinks live next to the retained paint root binding.

use fret_ui::UiHost;

pub(super) trait PaintRootTailCleanupCx<H: UiHost> {
    fn pop_paint_root_tail_clip(&mut self);
}

pub(super) fn pop_paint_root_tail_clip<H>(cx: &mut impl PaintRootTailCleanupCx<H>)
where
    H: UiHost,
{
    cx.pop_paint_root_tail_clip();
}
