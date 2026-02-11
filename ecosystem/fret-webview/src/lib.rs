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

use fret_core::{AppWindowId, Rect};

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

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Px, Size};

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
}
