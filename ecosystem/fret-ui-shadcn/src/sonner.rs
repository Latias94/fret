use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::Duration;
use std::{fmt, future::Future};

use fret_core::AppWindowId;
use fret_core::Px;
use fret_executor::FutureSpawnerHandle;
use fret_runtime::{CommandId, DispatcherHandle, Model};
use fret_ui::action::UiActionHost;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{OverlayController, OverlayRequest, ToastAsyncQueueHandle, ToastStore};

#[derive(Debug, Clone)]
pub struct Toaster {
    id: Option<Arc<str>>,
    position: ToastPosition,
    margin: Option<Px>,
    gap: Option<Px>,
    toast_min_width: Option<Px>,
    toast_max_width: Option<Px>,
    max_toasts: Option<usize>,
    visible_toasts: Option<usize>,
    expand_by_default: bool,
    rich_colors: bool,
    invert: bool,
}

#[derive(Debug, Default)]
struct ToasterConfigState {
    max_toasts: Option<usize>,
}

impl Default for Toaster {
    fn default() -> Self {
        Self {
            id: None,
            // shadcn/ui v4 app layout mounts Sonner's `<Toaster position="top-center" />`.
            // Keep the default aligned with that "golden" baseline.
            position: ToastPosition::TopCenter,
            // Sonner defaults (sonner@2.x):
            // - offset: 24px (non-mobile)
            // - gap: 14px
            // - width: 356px
            margin: Some(Px(24.0)),
            gap: Some(Px(14.0)),
            toast_min_width: Some(Px(356.0)),
            toast_max_width: Some(Px(356.0)),
            max_toasts: Some(fret_ui_kit::DEFAULT_MAX_TOASTS),
            visible_toasts: Some(fret_ui_kit::DEFAULT_VISIBLE_TOASTS),
            expand_by_default: false,
            rich_colors: false,
            invert: false,
        }
    }
}

impl Toaster {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.id = Some(id.into());
        self
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

    pub fn visible_toasts(mut self, visible_toasts: usize) -> Self {
        self.visible_toasts = Some(visible_toasts.max(1));
        self
    }

    pub fn expand_by_default(mut self, expand: bool) -> Self {
        self.expand_by_default = expand;
        self
    }

    pub fn rich_colors(mut self, rich_colors: bool) -> Self {
        self.rich_colors = rich_colors;
        self
    }

