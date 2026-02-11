//! Contract types for embedding an external WebView backend into Fret.
//!
//! This crate is intentionally **contract-only**:
//!
//! - No heavy platform dependencies (`wry`, WebView2, WKWebView, etc.).
//! - No UI composition surfaces (those belong in `fret-ui-ai` / `fret-ui-shadcn`).
//! - No runner integration (backend crates should own that wiring).
//!
//! Initial consumer: `fret-ui-ai` (`WebPreview`).
//!
//! Workstream: `docs/workstreams/webview-wry-v1.md`.

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use fret_core::{AppWindowId, Rect, SemanticsSnapshot};
use fret_runtime::GlobalsHost;

/// Stable identifier for a hosted webview instance.
///
/// This ID is assigned by the app/host and must remain stable while the instance is alive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WebViewId(pub u64);

/// Desired placement of a WebView within a window.
///
/// Coordinates are in window space, expressed in **logical pixels** (see ADR 0017).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebViewPlacement {
    pub window: AppWindowId,
    pub bounds: Rect,
    pub visible: bool,
}

impl WebViewPlacement {
    pub fn new(window: AppWindowId, bounds: Rect) -> Self {
        Self {
            window,
            bounds,
            visible: true,
        }
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

/// Navigation state snapshot emitted by a backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WebViewNavigationState {
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_loading: bool,
}

/// Console log level emitted by a backend (optional).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebViewConsoleLevel {
    Log,
    Warn,
    Error,
}

/// Requests emitted by UI/policy code, to be handled by a concrete backend.
#[derive(Debug, Clone, PartialEq)]
pub enum WebViewRequest {
    Create {
        id: WebViewId,
        window: AppWindowId,
        initial_url: Arc<str>,
    },
    Destroy {
        id: WebViewId,
    },
    SetPlacement {
        id: WebViewId,
        placement: WebViewPlacement,
    },
    LoadUrl {
        id: WebViewId,
        url: Arc<str>,
    },
    GoBack {
        id: WebViewId,
    },
    GoForward {
        id: WebViewId,
    },
    Reload {
        id: WebViewId,
    },
}

/// Events emitted by a backend, to be consumed by UI/policy code.
#[derive(Debug, Clone, PartialEq)]
pub enum WebViewEvent {
    Created {
        id: WebViewId,
    },
    Destroyed {
        id: WebViewId,
    },
    UrlChanged {
        id: WebViewId,
        url: Arc<str>,
    },
    TitleChanged {
        id: WebViewId,
        title: Arc<str>,
    },
    NavigationStateChanged {
        id: WebViewId,
        state: WebViewNavigationState,
    },
    ConsoleMessage {
        id: WebViewId,
        level: WebViewConsoleLevel,
        message: Arc<str>,
    },
    LoadFailed {
        id: WebViewId,
        error: Arc<str>,
    },
}

/// App-global host state used to bridge UI/policy code and runner-owned WebView backends.
///
/// This is a pragmatic v1 integration surface:
///
/// - UI code can register "surfaces" (a `test_id` anchor in the semantics tree) and push requests.
/// - Runner code can drain requests and drive a concrete backend.
///
/// This stays in `fret-webview` (contract crate) so both UI and runner code can use it without
/// creating a dependency edge from the kernel into `ecosystem/*` crates.
#[derive(Debug, Default)]
pub struct WebViewHost {
    requests: VecDeque<WebViewRequest>,
    surfaces: HashMap<WebViewId, WebViewSurfaceRegistration>,
}

impl WebViewHost {
    pub fn push_request(&mut self, request: WebViewRequest) {
        self.requests.push_back(request);
    }

    pub fn push_requests(&mut self, requests: impl IntoIterator<Item = WebViewRequest>) {
        self.requests.extend(requests);
    }

    pub fn drain_requests(&mut self) -> Vec<WebViewRequest> {
        self.requests.drain(..).collect()
    }

    pub fn requeue_requests_front(&mut self, requests: impl IntoIterator<Item = WebViewRequest>) {
        let mut head: VecDeque<WebViewRequest> = requests.into_iter().collect();
        head.append(&mut self.requests);
        self.requests = head;
    }

    pub fn register_surface(&mut self, surface: WebViewSurfaceRegistration) {
        self.surfaces.insert(surface.id, surface);
    }

    pub fn surface(&self, id: WebViewId) -> Option<&WebViewSurfaceRegistration> {
        self.surfaces.get(&id)
    }

    pub fn surfaces_for_window(&self, window: AppWindowId) -> Vec<WebViewSurfaceRegistration> {
        self.surfaces
            .values()
            .filter(|s| s.window == window)
            .cloned()
            .collect()
    }

    pub fn has_surfaces_for_window(&self, window: AppWindowId) -> bool {
        self.surfaces.values().any(|s| s.window == window)
    }
}

/// Stable registration record that ties a hosted WebView to a UI-authored semantics anchor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebViewSurfaceRegistration {
    pub id: WebViewId,
    pub window: AppWindowId,
    pub surface_test_id: Arc<str>,
    pub visible: bool,
}

