pub const SOURCE: &str = include_str!("sampling.rs");

// region: example
use fret_core::ImageId;
use fret_core::scene::ImageSamplingHint;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    streaming_image: Model<Option<ImageId>>,
) -> AnyElement {
    let linear = shadcn::MediaImage::model(streaming_image.clone())
        .fit(fret_core::ViewportFit::Stretch)
        .loading(true)
        .sampling_hint(ImageSamplingHint::Linear)
        .refine_style(ChromeRefinement::default().rounded(Radius::Md))
        .refine_layout(LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(160.0)))
        .into_element(cx)
        .test_id("ui-gallery-image-sampling-linear");

    let nearest = shadcn::MediaImage::model(streaming_image)
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
