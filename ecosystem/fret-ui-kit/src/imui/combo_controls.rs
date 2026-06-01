//! Immediate-mode combo helpers.

mod entry;
mod state;
mod trigger;

use std::sync::Arc;

use fret_ui::UiHost;

use super::{ComboOptions, ComboResponse, UiWriterImUiFacadeExt};

pub(super) fn combo_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    preview: Arc<str>,
    options: ComboOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut super::ImUiFacade<'cx2, 'a2, H>),
) -> ComboResponse {
    entry::combo_with_options(ui, id, label, preview, options, f)
}

#[cfg(test)]
mod tests;