impl WebViewSurfaceRegistration {
    pub fn new(id: WebViewId, window: AppWindowId, surface_test_id: impl Into<Arc<str>>) -> Self {
        Self {
            id,
            window,
            surface_test_id: surface_test_id.into(),
            visible: true,
        }
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

pub fn with_webview_host_mut<H: GlobalsHost, R>(
    host: &mut H,
    f: impl FnOnce(&mut WebViewHost) -> R,
) -> R {
    host.with_global_mut_untracked(WebViewHost::default, |state, _host| f(state))
}

pub fn with_webview_host<H: GlobalsHost, R>(
    host: &H,
    f: impl FnOnce(Option<&WebViewHost>) -> R,
) -> R {
    f(host.global::<WebViewHost>())
}

pub fn webview_push_request(host: &mut impl GlobalsHost, request: WebViewRequest) {
    with_webview_host_mut(host, |st| st.push_request(request));
}

pub fn webview_push_requests(
    host: &mut impl GlobalsHost,
    requests: impl IntoIterator<Item = WebViewRequest>,
) {
    with_webview_host_mut(host, |st| st.push_requests(requests));
}

pub fn webview_drain_requests(host: &mut impl GlobalsHost) -> Vec<WebViewRequest> {
    with_webview_host_mut(host, |st| st.drain_requests())
}

pub fn webview_requeue_requests_front(
    host: &mut impl GlobalsHost,
    requests: impl IntoIterator<Item = WebViewRequest>,
) {
    with_webview_host_mut(host, |st| st.requeue_requests_front(requests));
}

pub fn webview_register_surface(host: &mut impl GlobalsHost, surface: WebViewSurfaceRegistration) {
    with_webview_host_mut(host, |st| st.register_surface(surface));
}

pub fn webview_has_surfaces_for_window(host: &impl GlobalsHost, window: AppWindowId) -> bool {
    with_webview_host(host, |st| {
        st.is_some_and(|st| st.has_surfaces_for_window(window))
    })
}

pub fn webview_surfaces_for_window(
    host: &impl GlobalsHost,
    window: AppWindowId,
) -> Vec<WebViewSurfaceRegistration> {
    with_webview_host(host, |st| {
        st.map(|st| st.surfaces_for_window(window))
            .unwrap_or_default()
    })
}

/// Returns the best candidate bounds for `test_id` within a window semantics snapshot.
///
/// This is a pragmatic v1 seam: it allows a host/backend to locate a UI-authored "webview surface"
/// by a stable `test_id` and then position a native child view accordingly.
///
/// Selection heuristic:
///
/// - If multiple nodes share the same `test_id`, prefer the node with the **largest area**.
///
/// Notes:
///
/// - `test_id` is explicitly a *debug/automation* field and MUST NOT be treated as an accessibility
///   label.
/// - This helper is intentionally contract-level (backend-agnostic). More explicit anchoring
///   surfaces may replace this in later iterations.
pub fn best_bounds_for_test_id(snapshot: &SemanticsSnapshot, test_id: &str) -> Option<Rect> {
    snapshot
        .nodes
        .iter()
        .filter(|n| n.test_id.as_deref().is_some_and(|v| v == test_id))
        .max_by_key(|n| {
            let w = (n.bounds.size.width.0.max(0.0) * 1000.0) as i64;
            let h = (n.bounds.size.height.0.max(0.0) * 1000.0) as i64;
            let area = w.saturating_mul(h);
            (area, h, w)
        })
        .map(|n| n.bounds)
}

/// Builds a placement request from a window semantics snapshot and a stable `test_id`.
pub fn placement_for_test_id(
    snapshot: &SemanticsSnapshot,
    test_id: &str,
    visible: bool,
) -> Option<WebViewPlacement> {
    let bounds = best_bounds_for_test_id(snapshot, test_id)?;
    Some(WebViewPlacement {
        window: snapshot.window,
        bounds,
        visible,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{NodeId, Point, Px, SemanticsNode, SemanticsRole, Size};

    #[test]
    fn placement_defaults_to_visible() {
        let window = AppWindowId::default();
        let placement = WebViewPlacement::new(
            window,
            Rect::new(Point::new(Px(1.0), Px(2.0)), Size::new(Px(3.0), Px(4.0))),
        );
        assert!(placement.visible);
        assert_eq!(placement.window, window);
    }

    #[test]
    fn best_bounds_for_test_id_prefers_largest_area() {
        let window = AppWindowId::default();
        let mut snapshot = SemanticsSnapshot::default();
        snapshot.window = window;

        snapshot.nodes.push(SemanticsNode {
            id: NodeId::from(slotmap::KeyData::from_ffi(1)),
            parent: None,
            role: SemanticsRole::Generic,
            bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
            flags: Default::default(),
            test_id: Some(String::from("webview-surface")),
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            text_selection: None,
            text_composition: None,
            actions: Default::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
        });

        snapshot.nodes.push(SemanticsNode {
            id: NodeId::from(slotmap::KeyData::from_ffi(2)),
            parent: None,
            role: SemanticsRole::Generic,
            bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(5.0))),
            flags: Default::default(),
            test_id: Some(String::from("webview-surface")),
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            text_selection: None,
            text_composition: None,
            actions: Default::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
        });

        let picked = best_bounds_for_test_id(&snapshot, "webview-surface").expect("must pick");
        assert_eq!(picked.size.width, Px(10.0));
        assert_eq!(picked.size.height, Px(10.0));
    }
}
