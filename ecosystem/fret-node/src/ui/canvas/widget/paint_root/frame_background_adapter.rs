//! Paint-root frame background adapter contract.
//!
//! This module keeps root frame background scene emission behind a named seam. Chrome hint policy
//! stays in frame background preparation; concrete scene sinks live next to the retained paint root
//! binding.

use fret_core::{Color, Rect};
use fret_ui::UiHost;

pub(super) trait PaintRootFrameBackgroundCx<H: UiHost> {
    fn paint_root_frame_background(&mut self, viewport_rect: Rect, background: Color);
}

pub(super) fn paint_root_frame_background<H>(
    cx: &mut impl PaintRootFrameBackgroundCx<H>,
    viewport_rect: Rect,
    background: Color,
) where
    H: UiHost,
{
    cx.paint_root_frame_background(viewport_rect, background);
}
