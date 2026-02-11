use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use raw_window_handle::HasWindowHandle;
use tracing::warn;
use wry::PageLoadEvent;

use fret_core::AppWindowId;
use fret_webview::{
    WebViewEvent, WebViewId, WebViewNavigationState, WebViewPlacement, WebViewRequest,
};

use crate::wry_backend::{WryWebView, build_webview_as_child};

#[derive(Debug)]
pub struct WryWebViewHost {
    instances: HashMap<WebViewId, HostedWryWebView>,
    enable_devtools: bool,
    events: Arc<Mutex<HashMap<AppWindowId, VecDeque<WebViewEvent>>>>,
}

#[derive(Debug)]
struct HostedWryWebView {
    window: AppWindowId,
    webview: WryWebView,
    last_placement: Option<WebViewPlacement>,
    state: Arc<Mutex<HostedState>>,
}

#[derive(Debug, Clone)]
enum PendingNav {
    LoadUrl,
    Back,
    Forward,
    Reload,
}

#[derive(Debug, Default)]
struct HostedState {
    history: Vec<Arc<str>>,
    index: usize,
    is_loading: bool,
    pending: Option<PendingNav>,
}

impl HostedState {
    fn navigation_state(&self) -> WebViewNavigationState {
        WebViewNavigationState {
            can_go_back: self.index > 0,
            can_go_forward: self.index + 1 < self.history.len(),
            is_loading: self.is_loading,
        }
    }

    fn truncate_forward(&mut self) {
        let keep = self.index.saturating_add(1).min(self.history.len());
        self.history.truncate(keep);
    }

    fn ensure_current(&mut self, url: Arc<str>) {
        if self.history.is_empty() {
            self.history.push(url);
            self.index = 0;
            return;
        }
        if self.index >= self.history.len() {
            self.index = self.history.len() - 1;
        }
        self.history[self.index] = url;
    }

    fn push_new_url(&mut self, url: Arc<str>) {
        if self
            .history
            .get(self.index)
            .is_some_and(|cur| cur.as_ref() == url.as_ref())
        {
            return;
        }
        self.truncate_forward();
        self.history.push(url);
        self.index = self.history.len().saturating_sub(1);
    }
}

impl WryWebViewHost {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            enable_devtools: false,
            events: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn enable_devtools(mut self, enable: bool) -> Self {
        self.enable_devtools = enable;
        self
    }

    pub fn has_instance(&self, id: WebViewId) -> bool {
        self.instances.contains_key(&id)
    }

