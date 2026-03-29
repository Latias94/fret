pub const SOURCE: &str = include_str!("composable_children.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{ImageColorSpace, ImageId, Px};
use fret_ui::Theme;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::OnceLock;

fn composable_landscape_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(
            640,
            360,
            composable_preview_rgba8(640, 360, [120, 174, 236]),
            ImageColorSpace::Srgb,
        )
    })
}

fn composable_landscape_image_id(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state(composable_landscape_source())
        .image
}

fn composable_preview_rgba8(width: u32, height: u32, accent: [u8; 3]) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;
            let band = (((fx * 8.0) + (fy * 6.0)).sin() * 0.5 + 0.5) * 18.0;

            let r = (18.0 + 44.0 * (1.0 - fy) + accent[0] as f32 * (0.34 + 0.32 * fx) + band)
                .min(255.0);
            let g = (24.0 + 36.0 * fx + accent[1] as f32 * (0.28 + 0.30 * (1.0 - fy)) + band * 0.6)
                .min(255.0);
            let b = (32.0 + 60.0 * fy + accent[2] as f32 * (0.26 + 0.34 * (1.0 - fx))).min(255.0);

            let (r, g, b) = if x < 8 || y < 8 || x + 8 >= width || y + 8 >= height {
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
    let border = theme.color_token("border");

    let image = shadcn::MediaImage::maybe(composable_landscape_image_id(cx))
        .loading(true)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-composable-children-image");

    let badge = shadcn::Badge::new("Featured")
        .variant(shadcn::BadgeVariant::Secondary)
        .refine_layout(
            LayoutRefinement::default()
                .absolute()
                .left_px(Px(12.0))
                .bottom_px(Px(12.0)),
        )
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-composable-children-badge");

    let frame = shadcn::AspectRatio::with_children([image, badge])
        .ratio(16.0 / 9.0)
        .refine_style(
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .border_1()
                .bg(ColorRef::Color(muted_bg))
                .border_color(ColorRef::Color(border)),
        )
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-composable-children");

    ui::h_flex(move |_cx| vec![frame])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_center()
        .into_element(cx)
}
// endregion: example
