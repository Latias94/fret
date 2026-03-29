//! Snippet-backed Avatar examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling the snippet `render(...)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-avatar-*` `test_id`s stable: diag scripts depend on them.

use fret_core::{ImageColorSpace, ImageId};
use fret_ui::{ElementContext, UiHost};
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use std::sync::OnceLock;

pub(crate) fn demo_image<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Option<ImageId> {
    cx.use_image_source_state(demo_image_source()).image
}

fn demo_image_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(160, 160, demo_image_rgba8(160, 160), ImageColorSpace::Srgb)
    })
}

fn demo_image_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;
            let ring = (((fx - 0.5).hypot(fy - 0.5) * 10.0).cos() * 0.5 + 0.5) * 22.0;

            out[idx] = (36.0 + 78.0 * (1.0 - fy) + 92.0 * fx + ring).min(255.0) as u8;
            out[idx + 1] = (44.0 + 112.0 * fy + 54.0 * (1.0 - fx) + ring * 0.55).min(255.0) as u8;
            out[idx + 2] = (72.0 + 136.0 * (1.0 - fy) + 46.0 * fx + ring * 0.35).min(255.0) as u8;
            out[idx + 3] = 255;
        }
    }

    out
}

pub mod badge_icon;
pub mod basic;
pub mod demo;
pub mod dropdown;
pub mod fallback_only;
pub mod group;
pub mod group_count;
pub mod group_count_icon;
pub mod rtl;
pub mod sizes;
pub mod usage;
pub mod with_badge;