    pub fn invert(mut self, invert: bool) -> Self {
        self.invert = invert;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let id = cx.root_id();
            let store = OverlayController::toast_store(&mut *cx.app);
            let config_changed = cx.with_state(ToasterConfigState::default, |st| {
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

            let mut style = fret_ui_kit::ToastLayerStyle::default();
            // Sonner's close button is opt-in (`closeButton` prop) and shadcn/ui does not enable it
            // in the v4 app layout baseline.
            style.show_close_button = false;

            let mut request = OverlayRequest::toast_layer(id, store)
                .toast_position(self.position)
                .toast_style(style)
                .toast_expand_by_default(self.expand_by_default)
                .toast_rich_colors(self.rich_colors)
                .toast_invert(self.invert);
            if let Some(toaster_id) = self.id.clone() {
                request = request.toast_toaster_id(toaster_id);
            }
            if let Some(visible) = self.visible_toasts {
                request = request.toast_visible_toasts(visible);
            }
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
    async_queue: ToastAsyncQueueHandle,
    dispatcher: Option<DispatcherHandle>,
    spawner: Option<FutureSpawnerHandle>,
}

#[derive(Debug, Default, Clone)]
pub struct ToastMessageOptions {
    pub description: Option<Arc<str>>,
    pub action: Option<ToastAction>,
    pub cancel: Option<ToastAction>,
    /// `None` means "use the default toast duration".
    /// `Some(None)` means "pinned" (no auto-close timer).
    pub duration: Option<Option<Duration>>,
    pub dismissible: Option<bool>,
}

impl ToastMessageOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn action(mut self, label: impl Into<Arc<str>>, command: impl Into<CommandId>) -> Self {
        self.action = Some(ToastAction {
            label: label.into(),
            command: command.into(),
        });
        self
    }

    pub fn action_with(mut self, action: ToastAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn cancel(mut self, label: impl Into<Arc<str>>, command: impl Into<CommandId>) -> Self {
        self.cancel = Some(ToastAction {
            label: label.into(),
            command: command.into(),
        });
        self
    }

    pub fn cancel_with(mut self, cancel: ToastAction) -> Self {
        self.cancel = Some(cancel);
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(Some(duration));
        self
    }

    pub fn pinned(mut self) -> Self {
        self.duration = Some(None);
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = Some(dismissible);
        self
    }
}

impl std::fmt::Debug for Sonner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sonner")
            .field("store", &"<model>")
            .field("async_queue", &"<queue>")
            .field(
                "dispatcher",
                &self.dispatcher.as_ref().map(|_| "<dispatcher>"),
            )
            .field("spawner", &self.spawner.as_ref().map(|_| "<spawner>"))
            .finish()
    }
}

impl Sonner {
    pub fn global<H: UiHost>(app: &mut H) -> Self {
        Self {
            store: OverlayController::toast_store(app),
            async_queue: fret_ui_kit::toast_async_queue(app),
            dispatcher: app.global::<DispatcherHandle>().cloned(),
            spawner: app.global::<FutureSpawnerHandle>().cloned(),
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

    /// A convenience helper matching the "message-style" `sonner` surface:
    /// `toast("title", { description, action, cancel, ... })`.
    pub fn toast_message(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        title: impl Into<Arc<str>>,
        options: ToastMessageOptions,
    ) -> ToastId {
        self.toast(
            host,
            window,
            apply_toast_message_options(
                base_message_request(title.into(), ToastVariant::Default),
                options,
            ),
        )
    }

    pub fn toast_message_update(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        id: ToastId,
        title: impl Into<Arc<str>>,
        options: ToastMessageOptions,
    ) -> ToastId {
        self.toast_update(
            host,
            window,
            id,
            apply_toast_message_options(
                base_message_request(title.into(), ToastVariant::Default),
                options,
            ),
        )
    }

    pub fn toast_success_message(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        title: impl Into<Arc<str>>,
        options: ToastMessageOptions,
    ) -> ToastId {
        self.toast(
            host,
            window,
            apply_toast_message_options(
                base_message_request(title.into(), ToastVariant::Success),
                options,
            ),
        )
    }

    pub fn toast_error_message(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        title: impl Into<Arc<str>>,
        options: ToastMessageOptions,
    ) -> ToastId {
        self.toast(
            host,
            window,
            apply_toast_message_options(
                base_message_request(title.into(), ToastVariant::Error),
                options,
            ),
        )
    }

    pub fn toast_info_message(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        title: impl Into<Arc<str>>,
        options: ToastMessageOptions,
    ) -> ToastId {
        self.toast(
            host,
            window,
            apply_toast_message_options(
                base_message_request(title.into(), ToastVariant::Info),
                options,
            ),
        )
    }

    pub fn toast_warning_message(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        title: impl Into<Arc<str>>,
        options: ToastMessageOptions,
    ) -> ToastId {
        self.toast(
            host,
            window,
            apply_toast_message_options(
                base_message_request(title.into(), ToastVariant::Warning),
                options,
            ),
        )
    }

    pub fn toast_loading_message(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        title: impl Into<Arc<str>>,
        options: ToastMessageOptions,
    ) -> ToastId {
        self.toast(
            host,
            window,
            apply_toast_message_options(
                base_message_request(title.into(), ToastVariant::Loading),
                options,
            ),
        )
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

    /// Starts a manual "promise" toast flow, similar to `sonner`'s `toast.promise(...)` on the web.
    ///
    /// This does not run async tasks. It returns a handle that can be resolved later by updating
    /// the same toast id (e.g. `Loading -> Success/Error`).
    pub fn toast_promise(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        loading: impl Into<std::sync::Arc<str>>,
    ) -> ToastPromise {
        let id = self.toast_loading(host, window, loading);
        ToastPromise {
            sonner: self.clone(),
            window,
            id,
        }
    }

    pub fn toast_promise_with(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        loading: ToastRequest,
    ) -> ToastPromise {
        let id = self.toast(
            host,
            window,
            loading.variant(ToastVariant::Loading).duration(None),
        );
        ToastPromise {
            sonner: self.clone(),
            window,
            id,
        }
    }

    /// Runs an async task and updates the toast when it resolves/rejects, mirroring Sonner's
    /// `toast.promise(...)` surface on the web.
    ///
    /// Notes:
    /// - This requires installing a `FutureSpawnerHandle` as a global.
    /// - Completion is applied via a queue drained during the window overlays render pass.
    pub fn toast_promise_async<T, E, Fut>(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        loading: impl Into<Arc<str>>,
        promise: impl FnOnce() -> Fut + Send + 'static,
        options: ToastPromiseAsyncOptions<T, E>,
    ) -> ToastId
    where
        T: Send + 'static,
        E: Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        self.toast_promise_async_with(
            host,
            window,
            ToastRequest::new(loading)
                .variant(ToastVariant::Loading)
                .duration(None),
            promise,
            options,
        )
    }

    pub fn toast_promise_async_with<T, E, Fut>(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        loading: ToastRequest,
        promise: impl FnOnce() -> Fut + Send + 'static,
        options: ToastPromiseAsyncOptions<T, E>,
    ) -> ToastId
    where
        T: Send + 'static,
        E: Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        self.toast_promise_async_handle_with(host, window, Some(loading), promise, options)
            .id()
            .unwrap_or(ToastId(0))
    }

    /// Runs an async task and returns a handle that can be awaited via `.unwrap()`.
    ///
    /// This is the closest Rust analogue to Sonner's `toast.promise(...).unwrap()` pattern.
    /// The returned handle:
    /// - always provides an awaitable `unwrap()` future (even if no toast is shown),
    /// - optionally provides a `ToastId` when a loading toast is created.
    ///
    /// If `loading` is `None`, no loading toast is shown and the handle's `id()` is `None` (matching
    /// Sonner's behavior where `toast.promise(promise, { success/error })` can return `{ unwrap }`
    /// without an id when no `loading` toast exists).
    pub fn toast_promise_async_handle_with<T, E, Fut>(
        &self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        loading: Option<ToastRequest>,
        promise: impl FnOnce() -> Fut + Send + 'static,
        options: ToastPromiseAsyncOptions<T, E>,
    ) -> ToastPromiseHandle<T, E>
    where
        T: Send + 'static,
        E: Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        let loading_description = options.description_static.clone();
        let id = loading.map(|loading| {
            let mut req = loading.variant(ToastVariant::Loading).duration(None);
            if let Some(desc) = loading_description {
                req = req.description(desc);
            }
            self.toast(host, window, req)
        });

        let shared = Arc::new(ToastPromiseShared::<T, E>::default());
        let handle = ToastPromiseHandle {
            id,
            shared: shared.clone(),
        };

        let Some(spawner) = self.spawner.clone() else {
            shared.complete(Err(
                ToastPromiseUnwrapError::MissingFutureSpawnerHandleGlobal,
            ));
            if let Some(id) = id {
                let _ = self.toast_update(
                    host,
                    window,
                    id,
                    ToastRequest::new("missing FutureSpawnerHandle global")
                        .variant(ToastVariant::Error),
                );
            }
            return handle;
        };
        let Some(dispatcher) = self.dispatcher.clone() else {
            shared.complete(Err(ToastPromiseUnwrapError::MissingDispatcherHandleGlobal));
            if let Some(id) = id {
                let _ = self.toast_update(
                    host,
                    window,
                    id,
                    ToastRequest::new("missing DispatcherHandle global")
                        .variant(ToastVariant::Error),
                );
            }
            return handle;
        };

        let queue = self.async_queue.clone();
        let success = options.success.clone();
        let error = options.error.clone();
        let finally = options.finally.clone();
        let description_static = options.description_static.clone();
        let description_success = options.description_success.clone();
        let description_error = options.description_error.clone();

        spawner.spawn_send(Box::pin(async move {
            let fut = promise();
            let result = fut.await;

            match result {
                Ok(value) => {
                    let desc = description_success
                        .as_ref()
                        .map(|f| (f)(&value))
                        .or_else(|| description_static.clone());

                    if let Some(success) = success {
                        let mut req = (success)(&value);
                        if req.variant == ToastVariant::Default {
                            req = req.variant(ToastVariant::Success);
                        }
                        if let Some(desc) = desc {
                            req = req.description(desc);
                        }
                        if let Some(id) = id {
                            queue.upsert(window, req.id(id));
                        } else {
                            queue.upsert(window, req);
                        }
                    } else if let Some(id) = id {
                        queue.dismiss(window, id);
                    }

                    shared.complete(Ok(value));
                }
                Err(err) => {
                    let desc = description_error
                        .as_ref()
                        .map(|f| (f)(&err))
                        .or_else(|| description_static.clone());

                    if let Some(error) = error {
                        let mut req = (error)(&err);
                        if req.variant == ToastVariant::Default {
                            req = req.variant(ToastVariant::Error);
                        }
                        if let Some(desc) = desc {
                            req = req.description(desc);
                        }
                        if let Some(id) = id {
                            queue.upsert(window, req.id(id));
                        } else {
                            queue.upsert(window, req);
                        }
                    } else if let Some(id) = id {
                        queue.dismiss(window, id);
                    }

                    shared.complete(Err(ToastPromiseUnwrapError::Rejected(err)));
                }
            }

            if let Some(finally) = finally {
                finally();
            }

            dispatcher.wake(Some(window));
        }));

        handle
    }
}

#[derive(Debug, Clone)]
pub struct ToastPromise {
    sonner: Sonner,
    window: AppWindowId,
    id: ToastId,
}

impl ToastPromise {
    pub fn id(&self) -> ToastId {
        self.id
    }

