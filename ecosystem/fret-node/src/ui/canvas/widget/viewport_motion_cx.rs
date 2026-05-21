use fret_ui::UiHost;

use super::{auto_pan_timer_cx::AutoPanTimerCx, widget_tail::WidgetPaintInvalidationCx};

pub(super) trait ViewportMotionCx<H: UiHost>:
    AutoPanTimerCx<H> + WidgetPaintInvalidationCx<H>
{
}

impl<H, T> ViewportMotionCx<H> for T
where
    H: UiHost,
    T: AutoPanTimerCx<H> + WidgetPaintInvalidationCx<H>,
{
}
