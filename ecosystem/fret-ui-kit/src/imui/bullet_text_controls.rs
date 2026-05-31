//! Immediate-mode bullet-list helper.

mod element;

use std::sync::Arc;

use fret_ui::UiHost;

use super::{BulletTextOptions, UiWriterImUiFacadeExt};

pub(in crate::imui::bullet_text_controls) use element::bullet_text_element;

pub(super) fn bullet_text_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    text: Arc<str>,
    options: BulletTextOptions,
) {
    let element = ui.with_cx_mut(|cx| bullet_text_element(cx, text, options));
    ui.add(element);
}

#[cfg(test)]
mod tests;