    pub fn update(&self, host: &mut dyn UiActionHost, request: ToastRequest) -> ToastId {
        self.sonner
            .toast_update(host, self.window, self.id, request)
    }

    pub fn success(
        &self,
        host: &mut dyn UiActionHost,
        title: impl Into<std::sync::Arc<str>>,
    ) -> ToastId {
        self.sonner
            .toast_success_update(host, self.window, self.id, title)
    }

    pub fn success_with(
        &self,
        host: &mut dyn UiActionHost,
        title: impl Into<Arc<str>>,
        options: ToastMessageOptions,
    ) -> ToastId {
        self.update(
            host,
            apply_toast_message_options(
                base_message_request(title.into(), ToastVariant::Success),
                options,
            ),
        )
    }

    pub fn error(
        &self,
        host: &mut dyn UiActionHost,
        title: impl Into<std::sync::Arc<str>>,
    ) -> ToastId {
        self.sonner
            .toast_error_update(host, self.window, self.id, title)
    }

    pub fn error_with(
        &self,
        host: &mut dyn UiActionHost,
        title: impl Into<Arc<str>>,
        options: ToastMessageOptions,
    ) -> ToastId {
        self.update(
            host,
            apply_toast_message_options(
                base_message_request(title.into(), ToastVariant::Error),
                options,
            ),
        )
    }

