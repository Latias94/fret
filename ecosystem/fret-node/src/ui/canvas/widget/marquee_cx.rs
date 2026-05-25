use fret_ui::UiHost;

use super::{low_level_adapter::CanvasPointerCaptureReleaseCx, pan_zoom_begin_cx::PanZoomBeginCx};

pub(super) trait MarqueeCx<H: UiHost>:
    CanvasPointerCaptureReleaseCx<H> + PanZoomBeginCx<H>
{
}

impl<H, T> MarqueeCx<H> for T
where
    H: UiHost,
    T: CanvasPointerCaptureReleaseCx<H> + PanZoomBeginCx<H>,
{
}
