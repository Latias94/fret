use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_attachments_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let usage = snippets::attachments_usage::render(cx);
    let grid = snippets::attachments_grid::render(cx);
    let inline = snippets::attachments_inline::render(cx);
    let list = snippets::attachments_list::render(cx);
    let empty = snippets::attachments_empty::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "A flexible, composable attachment surface for files, images, videos, audio, and source documents.",
        ),
        vec![
            DocSection::new("Usage with AI SDK", usage)
                .description("Rust/Fret analogue of the official AI Elements usage pattern.")
                .test_id_prefix("ui-gallery-ai-attachments-usage")
                .code_rust_from_file_region(snippets::attachments_usage::SOURCE, "example"),
            DocSection::new("Grid Variant", grid)
                .description("Best for message attachments with thumbnail-style previews.")
                .test_id_prefix("ui-gallery-ai-attachments-grid")
                .code_rust_from_file_region(snippets::attachments_grid::SOURCE, "example"),
            DocSection::new("Inline Variant", inline)
                .description("Compact badges with hover previews and remove affordances.")
                .test_id_prefix("ui-gallery-ai-attachments-inline")
                .code_rust_from_file_region(snippets::attachments_inline::SOURCE, "example"),
            DocSection::new("List Variant", list)
                .description("Row layout for filenames and metadata.")
                .test_id_prefix("ui-gallery-ai-attachments-list")
                .code_rust_from_file_region(snippets::attachments_list::SOURCE, "example"),
            DocSection::new("Empty State", empty)
                .description("Fallback content when there are no attachments.")
                .test_id_prefix("ui-gallery-ai-attachments-empty")
                .code_rust_from_file_region(snippets::attachments_empty::SOURCE, "example"),
        ],
    );

    vec![body]
}