    pub fn dismiss(&self, host: &mut dyn UiActionHost) -> bool {
        self.sonner.dismiss(host, self.window, self.id)
    }
}

pub use fret_ui_kit::{ToastAction, ToastId, ToastPosition, ToastRequest, ToastVariant};

#[derive(Clone)]
pub struct ToastPromiseAsyncOptions<T, E> {
    success: Option<Arc<dyn Fn(&T) -> ToastRequest + Send + Sync + 'static>>,
    error: Option<Arc<dyn Fn(&E) -> ToastRequest + Send + Sync + 'static>>,
    finally: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    description_static: Option<Arc<str>>,
    description_success: Option<Arc<dyn Fn(&T) -> Arc<str> + Send + Sync + 'static>>,
    description_error: Option<Arc<dyn Fn(&E) -> Arc<str> + Send + Sync + 'static>>,
}

impl<T, E> fmt::Debug for ToastPromiseAsyncOptions<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToastPromiseAsyncOptions")
            .field("success", &self.success.as_ref().map(|_| "<fn>"))
            .field("error", &self.error.as_ref().map(|_| "<fn>"))
            .field("finally", &self.finally.as_ref().map(|_| "<fn>"))
            .field(
                "description_static",
                &self.description_static.as_ref().map(|_| "<str>"),
            )
            .field(
                "description_success",
                &self.description_success.as_ref().map(|_| "<fn>"),
            )
            .field(
                "description_error",
                &self.description_error.as_ref().map(|_| "<fn>"),
            )
            .finish()
    }
}

