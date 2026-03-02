use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_transcript_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::transcript_torture::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("AI transcript torture is a harness surface for long virtualized transcripts."),
        vec![
            DocSection::new("Transcript Torture", demo)
                .max_w(Px(1000.0))
                .test_id_prefix("ui-gallery-ai-transcript-torture")
                .code_rust_from_file_region(snippets::transcript_torture::SOURCE, "example"),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-transcript-torture")]
}

