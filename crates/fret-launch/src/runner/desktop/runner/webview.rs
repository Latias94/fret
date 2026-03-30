use std::sync::Arc;

use fret_app::App;
use fret_core::AppWindowId;
use fret_runtime::FrameId;
use winit::window::Window;

#[derive(Debug, Default)]
pub(super) struct RunnerWebViewState {
    #[cfg(feature = "webview-wry")]
    host: fret_webview_wry::WryWebViewHost,
}

#[cfg(feature = "webview-wry")]
const WEBVIEW_STALE_SURFACE_GRACE_FRAMES: u64 = 2;

impl RunnerWebViewState {
    pub(super) fn sync_window(
        &mut self,
        app: &mut App,
        frame_id: FrameId,
        window: AppWindowId,
        native_window: &dyn Window,
        snapshot: Option<&Arc<fret_core::SemanticsSnapshot>>,
    ) {
        #[cfg(feature = "webview-wry")]
        {
            use fret_webview::{
                WebViewHost, webview_apply_events, webview_drain_requests,
                webview_gc_stale_surfaces, webview_requeue_requests_front,
                webview_surfaces_for_window,
            };

            if app.global::<WebViewHost>().is_none() {
                return;
            }

            let _ = webview_gc_stale_surfaces(
                app,
                window,
                frame_id.0,
                WEBVIEW_STALE_SURFACE_GRACE_FRAMES,
            );

            let mut requests = webview_drain_requests(app);
            requests.extend(build_placement_requests(
                window,
                snapshot.map(|value| value.as_ref()),
                webview_surfaces_for_window(app, window),
            ));

            let unhandled = self
                .host
                .handle_requests_for_window(window, native_window, requests);
            if !unhandled.is_empty() {
                webview_requeue_requests_front(app, unhandled);
            }

            let events = self.host.drain_events_for_window(window);
            if !events.is_empty() {
                webview_apply_events(app, events);
            }
        }

        #[cfg(not(feature = "webview-wry"))]
        {
            let _ = (app, frame_id, window, native_window, snapshot);
        }
    }

    pub(super) fn close_window(&mut self, app: &mut App, window: AppWindowId) {
        #[cfg(feature = "webview-wry")]
        {
            use fret_webview::{
                WebViewHost, webview_apply_events, webview_drop_requests_for_window_close,
                webview_remove_surfaces_for_window, webview_surfaces_for_window,
            };

            let had_host = app.global::<WebViewHost>().is_some();
            if had_host {
                let ids = webview_surfaces_for_window(app, window)
                    .into_iter()
                    .map(|surface| surface.id)
                    .collect::<Vec<_>>();
                let _ = webview_drop_requests_for_window_close(app, window, &ids);
                let _ = webview_remove_surfaces_for_window(app, window);
            }

            let events = self.host.destroy_all_for_window(window);
            if had_host && !events.is_empty() {
                webview_apply_events(app, events);
            }
        }

        #[cfg(not(feature = "webview-wry"))]
        {
            let _ = (app, window);
        }
    }
}

#[cfg(feature = "webview-wry")]
fn build_placement_requests(
    window: AppWindowId,
    snapshot: Option<&fret_core::SemanticsSnapshot>,
    surfaces: Vec<fret_webview::WebViewSurfaceRegistration>,
) -> Vec<fret_webview::WebViewRequest> {
    surfaces
        .into_iter()
        .map(|surface| placement_request_for_surface(window, snapshot, surface))
        .collect()
}

#[cfg(feature = "webview-wry")]
fn placement_request_for_surface(
    window: AppWindowId,
    snapshot: Option<&fret_core::SemanticsSnapshot>,
    surface: fret_webview::WebViewSurfaceRegistration,
) -> fret_webview::WebViewRequest {
    use fret_webview::{WebViewPlacement, WebViewRequest, best_bounds_for_test_id};

    let placement = snapshot
        .and_then(|snapshot| best_bounds_for_test_id(snapshot, surface.surface_test_id.as_ref()))
        .map(|bounds| WebViewPlacement::new(window, bounds).visible(surface.visible))
        .unwrap_or_else(|| {
            WebViewPlacement::new(window, fret_core::Rect::default()).visible(false)
        });

    WebViewRequest::SetPlacement {
        id: surface.id,
        placement,
    }
}

#[cfg(all(test, feature = "webview-wry"))]
mod tests {
    use fret_core::{AppWindowId, NodeId, Point, Px, Rect, SemanticsNode, SemanticsRole, Size};
    use fret_webview::{WebViewId, WebViewRequest, WebViewSurfaceRegistration};

    use super::{build_placement_requests, placement_request_for_surface};

    #[test]
    fn missing_anchor_hides_surface() {
        let window = AppWindowId::default();
        let request = placement_request_for_surface(
            window,
            Some(&fret_core::SemanticsSnapshot {
                window,
                ..Default::default()
            }),
            WebViewSurfaceRegistration::new(WebViewId(1), window, "missing-anchor"),
        );

        let WebViewRequest::SetPlacement { placement, .. } = request else {
            panic!("expected placement request");
        };

        assert!(!placement.visible);
        assert_eq!(placement.bounds, Rect::default());
    }

    #[test]
    fn anchor_bounds_are_forwarded_to_runner_request() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(30.0), Px(40.0)),
        );
        let mut snapshot = fret_core::SemanticsSnapshot {
            window,
            ..Default::default()
        };
        snapshot.nodes.push(SemanticsNode {
            id: NodeId::from(slotmap::KeyData::from_ffi(1)),
            parent: None,
            role: SemanticsRole::Generic,
            bounds,
            flags: Default::default(),
            test_id: Some(String::from("webview-surface")),
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            extra: fret_core::SemanticsNodeExtra::default(),
            text_selection: None,
            text_composition: None,
            actions: Default::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        });

        let requests = build_placement_requests(
            window,
            Some(&snapshot),
            vec![WebViewSurfaceRegistration::new(
                WebViewId(7),
                window,
                "webview-surface",
            )],
        );

        let [WebViewRequest::SetPlacement { placement, .. }] = requests.as_slice() else {
            panic!("expected a single placement request");
        };

        assert!(placement.visible);
        assert_eq!(placement.bounds, bounds);
    }
}
