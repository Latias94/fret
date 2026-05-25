//! Paint-root frame clip adapter contract.
//!
//! This module keeps root frame clip emission behind a named seam. Concrete scene sinks live next
//! to the retained paint root binding.

use fret_core::Rect;
use fret_ui::UiHost;

pub(super) trait PaintRootFrameClipCx<H: UiHost> {
    fn push_paint_root_frame_clip_rect(&mut self, rect: Rect);
}

pub(super) fn push_paint_root_frame_clip<H>(cx: &mut impl PaintRootFrameClipCx<H>, rect: Rect)
where
    H: UiHost,
{
    cx.push_paint_root_frame_clip_rect(rect);
}
