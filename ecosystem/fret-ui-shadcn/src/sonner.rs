use fret_core::AppWindowId;
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::action::UiActionHost;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{OverlayController, OverlayRequest, ToastStore};

#[derive(Debug, Clone, Copy)]
pub struct Toaster {
    position: ToastPosition,
    margin: Option<Px>,
    gap: Option<Px>,
    toast_min_width: Option<Px>,
    toast_max_width: Option<Px>,
}

impl Default for Toaster {
    fn default() -> Self {
        Self {
            position: ToastPosition::BottomRight,
            margin: None,
            gap: None,
            toast_min_width: None,
            toast_max_width: None,
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let id = cx.root_id();
            let store = OverlayController::toast_store(&mut *cx.app);
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

    /// Dispatches a toast request.
    ///
    /// Note: this is an upsert. If `request.id` is set and still refers to an open toast, the
    /// existing toast is updated (useful for `Loading -> Success/Error` flows).
    pub fn toast(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        request: ToastRequest,
    ) -> ToastId {
        OverlayController::toast_action(host, self.store.clone(), window, request)
    }

    /// Updates an existing toast by id (or creates a new one if the id is no longer valid).
    pub fn toast_update(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        id: ToastId,
        request: ToastRequest,
    ) -> ToastId {
        self.toast(host, window, request.id(id))
    }

    pub fn toast_loading(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        title: impl Into<std::sync::Arc<str>>,
    ) -> ToastId {
        self.toast(
            host,
            window,
            ToastRequest::new(title)
                .variant(ToastVariant::Loading)
                .duration(None),
        )
    }

    pub fn toast_loading_update(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        id: ToastId,
        title: impl Into<std::sync::Arc<str>>,
    ) -> ToastId {
        self.toast_update(
            host,
            window,
            id,
            ToastRequest::new(title)
                .variant(ToastVariant::Loading)
                .duration(None),
        )
    }

    pub fn toast_success(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        title: impl Into<std::sync::Arc<str>>,
    ) -> ToastId {
        self.toast(
            host,
            window,
            ToastRequest::new(title).variant(ToastVariant::Success),
        )
    }

    pub fn toast_success_update(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        id: ToastId,
        title: impl Into<std::sync::Arc<str>>,
    ) -> ToastId {
        self.toast_update(
            host,
            window,
            id,
            ToastRequest::new(title).variant(ToastVariant::Success),
        )
    }

    pub fn toast_error(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        title: impl Into<std::sync::Arc<str>>,
    ) -> ToastId {
        self.toast(
            host,
            window,
            ToastRequest::new(title).variant(ToastVariant::Error),
        )
    }

    pub fn toast_error_update(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        id: ToastId,
        title: impl Into<std::sync::Arc<str>>,
    ) -> ToastId {
        self.toast_update(
            host,
            window,
            id,
            ToastRequest::new(title).variant(ToastVariant::Error),
        )
    }

    pub fn toast_info(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        title: impl Into<std::sync::Arc<str>>,
    ) -> ToastId {
        self.toast(
            host,
            window,
            ToastRequest::new(title).variant(ToastVariant::Info),
        )
    }

    pub fn toast_info_update(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        id: ToastId,
        title: impl Into<std::sync::Arc<str>>,
    ) -> ToastId {
        self.toast_update(
            host,
            window,
            id,
            ToastRequest::new(title).variant(ToastVariant::Info),
        )
    }

    pub fn toast_warning(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        title: impl Into<std::sync::Arc<str>>,
    ) -> ToastId {
        self.toast(
            host,
            window,
            ToastRequest::new(title).variant(ToastVariant::Warning),
        )
    }

    pub fn toast_warning_update(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        id: ToastId,
        title: impl Into<std::sync::Arc<str>>,
    ) -> ToastId {
        self.toast_update(
            host,
            window,
            id,
            ToastRequest::new(title).variant(ToastVariant::Warning),
        )
    }

    pub fn dismiss(&self, host: &mut dyn UiActionHost, window: AppWindowId, id: ToastId) -> bool {
        OverlayController::dismiss_toast_action(host, self.store.clone(), window, id)
    }
}

pub use fret_ui_kit::{ToastAction, ToastId, ToastPosition, ToastRequest, ToastVariant};
