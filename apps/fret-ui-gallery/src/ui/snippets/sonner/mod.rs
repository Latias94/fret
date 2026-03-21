use fret::{UiChild, UiCx};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

pub(crate) const LOCAL_TOASTER_ID: &str = "ui-gallery-sonner-local";

pub(crate) fn last_action_model(cx: &mut UiCx<'_>) -> Model<Arc<str>> {
    cx.local_model_keyed("ui-gallery-sonner-last-action", || {
        Arc::<str>::from("<none>")
    })
}

pub(crate) fn position_model(cx: &mut UiCx<'_>) -> Model<shadcn::ToastPosition> {
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

pub(crate) fn local_toaster(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let position = position_model(cx);
    let position = cx
        .get_model_copied(&position, Invalidation::Layout)
        .unwrap_or(shadcn::ToastPosition::TopCenter);

    shadcn::Toaster::new()
        .id(LOCAL_TOASTER_ID)
        .position(position)
}

pub mod demo;
pub mod description;
pub mod extras;
pub mod notes;
pub mod position;
pub mod setup;
pub mod types;
pub mod usage;
