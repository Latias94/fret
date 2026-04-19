pub const DOCS_SOURCE: &str = include_str!("description.docs.rs.txt");
#[allow(dead_code)]
pub const SOURCE: &str = include_str!("description.rs");

// region: example
use super::{last_action_model, message_request, preview_controls_row, preview_frame};
use fret::app::AppRenderActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let sonner = shadcn::Sonner::global(&mut *cx.app);
    let last_action = last_action_model(cx);

    let show = shadcn::Button::new("Show Toast")
        .variant(shadcn::ButtonVariant::Outline)
        .on_activate(cx.actions().listen(move |host, action_cx| {
            sonner.toast(
                host,
                action_cx.window,
                message_request(
                    "Event has been created",
                    shadcn::ToastVariant::Default,
                    shadcn::ToastMessageOptions::new().description("Monday, January 3rd at 6:00pm"),
                ),
            );
            let _ = host.models_mut().update(&last_action, |value| {
                *value = Arc::<str>::from("sonner.description.show");
            });
            host.request_redraw(action_cx.window);
        }))
        .test_id("ui-gallery-sonner-description-show")
        .into_element(cx);

    preview_frame::<fret_app::App, _>(preview_controls_row::<fret_app::App>(Space::N2, vec![show]))
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-description"),
        )
        .test_id("ui-gallery-sonner-description-root")
}
// endregion: example
