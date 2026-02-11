//! `wry`-backed helpers and thin wrappers.
//!
//! This module intentionally does **not** decide how to integrate with the runner/window stack.
//! It only provides small utilities that can be reused once we wire window handles and lifecycle
//! into a concrete backend implementation.

use std::sync::Arc;

use raw_window_handle::HasWindowHandle;
use wry::{
    Rect, WebViewBuilder,
    dpi::{self, LogicalPosition, LogicalSize},
};

use fret_core::{Px, Rect as CoreRect};

/// Builds a `wry::WebView` as a native child of a window.
///
/// This matches the approach used by `repo-ref/gpui-component`:
///
/// - obtain a `raw_window_handle::WindowHandle` from the window
/// - call `WebViewBuilder::build_as_child(...)`
///
/// Notes:
///
/// - This assumes the host provides a window type that implements `HasWindowHandle`.
/// - URL loading policy is backend-owned; this helper only sets an initial URL if provided.
pub fn build_webview_as_child(
    window: &impl HasWindowHandle,
    initial_url: Option<Arc<str>>,
    enable_devtools: bool,
) -> Result<wry::WebView, BuildAsChildError> {
    let handle = window
        .window_handle()
        .map_err(BuildAsChildError::WindowHandle)?;
    let mut builder = WebViewBuilder::new();
    if enable_devtools {
        builder = builder.with_devtools(true);
    }
    if let Some(url) = initial_url {
        builder = builder.with_url(url.as_ref());
    }
    builder
        .build_as_child(&handle)
        .map_err(BuildAsChildError::Wry)
}

#[derive(Debug)]
pub enum BuildAsChildError {
    WindowHandle(raw_window_handle::HandleError),
    Wry(wry::Error),
}

/// Converts a Fret logical-pixel `Rect` to a wry logical `Rect`.
pub fn wry_rect_from_core(bounds: CoreRect) -> Rect {
    Rect {
        position: dpi::Position::Logical(LogicalPosition::new(
            bounds.origin.x.0.into(),
            bounds.origin.y.0.into(),
        )),
        size: dpi::Size::Logical(LogicalSize::new(
            bounds.size.width.0.into(),
            bounds.size.height.0.into(),
        )),
    }
}

/// A small wrapper around `wry::WebView` that uses Fret's coordinate vocabulary.
pub struct WryWebView {
    inner: wry::WebView,
}

impl std::fmt::Debug for WryWebView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WryWebView")
            .field("inner", &"<wry::WebView>")
            .finish()
    }
}

impl WryWebView {
    pub fn new(inner: wry::WebView) -> Self {
        Self { inner }
    }

    pub fn set_visible(&self, visible: bool) -> Result<(), wry::Error> {
        self.inner.set_visible(visible)
    }

    pub fn set_bounds_logical(&self, bounds: CoreRect) -> Result<(), wry::Error> {
        self.inner.set_bounds(wry_rect_from_core(bounds))
    }

    pub fn focus_parent(&self) -> Result<(), wry::Error> {
        self.inner.focus_parent()
    }

    pub fn load_url(&self, url: &str) -> Result<(), wry::Error> {
        self.inner.load_url(url)
    }

    pub fn go_back(&self) -> Result<(), wry::Error> {
        self.inner.evaluate_script("history.back();").map(|_| ())
    }

    pub fn go_forward(&self) -> Result<(), wry::Error> {
        self.inner.evaluate_script("history.forward();").map(|_| ())
    }

    pub fn reload(&self) -> Result<(), wry::Error> {
        self.inner.evaluate_script("location.reload();").map(|_| ())
    }

    pub fn device_pixel_ratio_hint(_scale_factor: f32) -> Px {
        // Placeholder: Fret expresses layout in logical pixels (ADR 0017). The backend integration
        // should translate to physical pixels if/when the embedding API requires it.
        Px(1.0)
    }

    pub fn raw(&self) -> &wry::WebView {
        &self.inner
    }
}