    pub fn handle_requests_for_window(
        &mut self,
        window_id: AppWindowId,
        window: &(impl HasWindowHandle + ?Sized),
        requests: impl IntoIterator<Item = WebViewRequest>,
    ) -> Vec<WebViewRequest> {
        let mut unhandled = Vec::new();

        for request in requests {
            match request {
                WebViewRequest::Create {
                    id,
                    window: target_window,
                    initial_url,
                } => {
                    if self.instances.contains_key(&id) {
                        continue;
                    }
                    if target_window != window_id {
                        unhandled.push(WebViewRequest::Create {
                            id,
                            window: target_window,
                            initial_url,
                        });
                        continue;
                    }

                    let events = self.events.clone();
                    let hosted_state: Arc<Mutex<HostedState>> =
                        Arc::new(Mutex::new(HostedState::default()));
                    let hosted_state_for_on_load = hosted_state.clone();

                    match build_webview_as_child(
                        window,
                        Some(initial_url.clone()),
                        self.enable_devtools,
                        move |builder| {
                            let id_for_title = id;
                            let target_window_for_title = target_window;
                            let events_for_title = events.clone();
                            let builder =
                                builder.with_document_title_changed_handler(move |title| {
                                    if let Ok(mut map) = events_for_title.lock() {
                                        map.entry(target_window_for_title).or_default().push_back(
                                            WebViewEvent::TitleChanged {
                                                id: id_for_title,
                                                title: Arc::<str>::from(title),
                                            },
                                        );
                                    }
                                });

                            let id_for_load = id;
                            let target_window_for_load = target_window;
                            let events_for_load = events.clone();
                            builder.with_on_page_load_handler(move |event, url| {
                                let url: Arc<str> = Arc::<str>::from(url);

                                let mut st = match hosted_state_for_on_load.lock() {
                                    Ok(st) => st,
                                    Err(poisoned) => poisoned.into_inner(),
                                };

                                match event {
                                    PageLoadEvent::Started => {
                                        let pending = st.pending.take();
                                        match pending {
                                            Some(PendingNav::Back) => {
                                                if st.index > 0 {
                                                    st.index -= 1;
                                                }
                                                st.ensure_current(url.clone());
                                            }
                                            Some(PendingNav::Forward) => {
                                                if st.index + 1 < st.history.len() {
                                                    st.index += 1;
                                                }
                                                st.ensure_current(url.clone());
                                            }
                                            Some(PendingNav::LoadUrl) | None => {
                                                st.push_new_url(url.clone());
                                            }
                                            Some(PendingNav::Reload) => {
                                                st.ensure_current(url.clone());
                                            }
                                        }
                                        st.is_loading = true;
                                    }
                                    PageLoadEvent::Finished => {
                                        st.ensure_current(url.clone());
                                        st.is_loading = false;
                                    }
                                }

                                let nav = st.navigation_state();
                                drop(st);

                                if let Ok(mut map) = events_for_load.lock() {
                                    let q = map.entry(target_window_for_load).or_default();
                                    q.push_back(WebViewEvent::UrlChanged {
                                        id: id_for_load,
                                        url: url.clone(),
                                    });
                                    q.push_back(WebViewEvent::NavigationStateChanged {
                                        id: id_for_load,
                                        state: nav,
                                    });
                                }
                            })
                        },
                    ) {
                        Ok(inner) => {
                            self.instances.insert(
                                id,
                                HostedWryWebView {
                                    window: target_window,
                                    webview: WryWebView::new(inner),
                                    last_placement: None,
                                    state: hosted_state,
                                },
                            );
                            if let Ok(mut map) = self.events.lock() {
                                map.entry(target_window)
                                    .or_default()
                                    .push_back(WebViewEvent::Created { id });
                            }
                        }
                        Err(err) => {
                            warn!(
                                ?id,
                                ?target_window,
                                ?err,
                                "wry webview build_as_child failed"
                            );
                            // Keep the create request so the runner can retry on the next frame.
                            unhandled.push(WebViewRequest::Create {
                                id,
                                window: target_window,
                                initial_url,
                            });
                        }
                    }
                }
                WebViewRequest::Destroy { id } => {
                    if let Some(inst) = self.instances.remove(&id) {
                        if let Ok(mut map) = self.events.lock() {
                            map.entry(inst.window)
                                .or_default()
                                .push_back(WebViewEvent::Destroyed { id });
                        }
                    }
                }
                WebViewRequest::SetPlacement { id, placement } => {
                    let Some(instance) = self.instances.get_mut(&id) else {
                        // Placement is recomputed continuously by the runner; drop if the instance
                        // is not ready yet.
                        continue;
                    };

                    if instance.last_placement == Some(placement) {
                        continue;
                    }

                    if let Err(err) = instance.webview.set_visible(placement.visible) {
                        warn!(?id, ?err, "wry webview set_visible failed");
                    }
                    if let Err(err) = instance.webview.set_bounds_logical(placement.bounds) {
                        warn!(?id, ?err, "wry webview set_bounds failed");
                    }
                    instance.last_placement = Some(placement);
                }
                WebViewRequest::LoadUrl { id, url } => {
                    let Some(instance) = self.instances.get(&id) else {
                        unhandled.push(WebViewRequest::LoadUrl { id, url });
                        continue;
                    };
                    // TODO: Consider tracking current URL and skipping no-op loads.
                    if let Ok(mut st) = instance.state.lock() {
                        st.pending = Some(PendingNav::LoadUrl);
                    }
                    if let Err(err) = instance.webview.load_url(url.as_ref()) {
                        warn!(?id, ?err, "wry webview load_url failed");
                        if let Ok(mut map) = self.events.lock() {
                            map.entry(window_id)
                                .or_default()
                                .push_back(WebViewEvent::LoadFailed {
                                    id,
                                    error: Arc::<str>::from(err.to_string()),
                                });
                        }
                    }
                }
                WebViewRequest::GoBack { id } => {
                    let Some(instance) = self.instances.get(&id) else {
                        unhandled.push(WebViewRequest::GoBack { id });
                        continue;
                    };
                    if let Ok(mut st) = instance.state.lock() {
                        st.pending = Some(PendingNav::Back);
                    }
                    if let Err(err) = instance.webview.go_back() {
                        warn!(?id, ?err, "wry webview go_back failed");
                    }
                }
                WebViewRequest::GoForward { id } => {
                    let Some(instance) = self.instances.get(&id) else {
                        unhandled.push(WebViewRequest::GoForward { id });
                        continue;
                    };
                    if let Ok(mut st) = instance.state.lock() {
                        st.pending = Some(PendingNav::Forward);
                    }
                    if let Err(err) = instance.webview.go_forward() {
                        warn!(?id, ?err, "wry webview go_forward failed");
                    }
                }
                WebViewRequest::Reload { id } => {
                    let Some(instance) = self.instances.get(&id) else {
                        unhandled.push(WebViewRequest::Reload { id });
                        continue;
                    };
                    if let Ok(mut st) = instance.state.lock() {
                        st.pending = Some(PendingNav::Reload);
                    }
                    if let Err(err) = instance.webview.reload() {
                        warn!(?id, ?err, "wry webview reload failed");
                    }
                }
            }
        }

        unhandled
    }

    pub fn drain_events_for_window(&mut self, window: AppWindowId) -> Vec<WebViewEvent> {
        let mut map = match self.events.lock() {
            Ok(map) => map,
            Err(poisoned) => poisoned.into_inner(),
        };
        let Some(mut q) = map.remove(&window) else {
            return Vec::new();
        };
        q.drain(..).collect()
    }

    pub fn destroy_all_for_window(&mut self, window: AppWindowId) -> Vec<WebViewEvent> {
        let ids = self
            .instances
            .iter()
            .filter_map(|(id, inst)| (inst.window == window).then_some(*id))
            .collect::<Vec<_>>();

        if ids.is_empty() {
            let _ = self.events.lock().map(|mut map| map.remove(&window));
            return Vec::new();
        }

        for id in &ids {
            self.instances.remove(id);
        }

        let mut out = ids
            .into_iter()
            .map(|id| WebViewEvent::Destroyed { id })
            .collect::<Vec<_>>();

        if let Ok(mut map) = self.events.lock() {
            map.remove(&window);
        }

        // Keep ordering stable for potential UI consumers.
        out.sort_by_key(|ev| match ev {
            WebViewEvent::Destroyed { id } => id.0,
            _ => 0,
        });

        out
    }

    pub fn window_for(&self, id: WebViewId) -> Option<AppWindowId> {
        self.instances.get(&id).map(|inst| inst.window)
    }
}
