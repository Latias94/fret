use fret_core::AppWindowId;
#[cfg(feature = "unstable-internals")]
use fret_ui::ElementContext;
use fret_ui::UiHost;

use super::WindowOverlaySynthesisDiagnosticsStore;
use super::state::WindowOverlays;
use super::{
    DismissiblePopoverRequest, HoverOverlayRequest, ModalRequest, ToastLayerRequest, TooltipRequest,
};

pub fn begin_frame<H: UiHost>(app: &mut H, window: AppWindowId) {
    let frame_id = app.frame_id();
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        if w.frame_id != frame_id {
            w.frame_id = frame_id;
            w.popovers.clear();
            w.modals.clear();
            w.hover_overlays.clear();
            w.tooltips.clear();
            w.toasts.clear();
        }
    });

    app.with_global_mut_untracked(
        WindowOverlaySynthesisDiagnosticsStore::default,
        |diag, _app| {
            diag.begin_frame(window, frame_id);
        },
    );
}

#[cfg(feature = "unstable-internals")]
pub fn request_dismissible_popover<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    request: DismissiblePopoverRequest,
) {
    request_dismissible_popover_for_window(cx.app, cx.window, request);
}

pub fn request_dismissible_popover_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: DismissiblePopoverRequest,
) {
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        overlays
            .cached_popover_requests
            .insert((window, request.id), request.clone());
        w.popovers.push(request);
    });
}

#[cfg(feature = "unstable-internals")]
pub fn request_modal<H: UiHost>(cx: &mut ElementContext<'_, H>, request: ModalRequest) {
    request_modal_for_window(cx.app, cx.window, request);
}

pub fn request_modal_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: ModalRequest,
) {
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        overlays
            .cached_modal_requests
            .insert((window, request.id), request.clone());
        w.modals.push(request);
    });
}

#[cfg(feature = "unstable-internals")]
pub fn request_hover_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    request: HoverOverlayRequest,
) {
    request_hover_overlay_for_window(cx.app, cx.window, request);
}

pub fn request_hover_overlay_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: HoverOverlayRequest,
) {
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        overlays
            .cached_hover_overlay_pointer_move_handlers
            .insert((window, request.id), request.on_pointer_move.clone());
        overlays
            .cached_hover_overlay_requests
            .insert((window, request.id), request.clone());
        w.hover_overlays.push(request);
    });
}

#[cfg(feature = "unstable-internals")]
pub fn request_tooltip<H: UiHost>(cx: &mut ElementContext<'_, H>, request: TooltipRequest) {
    request_tooltip_for_window(cx.app, cx.window, request);
}

pub fn request_tooltip_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: TooltipRequest,
) {
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        overlays
            .cached_tooltip_requests
            .insert((window, request.id), request.clone());
        w.tooltips.push(request);
    });
}

#[cfg(feature = "unstable-internals")]
pub fn request_toast_layer<H: UiHost>(cx: &mut ElementContext<'_, H>, request: ToastLayerRequest) {
    request_toast_layer_for_window(cx.app, cx.window, request);
}

pub fn request_toast_layer_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: ToastLayerRequest,
) {
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        overlays
            .cached_toast_layer_requests
            .insert((window, request.id), request.clone());
        w.toasts.push(request);
    });
}
