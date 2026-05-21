use fret_ui::UiHost;

use super::{pointer_move_tail_cx::PointerMoveTailCx, viewport_motion_cx::ViewportMotionCx};

pub(super) trait TimerMotionCx<H: UiHost>:
    ViewportMotionCx<H> + PointerMoveTailCx<H>
{
}

impl<H, T> TimerMotionCx<H> for T
where
    H: UiHost,
    T: ViewportMotionCx<H> + PointerMoveTailCx<H>,
{
}
