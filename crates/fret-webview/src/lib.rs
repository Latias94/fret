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
use fret_runtime::{GlobalsHost, TimeHost};

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

/// Runtime-observable state for a hosted WebView instance.
///
/// This is derived from backend-emitted [`WebViewEvent`]s and is intended to be polled by
/// UI/policy code (e.g. to disable navigation buttons).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WebViewRuntimeState {
    pub url: Option<Arc<str>>,
    pub title: Option<Arc<str>>,
    pub navigation: WebViewNavigationState,
    pub last_error: Option<Arc<str>>,
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
    events: VecDeque<WebViewEvent>,
    surfaces: HashMap<WebViewId, WebViewSurfaceRegistration>,
    runtime: HashMap<WebViewId, WebViewRuntimeState>,
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

    pub fn push_event(&mut self, event: WebViewEvent) {
        self.apply_event_to_runtime(&event);
        self.events.push_back(event);
    }

    pub fn push_events(&mut self, events: impl IntoIterator<Item = WebViewEvent>) {
        for ev in events {
            self.push_event(ev);
        }
    }

    pub fn drain_events(&mut self) -> Vec<WebViewEvent> {
        self.events.drain(..).collect()
    }

    pub fn runtime_state(&self, id: WebViewId) -> Option<&WebViewRuntimeState> {
        self.runtime.get(&id)
    }

    pub fn requeue_requests_front(&mut self, requests: impl IntoIterator<Item = WebViewRequest>) {
        let mut head: VecDeque<WebViewRequest> = requests.into_iter().collect();
        head.append(&mut self.requests);
        self.requests = head;
    }

    pub fn drop_requests_for_ids(&mut self, ids: &[WebViewId]) -> usize {
        use std::collections::HashSet;

        let ids: HashSet<WebViewId> = ids.iter().copied().collect();
        let before = self.requests.len();
        self.requests.retain(|req| match req {
            WebViewRequest::Create { id, .. } => !ids.contains(id),
            WebViewRequest::Destroy { id } => !ids.contains(id),
            WebViewRequest::SetPlacement { id, .. } => !ids.contains(id),
            WebViewRequest::LoadUrl { id, .. } => !ids.contains(id),
            WebViewRequest::GoBack { id } => !ids.contains(id),
            WebViewRequest::GoForward { id } => !ids.contains(id),
            WebViewRequest::Reload { id } => !ids.contains(id),
        });
        before.saturating_sub(self.requests.len())
    }

    pub fn drop_requests_for_window_close(
        &mut self,
        window: AppWindowId,
        ids: &[WebViewId],
    ) -> usize {
        use std::collections::HashSet;

        let ids: HashSet<WebViewId> = ids.iter().copied().collect();
        let before = self.requests.len();
        self.requests.retain(|req| match req {
            WebViewRequest::Create { id, window: w, .. } => *w != window && !ids.contains(id),
            WebViewRequest::Destroy { id } => !ids.contains(id),
            WebViewRequest::SetPlacement { id, .. } => !ids.contains(id),
            WebViewRequest::LoadUrl { id, .. } => !ids.contains(id),
            WebViewRequest::GoBack { id } => !ids.contains(id),
            WebViewRequest::GoForward { id } => !ids.contains(id),
            WebViewRequest::Reload { id } => !ids.contains(id),
        });
        before.saturating_sub(self.requests.len())
    }

    pub fn register_surface(&mut self, surface: WebViewSurfaceRegistration) {
        self.surfaces.insert(surface.id, surface);
    }

    pub fn remove_surfaces_for_window(
        &mut self,
        window: AppWindowId,
    ) -> Vec<WebViewSurfaceRegistration> {
        let ids = self
            .surfaces
            .iter()
            .filter_map(|(id, s)| (s.window == window).then_some(*id))
            .collect::<Vec<_>>();

        let mut removed = Vec::new();
        for id in ids {
            if let Some(s) = self.surfaces.remove(&id) {
                removed.push(s);
            }
            self.runtime.remove(&id);
        }
        removed
    }

    pub fn gc_stale_surfaces(
        &mut self,
        window: AppWindowId,
        now_frame: u64,
        grace_frames: u64,
    ) -> Vec<WebViewId> {
        let stale = self
            .surfaces
            .values()
            .filter(|s| {
                s.window == window
                    && now_frame.saturating_sub(s.last_registered_frame) > grace_frames
            })
            .map(|s| s.id)
            .collect::<Vec<_>>();

        if !stale.is_empty() {
            let _ = self.drop_requests_for_ids(&stale);
        }

        for id in &stale {
            self.surfaces.remove(id);
            self.runtime.remove(id);
            self.requests.push_back(WebViewRequest::Destroy { id: *id });
        }

        stale
    }

    fn apply_event_to_runtime(&mut self, event: &WebViewEvent) {
        match event {
            WebViewEvent::Created { id } => {
                self.runtime.insert(*id, WebViewRuntimeState::default());
            }
            WebViewEvent::Destroyed { id } => {
                self.runtime.remove(id);
                self.surfaces.remove(id);
            }
            WebViewEvent::UrlChanged { id, url } => {
                self.runtime
                    .entry(*id)
                    .or_default()
                    .url
                    .replace(url.clone());
            }
            WebViewEvent::TitleChanged { id, title } => {
                self.runtime
                    .entry(*id)
                    .or_default()
                    .title
                    .replace(title.clone());
            }
            WebViewEvent::NavigationStateChanged { id, state } => {
                self.runtime.entry(*id).or_default().navigation = *state;
            }
            WebViewEvent::ConsoleMessage { .. } => {}
            WebViewEvent::LoadFailed { id, error } => {
                self.runtime
                    .entry(*id)
                    .or_default()
                    .last_error
                    .replace(error.clone());
                self.runtime.entry(*id).or_default().navigation.is_loading = false;
            }
        }
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
    pub last_registered_frame: u64,
}

