use fret_ui::UiHost;

use super::low_level_adapter::CanvasPaintInvalidationCx;

pub(super) trait NodeResizeMoveCx<H: UiHost>: CanvasPaintInvalidationCx<H> {
    fn host(&mut self) -> &mut H;
}
