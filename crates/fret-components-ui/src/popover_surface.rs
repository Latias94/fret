//! Popover surface helpers built on top of the component kit's popover surface overlay.
//!
//! This module provides small helpers for setting the request and dispatching the standard
//! open/close commands. Focus restoration and layer visibility are handled by `WindowOverlays`.

use fret_core::AppWindowId;
use fret_runtime::CommandId;
use fret_ui::{EventCx, UiHost};

use crate::popover_surface_overlay::{PopoverSurfaceRequest, PopoverSurfaceService};

/// Opens a window-scoped popover surface by setting a `PopoverSurfaceRequest` and dispatching
/// `popover_surface.open`.
///
/// Notes:
/// - The application must have a `PopoverSurfaceOverlay` installed in an overlay layer (e.g. via
///   `fret_components_ui::WindowOverlays`).
pub fn open_popover_surface<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    request: PopoverSurfaceRequest,
) {
    cx.app
        .with_global_mut(PopoverSurfaceService::default, |service, _app| {
            service.set_request(window, request);
        });
    cx.dispatch_command(CommandId::from("popover_surface.open"));
    cx.stop_propagation();
}
