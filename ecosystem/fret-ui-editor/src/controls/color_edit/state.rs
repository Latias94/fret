use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::{ElementContext, Invalidation, UiHost};

use super::ColorEditPopupRuntimeOptions;

#[track_caller]
pub(super) fn popup_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    cx.local_model(|| false)
}

#[track_caller]
pub(super) fn tooltip_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    cx.local_model_keyed("tooltip_open", || false)
}

#[track_caller]
pub(super) fn copy_menu_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    cx.local_model_keyed("copy_menu_open", || false)
}

#[track_caller]
pub(super) fn reference_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Color>> {
    cx.local_model(|| None::<Color>)
}

#[track_caller]
pub(super) fn draft_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.local_model(String::new)
}

#[track_caller]
pub(super) fn error_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    cx.local_model(|| None::<Arc<str>>)
}

#[track_caller]
pub(super) fn popup_runtime_options_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    defaults: ColorEditPopupRuntimeOptions,
) -> Model<ColorEditPopupRuntimeOptions> {
    cx.local_model_keyed("popup_runtime_options", move || defaults)
}

pub(super) fn sync_popup_runtime_options<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: &Model<ColorEditPopupRuntimeOptions>,
    defaults: ColorEditPopupRuntimeOptions,
) {
    let needs_sync = cx
        .read_model_ref(model, Invalidation::Paint, |runtime| {
            runtime.needs_default_sync(defaults)
        })
        .unwrap_or(true);
    if !needs_sync {
        return;
    }

    let _ = cx
        .app
        .models_mut()
        .update(model, |runtime| runtime.sync_defaults(defaults));
}
