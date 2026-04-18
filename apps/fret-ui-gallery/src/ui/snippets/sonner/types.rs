pub const DOCS_SOURCE: &str = include_str!("types.docs.rs.txt");
#[allow(dead_code)]
pub const SOURCE: &str = include_str!("types.rs");

// region: example
use super::{
    last_action_model, message_request, preview_controls_row, preview_note, preview_stack, request,
};
use fret::app::UiCxActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_ui::Invalidation;
use fret_ui::UiHost;
use fret_ui::action::{ActionCx, UiActionHost};
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

fn type_button(
    cx: &mut AppComponentCx<'_>,
    label: &'static str,
    test_id: &'static str,
    on_activate: impl Fn(&mut dyn UiActionHost, ActionCx) + 'static,
) -> AnyElement {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .on_activate(cx.actions().listen(on_activate))
        .test_id(test_id)
        .into_element(cx)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let sonner = shadcn::Sonner::global(&mut *cx.app);
    let last_action = last_action_model(cx);
    let pending_promise = cx.local_model_keyed("pending_promise", || None::<shadcn::ToastId>);

    let default_button = {
        let sonner = sonner.clone();
        let last_action = last_action.clone();
        type_button(
            cx,
            "Default",
            "ui-gallery-sonner-types-default",
            move |host, action_cx| {
                sonner.toast(
                    host,
                    action_cx.window,
                    message_request(
                        "Event has been created",
                        shadcn::ToastVariant::Default,
                        shadcn::ToastMessageOptions::new(),
                    ),
                );
                let _ = host.models_mut().update(&last_action, |value| {
                    *value = Arc::<str>::from("sonner.types.default");
                });
                host.request_redraw(action_cx.window);
            },
        )
    };

    let success_button = {
        let sonner = sonner.clone();
        let last_action = last_action.clone();
        type_button(
            cx,
            "Success",
            "ui-gallery-sonner-types-success",
            move |host, action_cx| {
                sonner.toast(
                    host,
                    action_cx.window,
                    message_request(
                        "Event has been created",
                        shadcn::ToastVariant::Success,
                        shadcn::ToastMessageOptions::new(),
                    ),
                );
                let _ = host.models_mut().update(&last_action, |value| {
                    *value = Arc::<str>::from("sonner.types.success");
                });
                host.request_redraw(action_cx.window);
            },
        )
    };

    let info_button = {
        let sonner = sonner.clone();
        let last_action = last_action.clone();
        type_button(
            cx,
            "Info",
            "ui-gallery-sonner-types-info",
            move |host, action_cx| {
                sonner.toast(
                    host,
                    action_cx.window,
                    message_request(
                        "Be at the area 10 minutes before the event time",
                        shadcn::ToastVariant::Info,
                        shadcn::ToastMessageOptions::new(),
                    ),
                );
                let _ = host.models_mut().update(&last_action, |value| {
                    *value = Arc::<str>::from("sonner.types.info");
                });
                host.request_redraw(action_cx.window);
            },
        )
    };

    let warning_button = {
        let sonner = sonner.clone();
        let last_action = last_action.clone();
        type_button(
            cx,
            "Warning",
            "ui-gallery-sonner-types-warning",
            move |host, action_cx| {
                sonner.toast(
                    host,
                    action_cx.window,
                    message_request(
                        "Event start time cannot be earlier than 8am",
                        shadcn::ToastVariant::Warning,
                        shadcn::ToastMessageOptions::new(),
                    ),
                );
                let _ = host.models_mut().update(&last_action, |value| {
                    *value = Arc::<str>::from("sonner.types.warning");
                });
                host.request_redraw(action_cx.window);
            },
        )
    };

    let error_button = {
        let sonner = sonner.clone();
        let last_action = last_action.clone();
        type_button(
            cx,
            "Error",
            "ui-gallery-sonner-types-error",
            move |host, action_cx| {
                sonner.toast(
                    host,
                    action_cx.window,
                    message_request(
                        "Event has not been created",
                        shadcn::ToastVariant::Error,
                        shadcn::ToastMessageOptions::new(),
                    ),
                );
                let _ = host.models_mut().update(&last_action, |value| {
                    *value = Arc::<str>::from("sonner.types.error");
                });
                host.request_redraw(action_cx.window);
            },
        )
    };

    let promise_button = {
        let sonner = sonner.clone();
        let last_action = last_action.clone();
        let pending_model = pending_promise.clone();
        type_button(
            cx,
            "Promise",
            "ui-gallery-sonner-types-promise",
            move |host, action_cx| {
                let pending = host.models_mut().get_copied(&pending_model).flatten();
                if let Some(id) = pending {
                    sonner.toast_success_update(
                        host,
                        action_cx.window,
                        id,
                        "Event has been created",
                    );
                    let _ = host
                        .models_mut()
                        .update(&pending_model, |slot| *slot = None);
                    let _ = host.models_mut().update(&last_action, |value| {
                        *value = Arc::<str>::from("sonner.types.promise.resolve");
                    });
                } else {
                    let promise =
                        sonner.toast_promise_with(host, action_cx.window, request("Loading..."));
                    let _ = host
                        .models_mut()
                        .update(&pending_model, |slot| *slot = Some(promise.id()));
                    let _ = host.models_mut().update(&last_action, |value| {
                        *value = Arc::<str>::from("sonner.types.promise.start");
                    });
                }
                host.request_redraw(action_cx.window);
            },
        )
    };

    let pending = cx
        .get_model_copied(&pending_promise, Invalidation::Layout)
        .flatten()
        .is_some();

    let buttons = wrap_controls_row::<fret_app::App>(
        Space::N2,
        vec![
            default_button,
            success_button,
            info_button,
            warning_button,
            error_button,
            promise_button,
        ],
    )
    .into_element(cx);

    preview_stack::<fret_app::App>(
        Space::N3,
        vec![
            buttons,
            preview_note(
                cx,
                if pending {
                    "Promise toast pending: click Promise again to resolve."
                } else {
                    "Promise toast idle: click Promise to start loading state."
                },
            ),
        ],
    )
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-sonner-types"),
    )
    .test_id("ui-gallery-sonner-types-root")
}
// endregion: example
