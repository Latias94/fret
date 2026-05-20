use fret_ui::UiHost;

use super::{pan_zoom_begin_cx::PanZoomBeginCx, widget_tail::PointerCaptureReleaseCx};

pub(super) trait MarqueeCx<H: UiHost>:
    PointerCaptureReleaseCx<H> + PanZoomBeginCx<H>
{
}

impl<H, T> MarqueeCx<H> for T
where
    H: UiHost,
    T: PointerCaptureReleaseCx<H> + PanZoomBeginCx<H>,
{
}
