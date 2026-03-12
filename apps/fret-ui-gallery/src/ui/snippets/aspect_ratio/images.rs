use fret_core::{ImageColorSpace, ImageId};
use fret_ui::{ElementContext, UiHost};
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use std::sync::OnceLock;

pub(crate) fn landscape_image_id<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Option<ImageId> {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    let source = SOURCE.get_or_init(|| {
        ImageSource::rgba8(
            320,
            192,
            demo_preview_rgba8(320, 192, (92, 168, 255)),
            ImageColorSpace::Srgb,
        )
    });
    cx.use_image_source_state(source).image
}

pub(crate) fn portrait_image_id<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Option<ImageId> {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    let source = SOURCE.get_or_init(|| {
        ImageSource::rgba8(
            192,
            320,
            demo_preview_rgba8(192, 320, (255, 164, 118)),
            ImageColorSpace::Srgb,
        )
    });
    cx.use_image_source_state(source).image
}

fn demo_preview_rgba8(width: u32, height: u32, accent: (u8, u8, u8)) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;

            let mut r = (20.0 + accent.0 as f32 * (0.35 + 0.65 * fx)) as u8;
            let mut g = (24.0 + accent.1 as f32 * (0.28 + 0.72 * (1.0 - fy))) as u8;
            let mut b = (24.0 + accent.2 as f32 * (0.30 + 0.70 * fy)) as u8;

            let border = x < 3 || y < 3 || x + 3 >= width || y + 3 >= height;
            let horizon = y > height / 2 - 3 && y < height / 2 + 3;
            let badge = x > width / 8 && x < width / 4 && y > height / 10 && y < height / 5;

            if border {
                r = 245;
                g = 245;
                b = 245;
            } else if horizon {
                r = r.saturating_add(18);
                g = g.saturating_add(18);
                b = b.saturating_add(12);
            } else if badge {
                r = 250;
                g = 250;
                b = 250;
            }

            out[idx] = r;
            out[idx + 1] = g;
            out[idx + 2] = b;
            out[idx + 3] = 255;
        }
    }

    out
}
