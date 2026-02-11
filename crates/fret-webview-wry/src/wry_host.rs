use std::collections::HashMap;

use raw_window_handle::HasWindowHandle;
use tracing::warn;

use fret_core::AppWindowId;
use fret_webview::{WebViewId, WebViewPlacement, WebViewRequest};

use crate::wry_backend::{WryWebView, build_webview_as_child};

#[derive(Debug, Default)]
pub struct WryWebViewHost {
    instances: HashMap<WebViewId, HostedWryWebView>,
    enable_devtools: bool,
}

#[derive(Debug)]
struct HostedWryWebView {
    window: AppWindowId,
    webview: WryWebView,
    last_placement: Option<WebViewPlacement>,
}

impl WryWebViewHost {
    pub fn new() -> Self {
        Self::default()
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

                    match build_webview_as_child(
                        window,
                        Some(initial_url.clone()),
                        self.enable_devtools,
                    ) {
                        Ok(inner) => {
                            self.instances.insert(
                                id,
                                HostedWryWebView {
                                    window: target_window,
                                    webview: WryWebView::new(inner),
                                    last_placement: None,
                                },
                            );
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
                    self.instances.remove(&id);
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
                    if let Err(err) = instance.webview.load_url(url.as_ref()) {
                        warn!(?id, ?err, "wry webview load_url failed");
                    }
                }
                WebViewRequest::GoBack { id } => {
                    let Some(instance) = self.instances.get(&id) else {
                        unhandled.push(WebViewRequest::GoBack { id });
                        continue;
                    };
                    if let Err(err) = instance.webview.go_back() {
                        warn!(?id, ?err, "wry webview go_back failed");
                    }
                }
                WebViewRequest::GoForward { id } => {
                    let Some(instance) = self.instances.get(&id) else {
                        unhandled.push(WebViewRequest::GoForward { id });
                        continue;
                    };
                    if let Err(err) = instance.webview.go_forward() {
                        warn!(?id, ?err, "wry webview go_forward failed");
                    }
                }
                WebViewRequest::Reload { id } => {
                    let Some(instance) = self.instances.get(&id) else {
                        unhandled.push(WebViewRequest::Reload { id });
                        continue;
                    };
                    if let Err(err) = instance.webview.reload() {
                        warn!(?id, ?err, "wry webview reload failed");
                    }
                }
            }
        }

        unhandled
    }

    pub fn window_for(&self, id: WebViewId) -> Option<AppWindowId> {
        self.instances.get(&id).map(|inst| inst.window)
    }
}