impl<T, E> Default for ToastPromiseAsyncOptions<T, E> {
    fn default() -> Self {
        Self {
            success: None,
            error: None,
            finally: None,
            description_static: None,
            description_success: None,
            description_error: None,
        }
    }
}

impl<T, E> ToastPromiseAsyncOptions<T, E> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn success_message(mut self, title: impl Into<Arc<str>>) -> Self {
        let title = title.into();
        self.success = Some(Arc::new(move |_value| {
            ToastRequest::new(title.clone()).variant(ToastVariant::Success)
        }));
        self
    }

    pub fn error_message(mut self, title: impl Into<Arc<str>>) -> Self {
        let title = title.into();
        self.error = Some(Arc::new(move |_err| {
            ToastRequest::new(title.clone()).variant(ToastVariant::Error)
        }));
        self
    }

    pub fn success_with(mut self, f: impl Fn(&T) -> ToastRequest + Send + Sync + 'static) -> Self {
        self.success = Some(Arc::new(f));
        self
    }

    pub fn error_with(mut self, f: impl Fn(&E) -> ToastRequest + Send + Sync + 'static) -> Self {
        self.error = Some(Arc::new(f));
        self
    }

    pub fn finally(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.finally = Some(Arc::new(f));
        self
    }

    /// Sets a static description, mirroring Sonner's `description: "..."` option.
    ///
    /// This description is applied to:
    /// - the loading toast (if present),
    /// - the success/error toast updates (when a success/error handler is configured).
    pub fn description_message(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description_static = Some(description.into());
        self
    }

    /// Sets a description computed from the resolved promise value.
    pub fn description_success_with(
        mut self,
        f: impl Fn(&T) -> Arc<str> + Send + Sync + 'static,
    ) -> Self {
        self.description_success = Some(Arc::new(f));
        self
    }

    /// Sets a description computed from the rejected promise value.
    pub fn description_error_with(
        mut self,
        f: impl Fn(&E) -> Arc<str> + Send + Sync + 'static,
    ) -> Self {
        self.description_error = Some(Arc::new(f));
        self
    }
}

#[derive(Debug, Clone)]
pub enum ToastPromiseUnwrapError<E> {
    MissingFutureSpawnerHandleGlobal,
    MissingDispatcherHandleGlobal,
    Rejected(E),
}

struct ToastPromiseShared<T, E> {
    state: std::sync::Mutex<ToastPromiseSharedState<T, E>>,
}

struct ToastPromiseSharedState<T, E> {
    result: Option<Result<T, ToastPromiseUnwrapError<E>>>,
    waker: Option<Waker>,
}

impl<T, E> Default for ToastPromiseShared<T, E> {
    fn default() -> Self {
        Self {
            state: std::sync::Mutex::new(ToastPromiseSharedState {
                result: None,
                waker: None,
            }),
        }
    }
}

