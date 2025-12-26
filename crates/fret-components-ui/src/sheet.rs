use fret_core::AppWindowId;
use fret_runtime::CommandId;
use fret_ui::{EventCx, UiHost};

use crate::{SheetRequest, SheetService};

/// Opens a window-scoped sheet by setting a `SheetRequest` and dispatching `sheet.open`.
///
/// - The application must have a `SheetOverlay` installed in an overlay layer (e.g. via
///   `fret_components_ui::WindowOverlays`).
pub fn open_sheet<H: UiHost>(cx: &mut EventCx<'_, H>, window: AppWindowId, request: SheetRequest) {
    cx.app
        .with_global_mut(SheetService::default, |service, _app| {
            service.set_request(window, request);
        });
    cx.dispatch_command(CommandId::from("sheet.open"));
    cx.stop_propagation();
}
