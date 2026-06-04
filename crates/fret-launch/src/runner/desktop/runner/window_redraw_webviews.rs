use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, SemanticsSnapshot};
use fret_runtime::FrameId;
use winit::window::Window;

use super::WinitAppDriver;
use super::webview::RunnerWebViewState;

pub(super) struct WindowRedrawWebViewSyncInput<'a, D: WinitAppDriver> {
    pub(super) app: &'a mut App,
    pub(super) driver: &'a mut D,
    pub(super) webviews: &'a mut RunnerWebViewState,
    pub(super) frame_id: FrameId,
    pub(super) app_window: AppWindowId,
    pub(super) user: &'a mut D::WindowState,
    pub(super) window: &'a dyn Window,
    pub(super) last_semantics_snapshot: &'a Option<Arc<SemanticsSnapshot>>,
}

pub(super) fn sync_window_redraw_webviews<D: WinitAppDriver>(
    input: WindowRedrawWebViewSyncInput<'_, D>,
) {
    let webview_snapshot = window_redraw_webview_snapshot(
        input.app,
        input.driver,
        input.app_window,
        input.user,
        input.last_semantics_snapshot,
    );

    input.webviews.sync_window(
        input.app,
        input.frame_id,
        input.app_window,
        input.window,
        webview_snapshot.as_ref(),
    );
}

#[cfg(feature = "webview-wry")]
fn window_redraw_webview_snapshot<D: WinitAppDriver>(
    app: &mut App,
    driver: &mut D,
    app_window: AppWindowId,
    user: &mut D::WindowState,
    last_semantics_snapshot: &Option<Arc<SemanticsSnapshot>>,
) -> Option<Arc<SemanticsSnapshot>> {
    if app.global::<fret_webview::WebViewHost>().is_some()
        && fret_webview::webview_has_surfaces_for_window(app, app_window)
    {
        last_semantics_snapshot
            .clone()
            .or_else(|| driver.semantics_snapshot(app, app_window, user))
    } else {
        None
    }
}

#[cfg(not(feature = "webview-wry"))]
fn window_redraw_webview_snapshot<D: WinitAppDriver>(
    _app: &mut App,
    _driver: &mut D,
    _app_window: AppWindowId,
    _user: &mut D::WindowState,
    _last_semantics_snapshot: &Option<Arc<SemanticsSnapshot>>,
) -> Option<Arc<SemanticsSnapshot>> {
    None
}
