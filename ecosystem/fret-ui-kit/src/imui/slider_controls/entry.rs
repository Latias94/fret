use std::sync::Arc;

use fret_ui::UiHost;

mod element;

use element::slider_f32_model_element;

use super::super::label_identity::parse_label_identity;
use super::super::{ResponseExt, SliderOptions, UiWriterImUiFacadeExt};

pub(in crate::imui) fn slider_f32_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<f32>,
    options: SliderOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("slider-label", identity), |ui| {
        slider_f32_model_element(ui, visible_label, model, options)
    })
}
