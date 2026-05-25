use fret_runtime::CommandId;
use fret_ui::UiHost;

use super::low_level_adapter::CanvasHandledCx;

pub(super) trait PointerDownCloseButtonCx<H: UiHost>: CanvasHandledCx<H> {
    fn dispatch_close_command(&mut self, command: CommandId);
}
