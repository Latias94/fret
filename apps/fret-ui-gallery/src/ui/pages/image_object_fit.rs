use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::image_object_fit as snippets;

pub(super) fn preview_image_object_fit(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let mapping = snippets::mapping::render(cx);
    let sampling = snippets::sampling::render(cx);

    let notes = doc_layout::notes_block([
        "Use `ViewportFit::Contain` to avoid cropping; use `Cover` when you want to fill the frame.",
        "Always set explicit frame sizes in docs so `Stretch/Contain/Cover` comparisons are meaningful.",
        "If pixel art looks blurry, prefer `ImageSamplingHint::Nearest` for scaling.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes).description("Usage notes.");
    let mapping = DocSection::build(cx, "Fit mapping", mapping)
        .max_w(Px(980.0))
        .description("Compare Stretch / Contain / Cover across wide, tall, and square sources.")
        .code_rust_from_file_region(snippets::mapping::SOURCE, "example");
    let sampling = DocSection::build(cx, "Sampling", sampling)
        .description("Linear vs nearest sampling (useful for pixel art).")
        .code_rust_from_file_region(snippets::sampling::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some("MediaImage object-fit demo using self-contained generated ImageSources."),
        vec![mapping, sampling, notes],
    );

    vec![body.test_id("ui-gallery-image-object-fit").into_element(cx)]
}