impl WebViewSurfaceRegistration {
    pub fn new(id: WebViewId, window: AppWindowId, surface_test_id: impl Into<Arc<str>>) -> Self {
        Self {
            id,
            window,
            surface_test_id: surface_test_id.into(),
            visible: true,
            last_registered_frame: 0,
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

pub fn webview_push_event(host: &mut impl GlobalsHost, event: WebViewEvent) {
    with_webview_host_mut(host, |st| st.push_event(event));
}

pub fn webview_push_events(
    host: &mut impl GlobalsHost,
    events: impl IntoIterator<Item = WebViewEvent>,
) {
    with_webview_host_mut(host, |st| st.push_events(events));
}

pub fn webview_drain_events(host: &mut impl GlobalsHost) -> Vec<WebViewEvent> {
    with_webview_host_mut(host, |st| st.drain_events())
}

pub fn webview_runtime_state(
    host: &impl GlobalsHost,
    id: WebViewId,
) -> Option<WebViewRuntimeState> {
    with_webview_host(host, |st| st.and_then(|st| st.runtime_state(id).cloned()))
}

pub fn webview_register_surface(host: &mut impl GlobalsHost, surface: WebViewSurfaceRegistration) {
    with_webview_host_mut(host, |st| st.register_surface(surface));
}

pub fn webview_register_surface_tracked(
    host: &mut (impl GlobalsHost + TimeHost),
    mut surface: WebViewSurfaceRegistration,
) {
    surface.last_registered_frame = host.frame_id().0;
    with_webview_host_mut(host, |st| st.register_surface(surface));
}

pub fn webview_remove_surfaces_for_window(
    host: &mut impl GlobalsHost,
    window: AppWindowId,
) -> Vec<WebViewSurfaceRegistration> {
    with_webview_host_mut(host, |st| st.remove_surfaces_for_window(window))
}

pub fn webview_drop_requests_for_ids(host: &mut impl GlobalsHost, ids: &[WebViewId]) -> usize {
    with_webview_host_mut(host, |st| st.drop_requests_for_ids(ids))
}

pub fn webview_drop_requests_for_window_close(
    host: &mut impl GlobalsHost,
    window: AppWindowId,
    ids: &[WebViewId],
) -> usize {
    with_webview_host_mut(host, |st| st.drop_requests_for_window_close(window, ids))
}

pub fn webview_gc_stale_surfaces(
    host: &mut impl GlobalsHost,
    window: AppWindowId,
    now_frame: u64,
    grace_frames: u64,
) -> Vec<WebViewId> {
    with_webview_host_mut(host, |st| {
        st.gc_stale_surfaces(window, now_frame, grace_frames)
    })
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

    #[test]
    fn push_event_updates_runtime_state() {
        let mut host = WebViewHost::default();
        let id = WebViewId(1);

        host.push_event(WebViewEvent::Created { id });
        host.push_event(WebViewEvent::UrlChanged {
            id,
            url: Arc::<str>::from("https://example.com"),
        });
        host.push_event(WebViewEvent::TitleChanged {
            id,
            title: Arc::<str>::from("Example Domain"),
        });
        host.push_event(WebViewEvent::NavigationStateChanged {
            id,
            state: WebViewNavigationState {
                can_go_back: false,
                can_go_forward: false,
                is_loading: true,
            },
        });

        let st = host.runtime_state(id).expect("runtime state must exist");
        assert_eq!(st.url.as_deref(), Some("https://example.com"));
        assert_eq!(st.title.as_deref(), Some("Example Domain"));
        assert!(st.navigation.is_loading);
    }

    #[test]
    fn gc_stale_surfaces_drops_requests_only_for_stale_ids() {
        let window = AppWindowId::default();
        let id_stale = WebViewId(1);
        let id_live = WebViewId(2);

        let mut host = WebViewHost::default();

        let mut stale = WebViewSurfaceRegistration::new(id_stale, window, "surface-stale");
        stale.last_registered_frame = 0;
        host.register_surface(stale);

        let mut live = WebViewSurfaceRegistration::new(id_live, window, "surface-live");
        live.last_registered_frame = 10;
        host.register_surface(live);

        host.push_request(WebViewRequest::Create {
            id: id_stale,
            window,
            initial_url: Arc::<str>::from("https://example.com"),
        });
        host.push_request(WebViewRequest::Create {
            id: id_live,
            window,
            initial_url: Arc::<str>::from("https://example.org"),
        });

        let stale_ids = host.gc_stale_surfaces(window, 10, 2);
        assert_eq!(stale_ids, vec![id_stale]);

        let requests = host.drain_requests();
        assert_eq!(
            requests,
            vec![
                WebViewRequest::Create {
                    id: id_live,
                    window,
                    initial_url: Arc::<str>::from("https://example.org"),
                },
                WebViewRequest::Destroy { id: id_stale },
            ]
        );
    }

    #[test]
    fn drop_requests_for_window_close_drops_creates_for_window() {
        let window_a = AppWindowId::from(slotmap::KeyData::from_ffi(1));
        let window_b = AppWindowId::from(slotmap::KeyData::from_ffi(2));
        let id_a1 = WebViewId(1);
        let id_a2 = WebViewId(2);
        let id_b = WebViewId(3);

        let mut host = WebViewHost::default();
        host.push_request(WebViewRequest::Create {
            id: id_a1,
            window: window_a,
            initial_url: Arc::<str>::from("https://a1"),
        });
        host.push_request(WebViewRequest::Create {
            id: id_a2,
            window: window_a,
            initial_url: Arc::<str>::from("https://a2"),
        });
        host.push_request(WebViewRequest::Create {
            id: id_b,
            window: window_b,
            initial_url: Arc::<str>::from("https://b"),
        });

        let dropped = host.drop_requests_for_window_close(window_a, &[id_a1]);
        assert_eq!(dropped, 2);

        let remaining = host.drain_requests();
        assert_eq!(
            remaining,
            vec![WebViewRequest::Create {
                id: id_b,
                window: window_b,
                initial_url: Arc::<str>::from("https://b"),
            }]
        );
    }
}
