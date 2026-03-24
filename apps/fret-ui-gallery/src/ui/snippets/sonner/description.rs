pub const SOURCE: &str = include_str!("description.rs");

// region: example
use super::{last_action_model, message_request};
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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

    ui::h_flex(move |_cx| vec![show])
        .gap(Space::N2)
        .items_center()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-description"),
        )
        .test_id("ui-gallery-sonner-description-root")
}
// endregion: example
