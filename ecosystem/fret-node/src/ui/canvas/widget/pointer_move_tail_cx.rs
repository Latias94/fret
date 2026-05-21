use fret_ui::UiHost;

use super::{
    auto_pan_timer_cx::AutoPanTimerCx, cursor_cx::CanvasCursorCx, pointer_move_cx::PointerMoveCx,
};

pub(super) trait PointerMoveTailCx<H: UiHost>:
    CanvasCursorCx<H> + PointerMoveCx<H> + AutoPanTimerCx<H>
{
}

impl<H, T> PointerMoveTailCx<H> for T
where
    H: UiHost,
    T: CanvasCursorCx<H> + PointerMoveCx<H> + AutoPanTimerCx<H>,
{
}
