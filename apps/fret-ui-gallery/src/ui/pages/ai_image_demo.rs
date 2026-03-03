use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_image_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::image_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("AI Image is a presentation surface; apps own image generation pipelines."),
        vec![
            DocSection::new("Image", demo)
                .test_id_prefix("ui-gallery-ai-image-demo")
                .code_rust_from_file_region(snippets::image_demo::SOURCE, "example"),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-image-demo")]
}
