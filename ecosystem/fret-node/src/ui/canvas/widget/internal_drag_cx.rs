use fret_core::{AppWindowId, PointerId};
use fret_runtime::DragSession;
use fret_ui::UiHost;

use super::widget_tail::WidgetHandledCx;

pub(super) trait InternalDragCx<H: UiHost>: WidgetHandledCx<H> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
    fn drag_session(&self, pointer_id: PointerId) -> Option<&DragSession>;
}
