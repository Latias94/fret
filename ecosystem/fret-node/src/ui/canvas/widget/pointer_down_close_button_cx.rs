use fret_ui::UiHost;

use super::command_adapter::CanvasCommandDispatchCx;
use super::low_level_adapter::CanvasHandledCx;

pub(super) trait PointerDownCloseButtonCx<H: UiHost>:
    CanvasHandledCx<H> + CanvasCommandDispatchCx
{
}

impl<H, T> PointerDownCloseButtonCx<H> for T
where
    H: UiHost,
    T: CanvasHandledCx<H> + CanvasCommandDispatchCx,
{
}
