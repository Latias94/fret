use fret::UiCx;
use fret_core::{ImageColorSpace, ImageId};
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use std::sync::OnceLock;

pub(crate) fn demo_cover_image(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state(demo_cover_source()).image
}

fn demo_cover_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(640, 360, demo_cover_rgba8(640, 360), ImageColorSpace::Srgb)
    })
}

fn demo_cover_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;
            let wave = (((fx * 9.0) - (fy * 4.0)).sin() * 0.5 + 0.5) * 18.0;

            out[idx] = (22.0 + 34.0 * (1.0 - fy) + 148.0 * fx + wave).min(255.0) as u8;
            out[idx + 1] = (30.0 + 86.0 * fy + 76.0 * (1.0 - fx) + wave * 0.55).min(255.0) as u8;
            out[idx + 2] = (54.0 + 112.0 * (1.0 - fy) + 68.0 * fx + wave * 0.35).min(255.0) as u8;
            out[idx + 3] = 255;
        }
    }

    out
}

pub mod card_content;
pub mod compositions;
pub mod demo;
pub mod image;
pub mod meeting_notes;
pub mod rtl;
pub mod size;
pub mod title_children;
pub mod usage;
