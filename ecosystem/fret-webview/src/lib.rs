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

use std::sync::Arc;

use fret_core::{AppWindowId, Rect, SemanticsSnapshot};

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
