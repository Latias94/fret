use fret_core::AppWindowId;
#[cfg(feature = "unstable-internals")]
use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::elements::GlobalElementId;

use super::WindowOverlaySynthesisDiagnosticsStore;
use super::requests::{
    CachedDismissiblePopoverDecl, CachedHoverOverlayDecl, CachedModalDecl, CachedToastLayerDecl,
    CachedTooltipDecl,
};
use super::state::WindowOverlays;
use super::{
    DismissiblePopoverRequest, HoverOverlayRequest, ModalRequest, ToastLayerRequest, TooltipRequest,
};

fn record_dismissible_popover_request(
    overlays: &mut WindowOverlays,
    window: AppWindowId,
    request: DismissiblePopoverRequest,
    owner: Option<GlobalElementId>,
) {
    let w = overlays.windows.entry(window).or_default();
    overlays.cached_popover_requests.insert(
        (window, request.id),
        CachedDismissiblePopoverDecl::from_request(&request, owner),
    );
    w.popovers.push(request);
}

fn record_modal_request(
    overlays: &mut WindowOverlays,
    window: AppWindowId,
    request: ModalRequest,
    owner: Option<GlobalElementId>,
) {
    let w = overlays.windows.entry(window).or_default();
    overlays.cached_modal_requests.insert(
        (window, request.id),
        CachedModalDecl::from_request(&request, owner),
    );
    w.modals.push(request);
}

fn record_hover_overlay_request(
    overlays: &mut WindowOverlays,
    window: AppWindowId,
    request: HoverOverlayRequest,
    owner: Option<GlobalElementId>,
) {
    let w = overlays.windows.entry(window).or_default();
    overlays
        .cached_hover_overlay_pointer_move_handlers
        .insert((window, request.id), request.on_pointer_move.clone());
    overlays.cached_hover_overlay_requests.insert(
        (window, request.id),
        CachedHoverOverlayDecl::from_request(&request, owner),
    );
    w.hover_overlays.push(request);
}

fn record_tooltip_request(
    overlays: &mut WindowOverlays,
    window: AppWindowId,
    request: TooltipRequest,
    owner: Option<GlobalElementId>,
) {
    let w = overlays.windows.entry(window).or_default();
    overlays.cached_tooltip_requests.insert(
        (window, request.id),
        CachedTooltipDecl::from_request(&request, owner),
    );
    w.tooltips.push(request);
}

fn record_toast_layer_request(
    overlays: &mut WindowOverlays,
    window: AppWindowId,
    request: ToastLayerRequest,
    owner: Option<GlobalElementId>,
) {
    let w = overlays.windows.entry(window).or_default();
    overlays.cached_toast_layer_requests.insert(
        (window, request.id),
        CachedToastLayerDecl::from_request(&request, owner),
    );
    w.toasts.push(request);
}

pub(crate) fn request_dismissible_popover_for_window_owned<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: DismissiblePopoverRequest,
    owner: Option<GlobalElementId>,
) {
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        record_dismissible_popover_request(overlays, window, request, owner);
    });
}

pub(crate) fn request_modal_for_window_owned<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: ModalRequest,
    owner: Option<GlobalElementId>,
) {
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        record_modal_request(overlays, window, request, owner);
    });
}

pub(crate) fn request_hover_overlay_for_window_owned<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: HoverOverlayRequest,
    owner: Option<GlobalElementId>,
) {
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        record_hover_overlay_request(overlays, window, request, owner);
    });
}

pub(crate) fn request_tooltip_for_window_owned<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: TooltipRequest,
    owner: Option<GlobalElementId>,
) {
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        record_tooltip_request(overlays, window, request, owner);
    });
}

pub(crate) fn request_toast_layer_for_window_owned<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: ToastLayerRequest,
    owner: Option<GlobalElementId>,
) {
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        record_toast_layer_request(overlays, window, request, owner);
    });
}

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
    request_dismissible_popover_for_window_owned(cx.app, cx.window, request, Some(cx.root_id()));
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn request_dismissible_popover_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: DismissiblePopoverRequest,
) {
    request_dismissible_popover_for_window_owned(app, window, request, None);
}

#[cfg(feature = "unstable-internals")]
pub fn request_modal<H: UiHost>(cx: &mut ElementContext<'_, H>, request: ModalRequest) {
    request_modal_for_window_owned(cx.app, cx.window, request, Some(cx.root_id()));
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn request_modal_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: ModalRequest,
) {
    request_modal_for_window_owned(app, window, request, None);
}

#[cfg(feature = "unstable-internals")]
pub fn request_hover_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    request: HoverOverlayRequest,
) {
    request_hover_overlay_for_window_owned(cx.app, cx.window, request, Some(cx.root_id()));
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn request_hover_overlay_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: HoverOverlayRequest,
) {
    request_hover_overlay_for_window_owned(app, window, request, None);
}

#[cfg(feature = "unstable-internals")]
pub fn request_tooltip<H: UiHost>(cx: &mut ElementContext<'_, H>, request: TooltipRequest) {
    request_tooltip_for_window_owned(cx.app, cx.window, request, Some(cx.root_id()));
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn request_tooltip_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: TooltipRequest,
) {
    request_tooltip_for_window_owned(app, window, request, None);
}

#[cfg(feature = "unstable-internals")]
pub fn request_toast_layer<H: UiHost>(cx: &mut ElementContext<'_, H>, request: ToastLayerRequest) {
    request_toast_layer_for_window_owned(cx.app, cx.window, request, Some(cx.root_id()));
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn request_toast_layer_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: ToastLayerRequest,
) {
    request_toast_layer_for_window_owned(app, window, request, None);
}
