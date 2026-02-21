use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use fret_core::scene::ImageSamplingHint;

pub(super) fn preview_image_object_fit(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    square_image: Model<Option<ImageId>>,
    wide_image: Model<Option<ImageId>>,
    tall_image: Model<Option<ImageId>>,
    streaming_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let image_cell = |cx: &mut ElementContext<'_, App>,
                      label: &'static str,
                      source: Model<Option<ImageId>>,
                      fit: fret_core::ViewportFit|
     -> AnyElement {
        let label = cx.text(label);
        let image = shadcn::MediaImage::model(source)
            .fit(fit)
            .loading(true)
            .refine_style(ChromeRefinement::default().rounded(Radius::Md))
            .refine_layout(LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(96.0)))
            .into_element(cx)
            .test_id(format!("ui-gallery-image-object-fit-cell-{:?}", fit).to_lowercase());

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default()),
            |_cx| vec![label, image],
        )
    };

    let row = |cx: &mut ElementContext<'_, App>,
               title: &'static str,
               image: Model<Option<ImageId>>|
     -> AnyElement {
        let stretch = image_cell(
            cx,
            "Stretch",
            image.clone(),
            fret_core::ViewportFit::Stretch,
        );
        let contain = image_cell(
            cx,
            "Contain",
            image.clone(),
            fret_core::ViewportFit::Contain,
        );
        let cover = image_cell(cx, "Cover", image, fret_core::ViewportFit::Cover);

        let grid = doc_layout::wrap_row(
            cx,
            theme,
            Space::N4,
            fret_ui::element::CrossAlign::Start,
            |_cx| vec![stretch, contain, cover],
        );

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |cx| vec![cx.text(title), grid],
        )
    };

    let mapping = {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |cx| {
                vec![
                    row(cx, "Wide source → fixed 160×96", wide_image.clone()),
                    row(cx, "Tall source → fixed 160×96", tall_image.clone()),
                    row(cx, "Square source → fixed 160×96", square_image.clone()),
                ]
            },
        )
        .test_id("ui-gallery-image-object-fit-mapping")
    };

    let sampling = {
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

        doc_layout::wrap_row(
            cx,
            theme,
            Space::N4,
            fret_ui::element::CrossAlign::Start,
            |_cx| vec![linear, nearest],
        )
        .test_id("ui-gallery-image-object-fit-sampling")
    };

    let notes = doc_layout::notes(
        cx,
        [
            "Use `ViewportFit::Contain` to avoid cropping; use `Cover` when you want to fill the frame.",
            "Always set explicit frame sizes in docs so `Stretch/Contain/Cover` comparisons are meaningful.",
            "If pixel art looks blurry, prefer `ImageSamplingHint::Nearest` for scaling.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("MediaImage object-fit demo using gallery asset-backed ImageIds."),
        vec![
            DocSection::new("Fit mapping", mapping)
                .max_w(Px(980.0))
                .description(
                    "Compare Stretch / Contain / Cover across wide, tall, and square sources.",
                )
                .code(
                    "rust",
                    r#"shadcn::MediaImage::model(model)
    .fit(fret_core::ViewportFit::Cover)
    .loading(true)
    .into_element(cx);"#,
                ),
            DocSection::new("Sampling", sampling)
                .description("Linear vs nearest sampling (useful for pixel art).")
                .code(
                    "rust",
                    r#"shadcn::MediaImage::model(model)
    .sampling_hint(fret_core::scene::ImageSamplingHint::Nearest)
    .into_element(cx);"#,
                ),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-image-object-fit")]
}
