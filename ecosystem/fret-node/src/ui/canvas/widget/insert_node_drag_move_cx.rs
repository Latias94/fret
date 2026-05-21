use fret_core::{AppWindowId, PointerId, Rect};
use fret_runtime::TickId;
use fret_ui::UiHost;

use super::widget_tail::PointerCaptureReleaseCx;

pub(super) trait InsertNodeDragMoveCx<H: UiHost>: PointerCaptureReleaseCx<H> {
    fn host(&mut self) -> &mut H;
    fn pointer_id(&self) -> Option<PointerId>;
    fn window(&self) -> Option<AppWindowId>;
    fn bounds(&self) -> Rect;
    fn tick_id(&self) -> TickId;
}
