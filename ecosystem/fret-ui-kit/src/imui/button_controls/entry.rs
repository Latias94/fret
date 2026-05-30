use std::sync::Arc;

use fret_ui::UiHost;

use super::super::label_identity::parse_label_identity;
use super::behavior;
use super::{ButtonOptions, ResponseExt, UiWriterImUiFacadeExt};

pub(super) fn button_impl<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: ButtonOptions,
    action: Option<behavior::ButtonAction>,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("button-label", identity), |ui| {
        behavior::button_pressable(ui, visible_label, options, action)
    })
}
