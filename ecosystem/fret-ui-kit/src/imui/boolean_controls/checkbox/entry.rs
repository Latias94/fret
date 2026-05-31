use std::sync::Arc;

use fret_ui::UiHost;

use super::super::super::label_identity::parse_label_identity;
use super::super::super::{CheckboxOptions, ResponseExt, UiWriterImUiFacadeExt};

mod render;

use render::checkbox_model_with_options_inner;

pub(in crate::imui) fn checkbox_model<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
) -> ResponseExt {
    checkbox_model_with_options(ui, label, model, CheckboxOptions::default())
}

pub(in crate::imui) fn checkbox_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
    options: CheckboxOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("checkbox-label", identity), |ui| {
        checkbox_model_with_options_inner(ui, visible_label, model, options)
    })
}
