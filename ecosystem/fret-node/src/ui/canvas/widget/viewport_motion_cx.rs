use fret_ui::UiHost;

use super::{auto_pan_timer_cx::AutoPanTimerCx, low_level_adapter::CanvasPaintInvalidationCx};

pub(super) trait ViewportMotionCx<H: UiHost>:
    AutoPanTimerCx<H> + CanvasPaintInvalidationCx<H>
{
}

impl<H, T> ViewportMotionCx<H> for T
where
    H: UiHost,
    T: AutoPanTimerCx<H> + CanvasPaintInvalidationCx<H>,
{
}
