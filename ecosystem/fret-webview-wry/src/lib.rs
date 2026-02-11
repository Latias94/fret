//! Native WebView backend implementation (planned: `wry`).
//!
//! This crate is intended to provide a concrete implementation for the request/event contract
//! defined in `fret-webview`.
//!
//! Notes:
//!
//! - The actual `wry` dependency is intentionally deferred until we decide how to integrate with
//!   the current runner/window stack (winit/tao compatibility, multi-window lifetimes, z-order).
//! - Until then, this crate provides a small, testable in-memory driver that can be used to wire
//!   UI/policy surfaces to a backend interface without committing to a platform implementation.
//!
//! Workstream: `docs/workstreams/webview-wry-v1.md`.

use std::collections::VecDeque;

use fret_webview::{WebViewEvent, WebViewId, WebViewRequest};

#[cfg(feature = "wry")]
pub mod wry_backend;

/// Minimal, backend-agnostic interface used by hosts to drive a WebView backend.
///
/// A concrete `wry` implementation should live in this crate once the runner glue strategy is
/// decided. Keeping the trait local lets us evolve it without forcing all consumers to update at
/// once.
pub trait WebViewBackend {
    fn push_request(&mut self, request: WebViewRequest);
    fn drain_events(&mut self) -> Vec<WebViewEvent>;
}

/// In-memory driver used as a placeholder backend while the `wry` integration is designed.
///
/// This is not a real WebView implementation; it only records requests and can be used to inject
/// events for UI validation tests.
#[derive(Debug, Default)]
pub struct WebViewBackendStub {
    requests: VecDeque<WebViewRequest>,
    events: VecDeque<WebViewEvent>,
}

impl WebViewBackendStub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn drain_requests(&mut self) -> Vec<WebViewRequest> {
        self.requests.drain(..).collect()
    }

    pub fn push_event(&mut self, event: WebViewEvent) {
        self.events.push_back(event);
    }

    pub fn created(&mut self, id: WebViewId) {
        self.push_event(WebViewEvent::Created { id });
    }
}

impl WebViewBackend for WebViewBackendStub {
    fn push_request(&mut self, request: WebViewRequest) {
        self.requests.push_back(request);
    }

    fn drain_events(&mut self) -> Vec<WebViewEvent> {
        self.events.drain(..).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret_core::AppWindowId;

    use super::*;

    #[test]
    fn stub_round_trips_requests_and_events() {
        let mut stub = WebViewBackendStub::new();
        let id = WebViewId(1);

        stub.push_request(WebViewRequest::Create {
            id,
            window: AppWindowId::default(),
            initial_url: Arc::<str>::from("https://example.com"),
        });
        stub.created(id);

        let reqs = stub.drain_requests();
        assert_eq!(reqs.len(), 1);

        let evs = stub.drain_events();
        assert_eq!(evs, vec![WebViewEvent::Created { id }]);
    }
}
