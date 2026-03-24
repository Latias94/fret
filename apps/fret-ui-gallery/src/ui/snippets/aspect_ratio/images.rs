#[allow(dead_code)]
pub const SOURCE: &str = include_str!("images.rs");

use fret_core::{ImageColorSpace, ImageId};
use fret_ui::{ElementContext, UiHost};
use fret_ui_assets::ImageSource;
use fret_ui_assets::image_asset_state::ImageLoadingStatus;
use fret_ui_assets::ui::ImageSourceElementContextExt as _;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PreviewImageState {
    pub image: Option<ImageId>,
    pub loading: bool,
}

fn preview_image_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    source: &'static ImageSource,
) -> PreviewImageState {
    let state = cx.use_image_source_state(source);
    PreviewImageState {
        image: state.image,
        loading: state.image.is_none()
            && matches!(
                state.status,
                ImageLoadingStatus::Idle | ImageLoadingStatus::Loading
            ),
    }
}

pub(crate) fn landscape_image_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> PreviewImageState {
    preview_image_state(cx, landscape_source())
}

pub(crate) fn portrait_image_state<H: UiHost>(cx: &mut ElementContext<'_, H>) -> PreviewImageState {
    preview_image_state(cx, portrait_source())
}

fn landscape_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(
            640,
            360,
            preview_rgba8(640, 360, [112, 172, 242]),
            ImageColorSpace::Srgb,
        )
    })
}

fn portrait_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(
            360,
            640,
            preview_rgba8(360, 640, [236, 128, 180]),
            ImageColorSpace::Srgb,
        )
    })
}

fn preview_rgba8(width: u32, height: u32, accent: [u8; 3]) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;
            let stripe = (((fx * 8.0) + (fy * 5.0)).sin() * 0.5 + 0.5) * 20.0;
            let frame = if x < 8 || y < 8 || x + 8 >= width || y + 8 >= height {
                230.0
            } else {
                0.0
            };

            out[idx] =
                (18.0 + 48.0 * (1.0 - fy) + accent[0] as f32 * (0.35 + 0.35 * fx) + stripe + frame)
                    .min(255.0) as u8;
            out[idx + 1] = (24.0
                + 42.0 * fx
                + accent[1] as f32 * (0.28 + 0.32 * (1.0 - fy))
                + stripe * 0.7
                + frame)
                .min(255.0) as u8;
            out[idx + 2] = (34.0
                + 56.0 * fy
                + accent[2] as f32 * (0.25 + 0.34 * (1.0 - fx))
                + stripe * 0.5
                + frame)
                .min(255.0) as u8;
            out[idx + 3] = 255;
        }
    }

    out
}
