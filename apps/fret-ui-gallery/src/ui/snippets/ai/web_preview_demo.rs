pub const SOURCE: &str = include_str!("web_preview_demo.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

mod act {
    fret::actions!([
        NavigateBack = "ui-gallery.ai.web_preview.navigate_back.v1",
        NavigateForward = "ui-gallery.ai.web_preview.navigate_forward.v1",
    ]);
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
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
    let url_model = cx.local_model_keyed("url_model", String::new);

    cx.actions().models::<act::NavigateBack>({
        let history = history.clone();
        let history_ix = history_ix.clone();
        let url_model = url_model.clone();
        move |models| {
            let ix = models.get_cloned(&history_ix).unwrap_or(0);
            if ix == 0 {
                return false;
            }
            let next_ix = ix - 1;
            let next_url = models
                .read(&history, |entries| entries.get(next_ix).cloned())
                .ok()
                .flatten()
                .unwrap_or_default();
            let history_ix_updated = models.update(&history_ix, |value| *value = next_ix).is_ok();
            let url_updated = models.update(&url_model, |value| *value = next_url).is_ok();
            history_ix_updated && url_updated
        }
    });

    cx.actions().models::<act::NavigateForward>({
        let history = history.clone();
        let history_ix = history_ix.clone();
        let url_model = url_model.clone();
        move |models| {
            let ix = models.get_cloned(&history_ix).unwrap_or(0);
            let len = models.read(&history, |entries| entries.len()).unwrap_or(0);
            if len == 0 || ix + 1 >= len {
                return false;
            }
            let next_ix = ix + 1;
            let next_url = models
                .read(&history, |entries| entries.get(next_ix).cloned())
                .ok()
                .flatten()
                .unwrap_or_default();
            let history_ix_updated = models.update(&history_ix, |value| *value = next_ix).is_ok();
            let url_updated = models.update(&url_model, |value| *value = next_url).is_ok();
            history_ix_updated && url_updated
        }
    });

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

    let back = ui_ai::WebPreviewNavigationButton::go_back([cx.text("←")])
        .disabled(!can_back)
        .test_id("ui-ai-web-preview-demo-nav-back")
        .action(act::NavigateBack);

    let forward = ui_ai::WebPreviewNavigationButton::go_forward([cx.text("→")])
        .disabled(!can_forward)
        .test_id("ui-ai-web-preview-demo-nav-forward")
        .action(act::NavigateForward);

    let nav = ui_ai::WebPreviewNavigation::default()
        .button(back)
        .button(forward)
        .url(ui_ai::WebPreviewUrl::new().test_id("ui-ai-web-preview-demo-url"));

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
        .test_id_marker("ui-ai-web-preview-demo-console-content-marker");

    ui_ai::WebPreview::new()
        .on_url_change(on_url_change)
        .url_model(url_model.clone())
        .test_id_root("ui-ai-web-preview-demo-root")
        .navigation(nav)
        .body(ui_ai::WebPreviewBody::new())
        .console(console)
        .children(markers)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .h_px(Px(360.0)),
        )
        .into_element(cx)
}
// endregion: example

// region: composable_children
pub fn render_composable_children(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let custom_body = ui::v_flex(move |cx| {
        vec![
            cx.text("Custom body content"),
            cx.text("Use this lane when preview chrome is enough for the current build."),
        ]
    })
    .layout(
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .h_full()
            .min_h_0(),
    )
    .gap(Space::N2)
    .into_element(cx);

    let custom_console_note = ui::v_flex(move |cx| {
        vec![
            cx.text("Custom console footer"),
            cx.text("Backend navigation is app-owned and optional in Fret."),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N1)
    .into_element(cx);

    ui_ai::WebPreview::new()
        .default_url("https://fret.dev/docs")
        .navigation(
            ui_ai::WebPreviewNavigation::default()
                .button(ui_ai::WebPreviewNavigationButton::reload([cx.text("↺")]))
                .url(ui_ai::WebPreviewUrl::new()),
        )
        .body(ui_ai::WebPreviewBody::new().child(custom_body))
        .console(
            ui_ai::WebPreviewConsole::new()
                .logs(Arc::from([ui_ai::WebPreviewConsoleLog::new(
                    ui_ai::WebPreviewConsoleLogLevel::Log,
                    "Console rows can also host caller-owned footer content.",
                )]))
                .children([custom_console_note]),
        )
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .h_px(Px(320.0)),
        )
        .into_element(cx)
}
// endregion: composable_children
