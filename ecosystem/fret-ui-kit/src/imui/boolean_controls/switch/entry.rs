use std::sync::Arc;

use fret_ui::UiHost;

use super::super::super::label_identity::parse_label_identity;
use super::super::super::{ResponseExt, SwitchOptions, UiWriterImUiFacadeExt};

mod render;

use render::switch_model_with_options_inner;

pub(in crate::imui) fn switch_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
    options: SwitchOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("switch-label", identity), |ui| {
        switch_model_with_options_inner(ui, visible_label, model, options)
    })
}
