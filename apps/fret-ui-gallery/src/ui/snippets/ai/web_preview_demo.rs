pub const SOURCE: &str = include_str!("web_preview_demo.rs");

// region: example
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let history = cx.local_model_keyed("history", Vec::<String>::new);
    let history_ix = cx.local_model_keyed("history_ix", || 0usize);
    let committed = cx.local_model_keyed("committed", || false);
    let loading = cx.local_model_keyed("loading", || false);

    let history_now = cx
        .get_model_cloned(&history, Invalidation::Layout)
        .unwrap_or_default();
    let ix_now = cx
        .get_model_cloned(&history_ix, Invalidation::Layout)
        .unwrap_or(0);
    let can_back = ix_now > 0 && !history_now.is_empty();
    let can_forward = !history_now.is_empty() && (ix_now + 1) < history_now.len();
    let loading_now = cx
        .get_model_copied(&loading, Invalidation::Layout)
        .unwrap_or(false);
    let committed_now = cx
        .get_model_copied(&committed, Invalidation::Layout)
        .unwrap_or(false);

    let markers = {
        let mut out: Vec<AnyElement> = Vec::new();
        if !loading_now {
            out.push(cx.text("").test_id("ui-ai-web-preview-demo-loading-false"));
        }
        if committed_now {
            out.push(cx.text("").test_id("ui-ai-web-preview-demo-committed-true"));
        }
        if can_back {
            out.push(cx.text("").test_id("ui-ai-web-preview-demo-can-back-true"));
        }
        if can_forward {
            out.push(
                cx.text("")
                    .test_id("ui-ai-web-preview-demo-can-forward-true"),
            );
        } else {
            out.push(
                cx.text("")
                    .test_id("ui-ai-web-preview-demo-can-forward-false"),
            );
        }
        out
    };

    let on_url_change: ui_ai::OnWebPreviewUrlChange = Arc::new({
        let history = history.clone();
        let history_ix = history_ix.clone();
        let committed = committed.clone();
        let loading = loading.clone();
        move |host, action_cx, next| {
            let _ = host.models_mut().update(&committed, |v| *v = true);
            let _ = host.models_mut().update(&loading, |v| *v = false);
            let ix = host.models_mut().get_cloned(&history_ix).unwrap_or(0);
            let _ = host.models_mut().update(&history, |v| {
                if !v.is_empty() && ix + 1 < v.len() {
                    v.truncate(ix + 1);
                }
                v.push(next.to_string());
            });
            let len = host.models_mut().read(&history, |h| h.len()).unwrap_or(0);
            let _ = host
                .models_mut()
                .update(&history_ix, |v| *v = len.saturating_sub(1));
            host.notify(action_cx);
        }
    });

    let view = ui_ai::WebPreview::new()
        .on_url_change(on_url_change)
        .test_id_root("ui-ai-web-preview-demo-root")
        .into_element_with_children(cx, move |cx, controller| {
            let back = ui_ai::WebPreviewNavigationButton::new([cx.text("←")])
                .disabled(!can_back)
                .test_id("ui-ai-web-preview-demo-nav-back")
                .on_activate(Arc::new({
                    let history = history.clone();
                    let history_ix = history_ix.clone();
                    let url = controller.url.clone();
                    move |host, action_cx, _reason| {
                        let ix = host.models_mut().get_cloned(&history_ix).unwrap_or(0);
                        if ix == 0 {
                            return;
                        }
                        let next_ix = ix - 1;
                        let next_url = host
                            .models_mut()
                            .read(&history, |h| h.get(next_ix).cloned())
                            .ok()
                            .flatten()
                            .unwrap_or_default();
                        let _ = host.models_mut().update(&history_ix, |v| *v = next_ix);
                        let _ = host.models_mut().update(&url, |v| *v = next_url);
                        host.notify(action_cx);
                    }
                }))
                .into_element(cx);

            let forward = ui_ai::WebPreviewNavigationButton::new([cx.text("→")])
                .disabled(!can_forward)
                .test_id("ui-ai-web-preview-demo-nav-forward")
                .on_activate(Arc::new({
                    let history = history.clone();
                    let history_ix = history_ix.clone();
                    let url = controller.url.clone();
                    move |host, action_cx, _reason| {
                        let ix = host.models_mut().get_cloned(&history_ix).unwrap_or(0);
                        let len = host.models_mut().read(&history, |h| h.len()).unwrap_or(0);
                        if len == 0 || ix + 1 >= len {
                            return;
                        }
                        let next_ix = ix + 1;
                        let next_url = host
                            .models_mut()
                            .read(&history, |h| h.get(next_ix).cloned())
                            .ok()
                            .flatten()
                            .unwrap_or_default();
                        let _ = host.models_mut().update(&history_ix, |v| *v = next_ix);
                        let _ = host.models_mut().update(&url, |v| *v = next_url);
                        host.notify(action_cx);
                    }
                }))
                .into_element(cx);

            let url = ui_ai::WebPreviewUrl::new()
                .test_id("ui-ai-web-preview-demo-url")
                .into_element(cx);

            let nav = ui_ai::WebPreviewNavigation::new([back, forward, url]).into_element(cx);

            let console = ui_ai::WebPreviewConsole::new()
                .logs(Arc::from([
                    ui_ai::WebPreviewConsoleLog::new(
                        ui_ai::WebPreviewConsoleLogLevel::Log,
                        "Console output (demo)",
                    ),
                    ui_ai::WebPreviewConsoleLog::new(
                        ui_ai::WebPreviewConsoleLogLevel::Warn,
                        "Warning output (demo)",
                    ),
                    ui_ai::WebPreviewConsoleLog::new(
                        ui_ai::WebPreviewConsoleLogLevel::Error,
                        "Error output (demo)",
                    ),
                ]))
                .test_id_trigger("ui-ai-web-preview-demo-console-trigger")
                .test_id_marker("ui-ai-web-preview-demo-console-content-marker")
                .into_element(cx);

            let body = ui_ai::WebPreviewBody::new().into_element(cx);

            let mut out: Vec<AnyElement> = vec![nav, body, console];
            out.extend(markers);
            out
        });

    ui::v_flex(move |cx| {
        vec![
            cx.text("Web Preview (AI Elements)"),
            cx.text("URL commit + simple history markers for diag scripts."),
            view,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
