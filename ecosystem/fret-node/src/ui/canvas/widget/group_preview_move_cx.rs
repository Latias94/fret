use fret_core::Rect;
use fret_ui::UiHost;

use super::low_level_adapter::CanvasPaintInvalidationCx;

pub(super) trait GroupPreviewMoveCx<H: UiHost>: CanvasPaintInvalidationCx<H> {
    fn host(&mut self) -> &mut H;
    fn bounds(&self) -> Rect;
}