impl<T, E> ToastPromiseShared<T, E> {
    fn complete(&self, result: Result<T, ToastPromiseUnwrapError<E>>) {
        let waker = {
            let mut guard = self.state.lock().unwrap_or_else(|p| p.into_inner());
            guard.result = Some(result);
            guard.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

#[derive(Clone)]
pub struct ToastPromiseHandle<T, E> {
    id: Option<ToastId>,
    shared: Arc<ToastPromiseShared<T, E>>,
}

impl<T, E> fmt::Debug for ToastPromiseHandle<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToastPromiseHandle")
            .field("id", &self.id)
            .finish()
    }
}

impl<T, E> ToastPromiseHandle<T, E> {
    pub fn id(&self) -> Option<ToastId> {
        self.id
    }

    pub fn unwrap(self) -> ToastPromiseUnwrap<T, E> {
        ToastPromiseUnwrap {
            shared: self.shared,
        }
    }
}

pub struct ToastPromiseUnwrap<T, E> {
    shared: Arc<ToastPromiseShared<T, E>>,
}

impl<T, E> Future for ToastPromiseUnwrap<T, E> {
    type Output = Result<T, ToastPromiseUnwrapError<E>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut guard = self.shared.state.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(result) = guard.result.take() {
            return Poll::Ready(result);
        }

