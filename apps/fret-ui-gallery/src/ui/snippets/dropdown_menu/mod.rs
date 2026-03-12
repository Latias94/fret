//! Snippet-backed Dropdown Menu examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-dropdown-menu-*` `test_id`s stable: diag scripts depend on them.

use fret_core::Px;
use fret_ui_kit::{IntoUiElement, LayoutRefinement, ui};
use fret_ui_shadcn::prelude::*;

pub mod avatar;
pub mod basic;
pub mod checkboxes;
pub mod checkboxes_icons;
pub mod complex;
pub mod demo;
pub mod destructive;
pub mod icons;
pub mod parts;
pub mod radio_group;
pub mod radio_icons;
pub mod rtl;
pub mod shortcuts;
pub mod submenu;
pub mod usage;

/// Match shadcn docs preview behavior locally without changing the global docs shell.
fn preview_frame<H: UiHost, B>(body: B) -> impl IntoUiElement<H> + use<H, B>
where
    B: IntoUiElement<H>,
{
    ui::h_flex(move |cx| [body.into_element(cx)])
        .layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .h_px(Px(288.0))
                .overflow_visible(),
        )
        .items_center()
        .justify_center()
}

fn preview_frame_with<H: UiHost, F, B>(
    cx: &mut ElementContext<'_, H>,
    build: F,
) -> impl IntoUiElement<H> + use<H, F, B>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> B,
    B: IntoUiElement<H>,
{
    let body = build(cx);
    preview_frame(body)
}
