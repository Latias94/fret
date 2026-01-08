//! Radix-aligned toast primitives.
//!
//! Upstream reference: `@radix-ui/react-toast` (`repo-ref/primitives/packages/react/toast`).
//!
//! Fret does not model DOM portals or ARIA live region announcement yet. This module focuses on
//! the reusable core outcomes:
//! - a per-window toast store with upsert-by-id
//! - a viewport root installed as a window overlay layer
//! - optional max-toasts limiting
//!
//! The shadcn `Sonner` wrapper builds on top of this substrate.

use fret_core::{AppWindowId, Px};
use fret_runtime::Model;
use fret_ui::action::UiActionHost;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::window_overlays;
use crate::{OverlayController, OverlayRequest};

pub use window_overlays::{
    DEFAULT_MAX_TOASTS, ToastAction, ToastId, ToastPosition, ToastRequest, ToastStore, ToastVariant,
};

#[derive(Debug, Clone, Copy)]
pub struct ToastViewport {
    position: ToastPosition,
    margin: Option<Px>,
    gap: Option<Px>,
    toast_min_width: Option<Px>,
    toast_max_width: Option<Px>,
    max_toasts: Option<usize>,
}

#[derive(Debug, Default)]
struct ToastViewportConfigState {
    max_toasts: Option<usize>,
}

impl Default for ToastViewport {
    fn default() -> Self {
        Self {
            position: ToastPosition::BottomRight,
            margin: None,
            gap: None,
            toast_min_width: None,
            toast_max_width: None,
            max_toasts: Some(DEFAULT_MAX_TOASTS),
        }
    }
}

impl ToastViewport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn margin(mut self, margin: Px) -> Self {
        self.margin = Some(margin);
        self
    }

    pub fn gap(mut self, gap: Px) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn toast_min_width(mut self, width: Px) -> Self {
        self.toast_min_width = Some(width);
        self
    }

    pub fn toast_max_width(mut self, width: Px) -> Self {
        self.toast_max_width = Some(width);
        self
    }

    pub fn max_toasts(mut self, max_toasts: usize) -> Self {
        self.max_toasts = Some(max_toasts.max(1));
        self
    }

    pub fn unlimited(mut self) -> Self {
        self.max_toasts = None;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let id = cx.root_id();
            let store = OverlayController::toast_store(&mut *cx.app);

            let config_changed = cx.with_state(ToastViewportConfigState::default, |st| {
                if st.max_toasts == self.max_toasts {
                    return false;
                }
                st.max_toasts = self.max_toasts;
                true
            });
            if config_changed {
                let _ = cx.app.models_mut().update(&store, |st| {
                    st.set_window_max_toasts(cx.window, self.max_toasts)
                });
            }

            let mut request = OverlayRequest::toast_layer(id, store).toast_position(self.position);
            if let Some(margin) = self.margin {
                request = request.toast_margin(margin);
            }
            if let Some(gap) = self.gap {
                request = request.toast_gap(gap);
            }
            if let Some(width) = self.toast_min_width {
                request = request.toast_min_width(width);
            }
            if let Some(width) = self.toast_max_width {
                request = request.toast_max_width(width);
            }
            OverlayController::request(cx, request);

            cx.stack(|_cx| Vec::new())
        })
    }
}

#[derive(Clone)]
pub struct ToastController {
    store: Model<ToastStore>,
}

impl std::fmt::Debug for ToastController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToastController")
            .field("store", &"<model>")
            .finish()
    }
}

impl ToastController {
    pub fn global<H: UiHost>(app: &mut H) -> Self {
        Self {
            store: OverlayController::toast_store(app),
        }
    }

    /// Dispatches a toast request.
    ///
    /// Note: this is an upsert. If `request.id` is set and still refers to an open toast, the
    /// existing toast is updated.
    pub fn toast(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        request: ToastRequest,
    ) -> ToastId {
        OverlayController::toast_action(host, self.store.clone(), window, request)
    }

    pub fn dismiss(&self, host: &mut dyn UiActionHost, window: AppWindowId, id: ToastId) -> bool {
        OverlayController::dismiss_toast_action(host, self.store.clone(), window, id)
    }
}
