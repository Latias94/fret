//! Paint grid tile cache adapter contract.
//!
//! This module keeps grid tile cache warmup scene sink access behind a named seam. Tile cache
//! planning and operation generation stay in grid modules; concrete scene sinks live next to the
//! retained paint binding.

use fret_core::Scene;
use fret_ui::UiHost;

pub(super) trait PaintGridTileCacheCx<H: UiHost>:
    super::low_level_adapter::CanvasRedrawCx<H>
{
    fn paint_grid_scene(&mut self) -> &mut Scene;
}

pub(super) fn paint_grid_scene<H>(cx: &mut impl PaintGridTileCacheCx<H>) -> &mut Scene
where
    H: UiHost,
{
    cx.paint_grid_scene()
}

pub(super) fn request_grid_paint_redraw<H>(cx: &mut impl PaintGridTileCacheCx<H>)
where
    H: UiHost,
{
    super::redraw_request::request_paint_redraw(cx);
}