        guard.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

fn base_message_request(title: Arc<str>, variant: ToastVariant) -> ToastRequest {
    let req = ToastRequest::new(title).variant(variant);
    if matches!(variant, ToastVariant::Loading) {
        req.duration(None)
    } else {
        req
    }
}

fn apply_toast_message_options(
    mut req: ToastRequest,
    options: ToastMessageOptions,
) -> ToastRequest {
    if let Some(description) = options.description {
        req = req.description(description);
    }
    if let Some(action) = options.action {
        req = req.action(action);
    }
    if let Some(cancel) = options.cancel {
        req = req.cancel(cancel);
    }
    if let Some(duration) = options.duration {
        req = req.duration(duration);
    }
    if let Some(dismissible) = options.dismissible {
        req = req.dismissible(dismissible);
    }
    req
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::task::{RawWaker, RawWakerVTable};

    use fret_executor::{FutureSpawner, FutureSpawnerHandle};
    use fret_runtime::{
        DispatchPriority, Dispatcher, DispatcherHandle, ExecCapabilities, Runnable,
    };
    use fret_ui::action::UiActionHostAdapter;

    fn noop_waker() -> Waker {
        unsafe fn clone(_: *const ()) -> RawWaker {
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        unsafe fn wake(_: *const ()) {}
        unsafe fn wake_by_ref(_: *const ()) {}
        unsafe fn drop(_: *const ()) {}

        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

        unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
    }

    #[test]
    fn toast_message_options_apply_description_action_cancel_and_duration() {
        let opts = ToastMessageOptions::new()
            .description("desc")
            .action("Undo", "toast.undo")
            .cancel("Cancel", "toast.cancel")
            .pinned()
            .dismissible(false);

        let req = apply_toast_message_options(
            base_message_request(Arc::from("Hello"), ToastVariant::Default),
            opts,
        );

        assert_eq!(req.title.as_ref(), "Hello");
        assert_eq!(req.description.as_ref().map(|d| d.as_ref()), Some("desc"));
        assert_eq!(req.duration, None);
        assert_eq!(req.dismissible, false);
        assert_eq!(req.action.as_ref().map(|a| a.label.as_ref()), Some("Undo"));
        assert_eq!(
            req.cancel.as_ref().map(|a| a.label.as_ref()),
            Some("Cancel")
        );
    }

    #[test]
    fn toast_loading_message_defaults_to_pinned() {
        let req = base_message_request(Arc::from("Loading..."), ToastVariant::Loading);
        assert_eq!(req.duration, None);
    }

    #[test]
    fn toast_promise_handle_unwrap_reports_missing_spawner() {
        let window = AppWindowId::default();
        let mut app = fret_app::App::new();
        let sonner = Sonner::global(&mut app);

        let mut host = UiActionHostAdapter { app: &mut app };
        let handle = sonner.toast_promise_async_handle_with(
            &mut host,
            window,
            None,
            || async { Ok::<u8, u8>(123) },
            ToastPromiseAsyncOptions::new(),
        );

        let mut fut = Box::pin(handle.unwrap());
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        let poll = fut.as_mut().poll(&mut cx);
        assert!(
            matches!(
                poll,
                Poll::Ready(Err(
                    ToastPromiseUnwrapError::MissingFutureSpawnerHandleGlobal
                ))
            ),
            "expected MissingFutureSpawnerHandleGlobal, got {poll:?}"
        );
    }

    #[test]
    fn toast_promise_handle_unwrap_reports_missing_dispatcher() {
        struct NoopSpawner;
        impl FutureSpawner for NoopSpawner {
            fn spawn_send(&self, _fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {}
        }

        let window = AppWindowId::default();
        let mut app = fret_app::App::new();
        let spawner: FutureSpawnerHandle = Arc::new(NoopSpawner);
        app.set_global::<FutureSpawnerHandle>(spawner);

        let sonner = Sonner::global(&mut app);
        let mut host = UiActionHostAdapter { app: &mut app };
        let handle = sonner.toast_promise_async_handle_with(
            &mut host,
            window,
            None,
            || async { Ok::<u8, u8>(123) },
            ToastPromiseAsyncOptions::new(),
        );

        let mut fut = Box::pin(handle.unwrap());
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        let poll = fut.as_mut().poll(&mut cx);
        assert!(
            matches!(
                poll,
                Poll::Ready(Err(ToastPromiseUnwrapError::MissingDispatcherHandleGlobal))
            ),
            "expected MissingDispatcherHandleGlobal, got {poll:?}"
        );
    }

    #[test]
    fn toast_promise_handle_unwrap_resolves_ok_when_spawner_and_dispatcher_present() {
        struct InlineSpawner;
        impl FutureSpawner for InlineSpawner {
            fn spawn_send(&self, mut fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
                let waker = noop_waker();
                let mut cx = Context::from_waker(&waker);
                let poll = fut.as_mut().poll(&mut cx);
                assert!(matches!(poll, Poll::Ready(())));
            }
        }

        #[derive(Default)]
        struct NoopDispatcher;
        impl Dispatcher for NoopDispatcher {
            fn dispatch_on_main_thread(&self, _task: Runnable) {}
            fn dispatch_background(&self, _task: Runnable, _priority: DispatchPriority) {}
            fn dispatch_after(&self, _delay: Duration, _task: Runnable) {}
            fn wake(&self, _window: Option<AppWindowId>) {}
            fn exec_capabilities(&self) -> ExecCapabilities {
                ExecCapabilities::default()
            }
        }

        let window = AppWindowId::default();
        let mut app = fret_app::App::new();
        let spawner: FutureSpawnerHandle = Arc::new(InlineSpawner);
        let dispatcher: DispatcherHandle = Arc::new(NoopDispatcher::default());
        app.set_global::<FutureSpawnerHandle>(spawner);
        app.set_global::<DispatcherHandle>(dispatcher);

        let sonner = Sonner::global(&mut app);
        let mut host = UiActionHostAdapter { app: &mut app };
        let handle = sonner.toast_promise_async_handle_with(
            &mut host,
            window,
            None,
            || async { Ok::<u8, u8>(7) },
            ToastPromiseAsyncOptions::new(),
        );

        let mut fut = Box::pin(handle.unwrap());
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        let poll = Future::poll(fut.as_mut(), &mut cx);
        assert!(
            matches!(poll, Poll::Ready(Ok(7))),
            "expected Ok(7), got {poll:?}"
        );
    }
}
