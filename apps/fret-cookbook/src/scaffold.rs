//! Shared UI scaffolding for cookbook examples.
//!
//! Cookbook examples intentionally live as one-file lessons under `examples/`. This module exists
//! to keep the "page shell" consistent (background, padding, card centering) without turning the
//! cookbook crate into a reusable product API.

use fret::app::prelude::*;
use fret::style::{ColorRef, Space, Theme};

/// Builds a centered cookbook page with a single primary surface (typically a Card).
///
/// The root node is stamped with a stable `test_id` so scripts can wait for it deterministically.
#[track_caller]
pub fn centered_page<B>(
    cx: &mut UiCx<'_>,
    root_test_id: &'static str,
    background_token: &'static str,
    surface: B,
) -> Ui
where
    B: UiChild,
{
    let theme = Theme::global(&*cx.app).snapshot();

    ui::container(move |cx| {
        ui::single(
            cx,
            ui::v_flex(move |cx| ui::single(cx, surface))
                .gap(Space::N6)
                .items_center()
                .justify_center()
                .size_full(),
        )
    })
    .bg(ColorRef::Color(theme.color_token(background_token)))
    .p(Space::N6)
    .size_full()
    .test_id(root_test_id)
    .into_element(cx)
    .into()
}

/// Uses the theme `background` token for the page background.
#[track_caller]
pub fn centered_page_background<B>(cx: &mut UiCx<'_>, root_test_id: &'static str, surface: B) -> Ui
where
    B: UiChild,
{
    centered_page(cx, root_test_id, "background", surface)
}

/// Uses the theme `muted` token for the page background (useful for smaller, dialog-like examples).
#[track_caller]
pub fn centered_page_muted<B>(cx: &mut UiCx<'_>, root_test_id: &'static str, surface: B) -> Ui
where
    B: UiChild,
{
    centered_page(cx, root_test_id, "muted", surface)
}
