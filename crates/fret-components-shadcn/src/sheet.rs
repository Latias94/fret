use fret_core::AppWindowId;
use fret_ui::{EventCx, UiHost};

pub use fret_components_ui::sheet_overlay::{
    SheetOverlay, SheetRequest, SheetService, SheetSide, SheetStyle,
};

/// Opens a window-scoped sheet by setting a `SheetRequest` and dispatching `sheet.open`.
///
/// This uses the standard sheet overlay installed by `fret_components_shadcn::WindowOverlays`.
pub fn open_sheet<H: UiHost>(cx: &mut EventCx<'_, H>, window: AppWindowId, request: SheetRequest) {
    fret_components_ui::sheet::open_sheet(cx, window, request);
}
