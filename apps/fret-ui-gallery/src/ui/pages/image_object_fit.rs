use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::image_object_fit as snippets;

pub(super) fn preview_image_object_fit(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    square_image: Model<Option<ImageId>>,
    wide_image: Model<Option<ImageId>>,
    tall_image: Model<Option<ImageId>>,
    streaming_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let mapping = snippets::mapping::render(
        cx,
        theme,
        square_image.clone(),
        wide_image.clone(),
        tall_image.clone(),
    );

    let sampling = snippets::sampling::render(cx, streaming_image.clone());

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
                .code_rust_from_file_region(
                    include_str!("../snippets/image_object_fit/mapping.rs"),
                    "example",
                ),
            DocSection::new("Sampling", sampling)
                .description("Linear vs nearest sampling (useful for pixel art).")
                .code_rust_from_file_region(
                    include_str!("../snippets/image_object_fit/sampling.rs"),
                    "example",
                ),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-image-object-fit")]
}
