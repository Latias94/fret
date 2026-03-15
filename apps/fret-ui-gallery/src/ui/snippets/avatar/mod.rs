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
        // Keep avatar snippets self-contained instead of depending on runtime bootstrap assets.
        ImageSource::rgba8(96, 96, demo_avatar_rgba8(96, 96), ImageColorSpace::Srgb)
    })
}

fn demo_avatar_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = ((dx * dx + dy * dy).sqrt()) / (width.min(height) as f32 * 0.5);

            let (mut r, mut g, mut b) = if distance <= 0.44 {
                (248u8, 215u8, 184u8)
            } else {
                (
                    (42.0 + 90.0 * fx) as u8,
                    (54.0 + 86.0 * (1.0 - fy)) as u8,
                    (110.0 + 104.0 * fy) as u8,
                )
            };

            let eye_band = y > height / 3 && y < height / 2;
            let left_eye = x > width / 3 - 6 && x < width / 3 + 2;
            let right_eye = x > (width * 2) / 3 - 2 && x < (width * 2) / 3 + 6;
            let outline = x < 2 || y < 2 || x + 2 >= width || y + 2 >= height;

            if eye_band && (left_eye || right_eye) {
                r = 18;
                g = 18;
                b = 24;
            } else if outline {
                r = r.saturating_add(8);
                g = g.saturating_add(8);
                b = b.saturating_add(8);
            }

            out[idx] = r;
            out[idx + 1] = g;
            out[idx + 2] = b;
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
