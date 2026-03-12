//! Snippet-backed Dropdown Menu examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-dropdown-menu-*` `test_id`s stable: diag scripts depend on them.

use fret_core::Px;
use fret_ui_kit::{LayoutRefinement, ui};
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
fn preview_frame<H: UiHost>(cx: &mut ElementContext<'_, H>, body: AnyElement) -> AnyElement {
    ui::h_flex(move |_cx| [body])
        .layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .h_px(Px(288.0))
                .overflow_visible(),
        )
        .items_center()
        .justify_center()
        .into_element(cx)
}

fn preview_frame_with<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    let body = build(cx);
    preview_frame(cx, body)
}
