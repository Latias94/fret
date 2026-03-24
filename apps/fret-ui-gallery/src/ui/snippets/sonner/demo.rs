pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use super::{last_action_model, message_request};
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn wrap_controls_row<H: UiHost>(
    gap: Space,
    children: Vec<AnyElement>,
) -> impl IntoUiElement<H> + use<H> {
    ui::h_flex(move |_cx| children)
        .gap(gap)
        .items_center()
        .wrap()
        .layout(LayoutRefinement::default().w_full())
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    const CMD_TOAST_ACTION: &str = "ui_gallery.toast.action";

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
                    shadcn::ToastMessageOptions::new()
                        .description("Sunday, December 03, 2023 at 9:00 AM")
                        .action_id("Undo", CMD_TOAST_ACTION),
                ),
            );
            let _ = host.models_mut().update(&last_action, |value| {
                *value = Arc::<str>::from("sonner.demo.show");
            });
            host.request_redraw(action_cx.window);
        }))
        .test_id("ui-gallery-sonner-demo-show")
        .into_element(cx);

    wrap_controls_row::<fret_app::App>(Space::N2, vec![show])
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-demo"),
        )
        .test_id("ui-gallery-sonner-demo-root")
}
// endregion: example
