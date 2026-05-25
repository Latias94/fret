use fret_ui::UiHost;

use super::low_level_adapter::CanvasPaintInvalidationCx;

pub(super) trait WireDragStartCx<H: UiHost>: CanvasPaintInvalidationCx<H> {
    fn capture_self_pointer(&mut self);
}
