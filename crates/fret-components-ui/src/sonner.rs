//! Sonner-style toast facade (shadcn/ui vocabulary).
//!
//! shadcn/ui exports a `Toaster` component and a `toast(...)` imperative API.
//! In Fret, the renderer-facing overlay (`ToastOverlay`) is installed by `WindowOverlays`,
//! so this module focuses on making the call sites ergonomic and consistent.

use fret_core::AppWindowId;
use fret_ui::{ToastAction, ToastKind, ToastRequest, ToastService, UiHost};

pub fn toast<H: UiHost>(app: &mut H, window: AppWindowId, request: ToastRequest) {
    app.with_global_mut(ToastService::default, |svc, app| {
        svc.push(app, window, request);
    });
}

pub fn toast_success<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    title: impl Into<std::sync::Arc<str>>,
) {
    toast(
        app,
        window,
        ToastRequest::new(title).kind(ToastKind::Success),
    );
}

pub fn toast_info<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    title: impl Into<std::sync::Arc<str>>,
) {
    toast(app, window, ToastRequest::new(title).kind(ToastKind::Info));
}

pub fn toast_warning<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    title: impl Into<std::sync::Arc<str>>,
) {
    toast(
        app,
        window,
        ToastRequest::new(title).kind(ToastKind::Warning),
    );
}

pub fn toast_error<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    title: impl Into<std::sync::Arc<str>>,
) {
    toast(app, window, ToastRequest::new(title).kind(ToastKind::Error));
}

pub fn toast_action<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    title: impl Into<std::sync::Arc<str>>,
    description: impl Into<std::sync::Arc<str>>,
    action: ToastAction,
) {
    toast(
        app,
        window,
        ToastRequest::new(title)
            .description(description)
            .action(action),
    );
}
