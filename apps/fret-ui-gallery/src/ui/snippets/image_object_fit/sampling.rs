pub const SOURCE: &str = include_str!("sampling.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::scene::ImageSamplingHint;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let sampling_image = super::sampling_image(cx);

    let linear = shadcn::MediaImage::maybe(sampling_image.clone())
        .fit(fret_core::ViewportFit::Stretch)
        .loading(true)
        .sampling_hint(ImageSamplingHint::Linear)
        .refine_style(ChromeRefinement::default().rounded(Radius::Md))
        .refine_layout(LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(160.0)))
        .into_element(cx)
        .test_id("ui-gallery-image-sampling-linear");

    let nearest = shadcn::MediaImage::maybe(sampling_image)
        .fit(fret_core::ViewportFit::Stretch)
        .loading(true)
        .sampling_hint(ImageSamplingHint::Nearest)
        .refine_style(ChromeRefinement::default().rounded(Radius::Md))
        .refine_layout(LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(160.0)))
        .into_element(cx)
        .test_id("ui-gallery-image-sampling-nearest");

    ui::h_flex(|_cx| vec![linear, nearest])
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-image-object-fit-sampling")
}
// endregion: example
