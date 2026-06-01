//! Immediate-mode selectable row helpers.

use std::sync::Arc;

use fret_ui::UiHost;

use super::label_identity::parse_label_identity;
use super::{ResponseExt, SelectableOptions, UiWriterImUiFacadeExt};

mod behavior;
mod entry;
mod keyboard;
mod props;
mod visual;

pub(super) fn selectable_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: SelectableOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("selectable-label", identity), |ui| {
        entry::selectable_with_visible_label(ui, visible_label, options)
    })
}

#[cfg(test)]
mod tests;
