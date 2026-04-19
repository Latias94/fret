pub const DOCS_SOURCE: &str = include_str!("usage.docs.rs.txt");
#[allow(dead_code)]
pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use super::{last_action_model, message_request, preview_frame};
use fret::app::AppRenderActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let sonner = shadcn::Sonner::global(&mut *cx.app);
    let last_action = last_action_model(cx);

    let button = shadcn::Button::new("Show Toast")
        .variant(shadcn::ButtonVariant::Outline)
        .on_activate(cx.actions().listen(move |host, action_cx| {
            sonner.toast(
                host,
                action_cx.window,
                message_request(
                    "Event has been created.",
                    shadcn::ToastVariant::Default,
                    shadcn::ToastMessageOptions::new(),
                ),
            );
            let _ = host.models_mut().update(&last_action, |value| {
                *value = Arc::<str>::from("sonner.usage.show");
            });
            host.request_redraw(action_cx.window);
        }))
        .test_id("ui-gallery-sonner-usage")
        .into_element(cx);

    preview_frame::<fret_app::App, _>(button).into_element(cx)
}
// endregion: example
