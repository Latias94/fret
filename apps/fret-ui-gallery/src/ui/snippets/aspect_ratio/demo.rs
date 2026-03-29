pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{ImageColorSpace, ImageId};
use fret_ui::Theme;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::OnceLock;

fn demo_landscape_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(
            640,
            360,
            demo_preview_rgba8(640, 360, [116, 170, 236]),
            ImageColorSpace::Srgb,
        )
    })
}

fn demo_landscape_image_id(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state(demo_landscape_source()).image
}

fn demo_preview_rgba8(width: u32, height: u32, accent: [u8; 3]) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;
            let wave = (((fx * 7.0) - (fy * 5.0)).sin() * 0.5 + 0.5) * 24.0;

            let r = (20.0 + 42.0 * (1.0 - fy) + accent[0] as f32 * (0.34 + 0.32 * fx) + wave)
                .min(255.0);
            let g = (26.0
                + 54.0 * (1.0 - fy)
                + accent[1] as f32 * (0.26 + 0.34 * (1.0 - fx))
                + wave * 0.6)
                .min(255.0);
            let b =
                (38.0 + 68.0 * fy + accent[2] as f32 * (0.24 + 0.36 * fx) + wave * 0.4).min(255.0);

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

fn render_frame<H: UiHost, E>(image: E) -> impl IntoUiElement<H> + use<H, E>
where
    E: IntoUiElement<H>,
{
    ui::h_flex(move |cx| {
        let theme = Theme::global(&*cx.app);
        let muted_bg = theme.color_token("muted");
        let frame = shadcn::AspectRatio::with_child(image.into_element(cx))
            .ratio(16.0 / 9.0)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .bg(ColorRef::Color(muted_bg)),
            )
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
            .into_element(cx)
            .test_id("ui-gallery-aspect-ratio-demo");

        [frame]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .justify_center()
}

#[allow(dead_code)]
pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let image = shadcn::MediaImage::maybe(demo_landscape_image_id(cx))
        .loading(true)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-demo-content");

    render_frame(image).into_element(cx)
}
// endregion: example

use fret::component::prelude::Model;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::primitives::visually_hidden::visually_hidden_label;

fn preview_status_marker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    image_id: Option<fret_core::ImageId>,
    loading: bool,
) -> AnyElement {
    let (test_id, label) = if image_id.is_some() {
        (
            "ui-gallery-aspect-ratio-demo-image-status-loaded",
            "Aspect Ratio demo image loaded",
        )
    } else if loading {
        (
            "ui-gallery-aspect-ratio-demo-image-status-loading",
            "Aspect Ratio demo image loading",
        )
    } else {
        (
            "ui-gallery-aspect-ratio-demo-image-status-missing",
            "Aspect Ratio demo image unavailable",
        )
    };

    visually_hidden_label(cx, label).test_id(test_id)
}

pub fn render_preview<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    demo_image: Option<Model<Option<fret_core::ImageId>>>,
) -> impl IntoUiElement<H> + use<H> {
    let model_image_id = demo_image
        .as_ref()
        .and_then(|model| cx.watch_model(model).layout().cloned().flatten());
    let asset_image = super::images::landscape_image_state(cx);
    let image_id = model_image_id.or(asset_image.image);
    let loading = model_image_id.is_none() && asset_image.loading;
    let status_marker = preview_status_marker(cx, image_id, loading);

    let image = shadcn::MediaImage::maybe(image_id)
        .loading(loading)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-demo-content");

    ui::v_flex(move |cx| vec![status_marker, render_frame(image).into_element(cx)])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}
