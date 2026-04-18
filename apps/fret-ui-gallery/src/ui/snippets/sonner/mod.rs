use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{Invalidation, UiHost};
use fret_ui_kit::{IntoUiElement, LayoutRefinement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub(crate) const LOCAL_TOASTER_ID: &str = "ui-gallery-sonner-local";

pub(crate) fn last_action_model(cx: &mut AppComponentCx<'_>) -> Model<Arc<str>> {
    cx.local_model_keyed("ui-gallery-sonner-last-action", || {
        Arc::<str>::from("<none>")
    })
}

pub(crate) fn position_model(cx: &mut AppComponentCx<'_>) -> Model<shadcn::ToastPosition> {
    cx.local_model_keyed("ui-gallery-sonner-position", || {
        shadcn::ToastPosition::TopCenter
    })
}

pub(crate) fn request(title: impl Into<Arc<str>>) -> shadcn::ToastRequest {
    shadcn::ToastRequest::new(title).toaster_id(LOCAL_TOASTER_ID)
}

pub(crate) fn message_request(
    title: impl Into<Arc<str>>,
    variant: shadcn::ToastVariant,
    options: shadcn::ToastMessageOptions,
) -> shadcn::ToastRequest {
    let mut request = request(title).variant(variant);

    if let Some(description) = options.description {
        request = request.description(description);
    }
    if let Some(action) = options.action {
        request = request.action(action);
    }
    if let Some(cancel) = options.cancel {
        request = request.cancel(cancel);
    }
    if let Some(icon) = options.icon {
        request = request.icon(icon);
    }
    if let Some(duration) = options.duration {
        request = request.duration(duration);
    }
    if let Some(dismissible) = options.dismissible {
        request = request.dismissible(dismissible);
    }

    request
}

pub(crate) fn local_toaster(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let position = position_model(cx);
    let position = cx
        .get_model_copied(&position, Invalidation::Layout)
        .unwrap_or(shadcn::ToastPosition::TopCenter);

    shadcn::Toaster::new()
        .id(LOCAL_TOASTER_ID)
        .position(position)
}

/// Match shadcn docs preview behavior locally without changing the shared docs shell.
pub(crate) fn preview_frame<H: UiHost, B>(body: B) -> impl IntoUiElement<H> + use<H, B>
where
    B: IntoUiElement<H>,
{
    ui::h_flex(move |cx| [body.into_element(cx)])
        .layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .h_px(Px(224.0))
                .overflow_visible(),
        )
        .items_center()
        .justify_center()
}

pub(crate) fn preview_controls_row<H: UiHost>(
    gap: Space,
    children: Vec<AnyElement>,
) -> impl IntoUiElement<H> + use<H> {
    ui::h_flex(move |_cx| children)
        .gap(gap)
        .items_center()
        .justify_center()
        .wrap()
        .layout(LayoutRefinement::default().w_full().min_w_0())
}

pub(crate) fn preview_stack<H: UiHost>(
    gap: Space,
    children: Vec<AnyElement>,
) -> impl IntoUiElement<H> + use<H> {
    preview_frame(
        ui::v_flex(move |_cx| children)
            .gap(gap)
            .items_center()
            .layout(
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .max_w(Px(560.0)),
            ),
    )
}

pub(crate) fn preview_note(cx: &mut AppComponentCx<'_>, text: impl Into<Arc<str>>) -> AnyElement {
    let text = text.into();
    ui::h_flex(move |cx| [shadcn::raw::typography::muted(text.clone()).into_element(cx)])
        .items_center()
        .justify_center()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}

pub mod demo;
pub mod description;
pub mod extras;
pub mod notes;
pub mod position;
pub mod setup;
pub mod types;
pub mod usage;
