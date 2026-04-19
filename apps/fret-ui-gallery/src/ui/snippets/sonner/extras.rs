pub const DOCS_SOURCE: &str = include_str!("extras.docs.rs.txt");
#[allow(dead_code)]
pub const SOURCE: &str = include_str!("extras.rs");

// region: example
use super::{last_action_model, message_request, preview_controls_row, preview_frame, request};
use fret::app::AppRenderActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn wrap_controls_row<H: UiHost>(
    gap: Space,
    children: Vec<AnyElement>,
) -> impl IntoUiElement<H> + use<H> {
    preview_controls_row::<H>(gap, children)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let sonner = shadcn::Sonner::global(&mut *cx.app);
    let last_action_model = last_action_model(cx);
    const CMD_TOAST_ACTION: &str = "ui_gallery.toast.action";

    let action = {
        let sonner = sonner.clone();
        let last_action_model = last_action_model.clone();
        shadcn::Button::new("Action")
            .variant(shadcn::ButtonVariant::Outline)
            .on_activate(cx.actions().listen(move |host, action_cx| {
                sonner.toast(
                    host,
                    action_cx.window,
                    message_request(
                        "Event has been created",
                        shadcn::ToastVariant::Default,
                        shadcn::ToastMessageOptions::new().action_id("Undo", CMD_TOAST_ACTION),
                    ),
                );
                let _ = host.models_mut().update(&last_action_model, |v| {
                    *v = Arc::<str>::from("sonner.extras.action");
                });
                host.request_redraw(action_cx.window);
            }))
            .test_id("ui-gallery-sonner-extras-action")
            .into_element(cx)
    };

    let cancel = {
        let sonner = sonner.clone();
        let last_action_model = last_action_model.clone();
        shadcn::Button::new("Cancel")
            .variant(shadcn::ButtonVariant::Outline)
            .on_activate(cx.actions().listen(move |host, action_cx| {
                sonner.toast(
                    host,
                    action_cx.window,
                    message_request(
                        "Event has been created",
                        shadcn::ToastVariant::Default,
                        shadcn::ToastMessageOptions::new().cancel_id("Cancel", CMD_TOAST_ACTION),
                    ),
                );
                let _ = host.models_mut().update(&last_action_model, |v| {
                    *v = Arc::<str>::from("sonner.extras.cancel");
                });
                host.request_redraw(action_cx.window);
            }))
            .test_id("ui-gallery-sonner-extras-cancel")
            .into_element(cx)
    };

    let swipe = shadcn::Button::new("Swipe Dismiss Toast")
        .variant(shadcn::ButtonVariant::Outline)
        .on_activate(cx.actions().listen(move |host, action_cx| {
            sonner.toast(
                host,
                action_cx.window,
                request("Swipe to dismiss")
                    .description("Drag up to dismiss (pinned)")
                    .duration(None)
                    .dismissible(true)
                    .test_id("ui-gallery-sonner-demo-toast-swipe"),
            );
            let _ = host.models_mut().update(&last_action_model, |v| {
                *v = Arc::<str>::from("sonner.extras.swipe_dismiss");
            });
            host.request_redraw(action_cx.window);
        }))
        .test_id("ui-gallery-sonner-demo-show-swipe")
        .into_element(cx);

    preview_frame::<fret_app::App, _>(wrap_controls_row::<fret_app::App>(
        Space::N2,
        vec![action, cancel, swipe],
    ))
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-sonner-extras"),
    )
}
// endregion: example
