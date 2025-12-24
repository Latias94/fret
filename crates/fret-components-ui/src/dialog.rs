//! Dialog helpers built on top of `fret-ui`'s dialog overlay.
//!
//! This module intentionally provides small, composable building blocks. Higher-level shadcn-style
//! `DialogTrigger`/`DialogContent` composition will likely require a general-purpose portal API.

use fret_core::AppWindowId;
use fret_runtime::CommandId;
use fret_ui::{DialogRequest, DialogService, EventCx, UiHost};

/// Opens a window-scoped dialog by setting a `DialogRequest` and dispatching `dialog.open`.
///
/// Notes:
/// - The application must have a `DialogOverlay` installed in an overlay layer (e.g. via
///   `fret_ui::WindowOverlays`).
pub fn open_dialog<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    request: DialogRequest,
) {
    cx.app
        .with_global_mut(DialogService::default, |service, _app| {
            service.set_request(window, request);
        });
    cx.dispatch_command(CommandId::from("dialog.open"));
    cx.stop_propagation();
}
