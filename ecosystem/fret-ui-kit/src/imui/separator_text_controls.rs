//! Immediate-mode section-label helper.

use std::sync::Arc;

use fret_ui::UiHost;

use super::label_identity::parse_label_identity;
use super::{SeparatorTextOptions, UiWriterImUiFacadeExt};

mod element;

pub(super) fn separator_text_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: SeparatorTextOptions,
) {
    let parts = parse_label_identity(label.as_ref());
    let label = Arc::<str>::from(parts.visible);
    let element = ui.with_cx_mut(|cx| element::separator_text_element(cx, label, options));
    ui.add(element);
}
