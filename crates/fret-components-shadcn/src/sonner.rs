use fret_components_ui::window_overlays;
use fret_core::AppWindowId;
use fret_runtime::Model;
use fret_ui::action::UiActionHost;
use fret_ui::element::AnyElement;
use fret_ui::{ElementCx, UiHost};

#[derive(Debug, Clone, Copy)]
pub struct Toaster {
    position: window_overlays::ToastPosition,
}

impl Default for Toaster {
    fn default() -> Self {
        Self {
            position: window_overlays::ToastPosition::BottomRight,
        }
    }
}

impl Toaster {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn position(mut self, position: window_overlays::ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let id = cx.root_id();
            let store = window_overlays::toast_store(&mut *cx.app);
            window_overlays::request_toast_layer(
                cx,
                window_overlays::ToastLayerRequest::new(id, store).position(self.position),
            );

            cx.stack(|_cx| Vec::new())
        })
    }
}

#[derive(Clone)]
pub struct Sonner {
    store: Model<window_overlays::ToastStore>,
}

impl std::fmt::Debug for Sonner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sonner").field("store", &"<model>").finish()
    }
}

impl Sonner {
    pub fn global<H: UiHost>(app: &mut H) -> Self {
        Self {
            store: window_overlays::toast_store(app),
        }
    }

    pub fn toast(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        request: window_overlays::ToastRequest,
    ) -> window_overlays::ToastId {
        window_overlays::toast_action(host, self.store.clone(), window, request)
    }

    pub fn dismiss(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        id: window_overlays::ToastId,
    ) -> bool {
        window_overlays::dismiss_toast_action(host, self.store.clone(), window, id)
    }
}

pub use window_overlays::{ToastAction, ToastId, ToastPosition, ToastRequest, ToastVariant};
