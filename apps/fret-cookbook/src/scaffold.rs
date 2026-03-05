//! Shared UI scaffolding for cookbook examples.
//!
//! Cookbook examples intentionally live as one-file lessons under `examples/`. This module exists
//! to keep the "page shell" consistent (background, padding, card centering) without turning the
//! cookbook crate into a reusable product API.

use fret::prelude::*;

/// Builds a centered cookbook page with a single primary surface (typically a Card).
///
/// The root node is stamped with a stable `test_id` so scripts can wait for it deterministically.
#[track_caller]
pub fn centered_page<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_test_id: &'static str,
    background_token: &'static str,
    surface: AnyElement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    ui::container(|cx| {
        [ui::v_flex(|_cx| [surface])
            .gap(Space::N6)
            .items_center()
            .justify_center()
            .size_full()
            .into_element(cx)]
    })
    .bg(ColorRef::Color(theme.color_token(background_token)))
    .p(Space::N6)
    .size_full()
    .into_element(cx)
    .test_id(root_test_id)
}

/// Uses the theme `background` token for the page background.
#[track_caller]
pub fn centered_page_background<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_test_id: &'static str,
    surface: AnyElement,
) -> AnyElement {
    centered_page(cx, root_test_id, "background", surface)
}

/// Uses the theme `muted` token for the page background (useful for smaller, dialog-like examples).
#[track_caller]
pub fn centered_page_muted<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_test_id: &'static str,
    surface: AnyElement,
) -> AnyElement {
    centered_page(cx, root_test_id, "muted", surface)
}
