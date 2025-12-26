use fret_core::AppWindowId;
use fret_ui::{EventCx, UiHost};

use crate::sheet::{SheetRequest, SheetSide};

/// shadcn/ui `Drawer` (compatibility alias).
///
/// Upstream shadcn often uses Vaul for drag gestures; in Fret we currently map this to a bottom
/// sheet with modal dismissal semantics.
pub fn open_drawer<H: UiHost>(cx: &mut EventCx<'_, H>, window: AppWindowId, request: SheetRequest) {
    crate::sheet::open_sheet(cx, window, request.side(SheetSide::Bottom));
}
