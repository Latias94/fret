pub const SOURCE: &str = include_str!("portrait.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{ImageColorSpace, ImageId, Px};
use fret_ui::Theme;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::OnceLock;

fn portrait_image_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(
            360,
            640,
            portrait_preview_rgba8(360, 640, [236, 128, 180]),
            ImageColorSpace::Srgb,
        )
    })
}

fn portrait_image_id(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state(portrait_image_source()).image
}

fn portrait_preview_rgba8(width: u32, height: u32, accent: [u8; 3]) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;
            let glow = (((fx * 4.0) + (fy * 10.0)).sin() * 0.5 + 0.5) * 22.0;

            let r = (24.0 + accent[0] as f32 * (0.26 + 0.30 * (1.0 - fy)) + glow).min(255.0);
            let g = (18.0 + accent[1] as f32 * (0.22 + 0.34 * fx) + glow * 0.72).min(255.0);
            let b = (34.0 + accent[2] as f32 * (0.30 + 0.36 * (1.0 - fx)) + glow * 0.48).min(255.0);

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

fn ratio_example<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    content: AnyElement,
    ratio: f32,
    max_w: Px,
    test_id: &'static str,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let muted_bg = theme.color_token("muted");
    let border = theme.color_token("border");

    let frame = shadcn::AspectRatio::with_child(content)
        .ratio(ratio)
        .refine_style(
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .border_1()
                .bg(ColorRef::Color(muted_bg))
                .border_color(ColorRef::Color(border)),
        )
        .refine_layout(LayoutRefinement::default().w_full().max_w(max_w))
        .into_element(cx)
        .test_id(test_id);

    ui::h_flex(move |_cx| vec![frame])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_center()
        .into_element(cx)
}

#[allow(dead_code)]
pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let content = shadcn::MediaImage::maybe(portrait_image_id(cx))
        .loading(true)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-portrait-content");

    ratio_example(
        cx,
        content,
        9.0 / 16.0,
        Px(160.0),
        "ui-gallery-aspect-ratio-portrait",
    )
}
// endregion: example

use fret::component::prelude::Model;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;

pub fn render_preview<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    demo_image: Option<Model<Option<fret_core::ImageId>>>,
) -> impl IntoUiElement<H> + use<H> {
    let model_image_id = demo_image
        .as_ref()
        .and_then(|model| cx.watch_model(model).layout().cloned().flatten());
    let asset_image = super::images::portrait_image_state(cx);
    let image_id = model_image_id.or(asset_image.image);
    let loading = model_image_id.is_none() && asset_image.loading;

    let content = shadcn::MediaImage::maybe(image_id)
        .loading(loading)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-portrait-content");

    ratio_example(
        cx,
        content,
        9.0 / 16.0,
        Px(160.0),
        "ui-gallery-aspect-ratio-portrait",
    )
    .into_element(cx)
}
