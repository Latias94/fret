//! Immediate-mode (`fret-imui`) adapters for `fret-code-view`.
//!
//! This module keeps `fret-code-view`'s primary APIs in the declarative authoring layer and adds a
//! small ergonomic bridge for `ImUi` consumers.

use fret_authoring::UiWriter;
use fret_ui::UiHost;

/// Adds a code block element to an `imui` output list.
#[track_caller]
pub fn code_block<H: UiHost>(
    ui: &mut impl UiWriter<H>,
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
) {
    let element = ui.with_cx_mut(|cx| crate::code_block(cx, code, language, show_line_numbers));
    ui.add(element);
}
