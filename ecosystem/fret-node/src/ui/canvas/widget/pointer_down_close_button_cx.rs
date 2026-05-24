use fret_runtime::CommandId;
use fret_ui::UiHost;

use super::widget_tail::WidgetHandledCx;

pub(super) trait PointerDownCloseButtonCx<H: UiHost>: WidgetHandledCx<H> {
    fn dispatch_close_command(&mut self, command: CommandId);
}
