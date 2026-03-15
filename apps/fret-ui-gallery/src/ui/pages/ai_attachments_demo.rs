use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_attachments_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let usage = snippets::attachments_usage::render(cx);
    let grid = snippets::attachments_grid::render(cx);
    let inline = snippets::attachments_inline::render(cx);
    let list = snippets::attachments_list::render(cx);
    let empty = snippets::attachments_empty::render(cx);
    let notes = doc_layout::notes_block([
        "Mechanism looked healthy in this audit: the drift was in the component layer and the UI Gallery page shape, not in `fret-ui` runtime contracts.",
        "`Attachment` already supports composable parts via `into_element_with_children(...)` plus `AttachmentPreview/Info/Remove::from_context()`, which is the Rust equivalent of the upstream compound-children API.",
        "`AttachmentHoverCard*` now lives in `fret-ui-ai`, so the inline example can mirror the official AI Elements docs without reaching directly into generic shadcn hover-card wrappers.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned Attachments examples covering the same grid / inline / list composition model as the official AI Elements page.",
        ),
        vec![
            DocSection::build(cx, "Usage with AI SDK", usage)
                .description("Rust/Fret analogue of the official AI Elements usage pattern.")
                .test_id_prefix("ui-gallery-ai-attachments-usage")
                .code_rust_from_file_region(snippets::attachments_usage::SOURCE, "example"),
            DocSection::build(cx, "Grid Variant", grid)
                .description("Best for message attachments with thumbnail-style previews.")
                .test_id_prefix("ui-gallery-ai-attachments-grid")
                .code_rust_from_file_region(snippets::attachments_grid::SOURCE, "example"),
            DocSection::build(cx, "Inline Variant", inline)
                .description("Compact badges with hover previews and remove affordances.")
                .test_id_prefix("ui-gallery-ai-attachments-inline")
                .code_rust_from_file_region(snippets::attachments_inline::SOURCE, "example"),
            DocSection::build(cx, "List Variant", list)
                .description("Row layout for filenames and metadata.")
                .test_id_prefix("ui-gallery-ai-attachments-list")
                .code_rust_from_file_region(snippets::attachments_list::SOURCE, "example"),
            DocSection::build(cx, "Empty State", empty)
                .description("Fallback content when there are no attachments.")
                .test_id_prefix("ui-gallery-ai-attachments-empty")
                .code_rust_from_file_region(snippets::attachments_empty::SOURCE, "example"),
            DocSection::build(cx, "Notes", notes)
                .description("Parity findings and layering decision for Attachments."),
        ],
    );

    vec![
        body.test_id("ui-gallery-page-ai-attachments-demo")
            .into_element(cx),
    ]
}
