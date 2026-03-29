pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{ImageColorSpace, ImageId, Px};
use fret_ui::Theme;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::OnceLock;

fn usage_landscape_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(
            640,
            360,
            usage_preview_rgba8(640, 360, [108, 162, 226]),
            ImageColorSpace::Srgb,
        )
    })
}

fn usage_landscape_image_id(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state(usage_landscape_source()).image
}

fn usage_preview_rgba8(width: u32, height: u32, accent: [u8; 3]) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;
            let stripe = (((fx * 6.0) + (fy * 4.0)).sin() * 0.5 + 0.5) * 18.0;

            let r = (16.0 + 38.0 * (1.0 - fy) + accent[0] as f32 * (0.30 + 0.38 * fx) + stripe)
                .min(255.0);
            let g =
                (22.0 + 44.0 * fx + accent[1] as f32 * (0.24 + 0.34 * (1.0 - fy)) + stripe * 0.55)
                    .min(255.0);
            let b = (32.0 + 62.0 * fy + accent[2] as f32 * (0.22 + 0.36 * (1.0 - fx))).min(255.0);

            let (r, g, b) = if x < 6 || y < 6 || x + 6 >= width || y + 6 >= height {
                (236.0, 239.0, 244.0)
            } else {
                (r, g, b)
            };

            out[idx] = r as u8;
            out[idx + 1] = g as u8;
            out[idx + 2] = b as u8;
            out[idx + 3] = 255;
        }
    }

    out
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app);
    let muted_bg = theme.color_token("muted");
    let image = shadcn::MediaImage::maybe(usage_landscape_image_id(cx))
        .loading(true)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-usage-content");

    let frame = shadcn::AspectRatio::with_child(image)
        .ratio(16.0 / 9.0)
        .refine_style(
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .bg(ColorRef::Color(muted_bg)),
        )
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-usage");

    ui::h_flex(move |_cx| vec![frame])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_center()
        .into_element(cx)
}
// endregion: example
