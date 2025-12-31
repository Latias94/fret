use fret_components_ui::{OverlayController, OverlayRequest, ToastStore};
use fret_core::AppWindowId;
use fret_runtime::Model;
use fret_ui::action::UiActionHost;
use fret_ui::element::AnyElement;
use fret_ui::{ElementCx, UiHost};

#[derive(Debug, Clone, Copy)]
pub struct Toaster {
    position: ToastPosition,
}

impl Default for Toaster {
    fn default() -> Self {
        Self {
            position: ToastPosition::BottomRight,
        }
    }
}

impl Toaster {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let id = cx.root_id();
            let store = OverlayController::toast_store(&mut *cx.app);
            let request = OverlayRequest::toast_layer(id, store).toast_position(self.position);
            OverlayController::request(cx, request);

            cx.stack(|_cx| Vec::new())
        })
    }
}

#[derive(Clone)]
pub struct Sonner {
    store: Model<ToastStore>,
}

impl std::fmt::Debug for Sonner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sonner").field("store", &"<model>").finish()
    }
}

impl Sonner {
    pub fn global<H: UiHost>(app: &mut H) -> Self {
        Self {
            store: OverlayController::toast_store(app),
        }
    }

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

pub use fret_components_ui::{ToastAction, ToastId, ToastPosition, ToastRequest, ToastVariant};
