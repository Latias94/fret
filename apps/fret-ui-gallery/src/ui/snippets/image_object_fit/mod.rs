//! Snippet-backed `MediaImage` object-fit examples for UI Gallery.

use fret::UiCx;
use fret_core::{ImageColorSpace, ImageId};
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use std::sync::OnceLock;

fn fit_demo_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;

            let mut r = (32.0 + 120.0 * fx) as u8;
            let mut g = (48.0 + 140.0 * (1.0 - fy)) as u8;
            let mut b = (80.0 + 120.0 * fy) as u8;

            let border = x < 4 || y < 4 || x + 4 >= width || y + 4 >= height;
            let center_band = x > width / 4 && x < (width * 3) / 4;
            let horizon = y > height / 3 && y < (height * 2) / 3;

            if border {
                r = 246;
                g = 246;
                b = 246;
            } else if center_band && horizon {
                r = r.saturating_add(24);
                g = g.saturating_add(24);
                b = b.saturating_add(20);
            }

            out[idx] = r;
            out[idx + 1] = g;
            out[idx + 2] = b;
            out[idx + 3] = 255;
        }
    }

    out
}

fn sampling_demo_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let tile_x = (x / 4) % 2;
            let tile_y = (y / 4) % 2;
            let accent = (x / 8 + y / 8) % 3;

            let (r, g, b) = match (tile_x, tile_y, accent) {
                (0, 0, 0) => (245, 120, 88),
                (0, 0, _) => (250, 214, 94),
                (0, 1, 0) => (96, 190, 140),
                (0, 1, _) => (72, 148, 232),
                (1, 0, 0) => (128, 102, 226),
                (1, 0, _) => (212, 118, 210),
                (1, 1, 0) => (36, 40, 52),
                (1, 1, _) => (232, 238, 246),
                _ => (180, 180, 180),
            };

            out[idx] = r;
            out[idx + 1] = g;
            out[idx + 2] = b;
            out[idx + 3] = 255;
        }
    }

    out
}

fn square_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(192, 192, fit_demo_rgba8(192, 192), ImageColorSpace::Srgb)
    })
}

fn wide_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(320, 180, fit_demo_rgba8(320, 180), ImageColorSpace::Srgb)
    })
}

fn tall_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(180, 320, fit_demo_rgba8(180, 320), ImageColorSpace::Srgb)
    })
}

fn sampling_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(32, 32, sampling_demo_rgba8(32, 32), ImageColorSpace::Srgb)
    })
}

pub(crate) fn square_image(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state(square_source()).image
}

pub(crate) fn wide_image(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state(wide_source()).image
}

pub(crate) fn tall_image(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state(tall_source()).image
}

pub(crate) fn sampling_image(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state(sampling_source()).image
}

pub mod mapping;
pub mod sampling;
