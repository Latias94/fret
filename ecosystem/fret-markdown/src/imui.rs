//! Immediate-mode (`fret-imui`) adapters for `fret-markdown`.
//!
//! This module is intentionally tiny: it keeps `fret-markdown`'s core rendering APIs in the
//! declarative layer, and only provides ergonomic glue for `ImUi` consumers.

use fret_authoring::UiWriter;
use fret_ui::UiHost;

/// Adds a markdown element to an `imui` output list.
#[track_caller]
pub fn markdown<H: UiHost>(ui: &mut impl UiWriter<H>, source: &str) {
    let element = ui.with_cx_mut(|cx| crate::markdown(cx, source));
    ui.add(element);
}

/// Adds a markdown element to an `imui` output list with a custom component set.
#[track_caller]
pub fn markdown_with<H: UiHost>(
    ui: &mut impl UiWriter<H>,
    source: &str,
    components: &crate::MarkdownComponents<H>,
) {
    let element = ui.with_cx_mut(|cx| crate::markdown_with(cx, source, components));
    ui.add(element);
}
