use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

fn parts_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Surface", "Notes"],
        [
            [
                "Attachments",
                "Container owns the shared `variant` and layout defaults; grid keeps the upstream fit-width + wrap feel while list stays full-width.",
            ],
            [
                "Attachment",
                "Item root supports direct defaults or the compound lane via `into_element_with_children(...)` when you want to reorder or replace parts.",
            ],
            [
                "AttachmentPreview",
                "Media slot renders the provided preview handle and falls back to category icons; inline/list/grid sizing stays variant-aware.",
            ],
            [
                "AttachmentInfo",
                "Filename or source title plus optional media type row for inline/list variants.",
            ],
            [
                "AttachmentRemove",
                "Remove affordance can be built directly or inherited from context via `from_context()` so custom compound layouts do not have to re-thread ids or callbacks.",
            ],
            [
                "AttachmentHoverCard*",
                "Inline preview wrappers now live in `fret_ui_ai`, so the first-party example can stay on the same teaching surface instead of dropping to generic shadcn hover-card types.",
            ],
            [
                "AttachmentEmpty",
                "Empty-state shell shows a default label but still accepts custom children.",
            ],
            [
                "Utility functions",
                "`get_media_category(...)` and `get_attachment_label(...)` mirror the upstream helper story for app-side branching and labeling.",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_attachments_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let usage = snippets::attachments_usage::render(cx);
    let grid = snippets::attachments_grid::render(cx);
    let inline = snippets::attachments_inline::render(cx);
    let list = snippets::attachments_list::render(cx);
    let empty = snippets::attachments_empty::render(cx);
    let features = doc_layout::notes_block([
        "Three docs-aligned variants are covered here: grid thumbnails, inline chips, and list rows.",
        "Compound composition is already available in Rust via `Attachment::into_element_with_children(...)` plus `AttachmentPreview/Info/Remove::from_context()`.",
        "Inline hover previews stay on the `fret_ui_ai` surface through `AttachmentHoverCard`, `AttachmentHoverCardTrigger`, and `AttachmentHoverCardContent`.",
        "Snippet code is now self-contained for copy/paste: preview media comes from deterministic in-memory RGBA sources instead of UI Gallery-only asset glue.",
        "When apps only have an image URL, `AttachmentPreview` now routes that through Fret's capability-gated asset loading path; docs snippets still stay deterministic by using in-memory previews.",
    ])
    .test_id("ui-gallery-ai-attachments-features");
    let parts = parts_table(cx).test_id("ui-gallery-ai-attachments-parts");
    let notes = doc_layout::notes_block([
        "Audit result: the core runtime/mechanism layer looked healthy; the main drift was in component/public-surface teaching rather than `crates/fret-ui` contracts.",
        "Remote image URLs now work as a capability-gated preview input, while explicit `ImageId` previews still stay the most deterministic path for first-party docs and native demos.",
        "The main remaining transport gap versus AI Elements is richer non-image remote previewing: upstream can lean on DOM `<video>`/browser media, while Fret still falls back to category icons for video/audio in this self-drawn surface.",
        "Existing diag coverage is already in place for grid remove flow and screenshot capture, so this pass focuses on authoring-surface correctness rather than new runtime gates.",
    ])
    .test_id("ui-gallery-ai-attachments-notes");

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned Attachments coverage for AI Elements: complete usage first, then focused grid/inline/list examples, composable parts, and the remaining preview-transport gap.",
        ),
        vec![
            DocSection::build(cx, "Usage with AI SDK", usage)
                .description("Rust/Fret analogue of the official AI Elements usage pattern, using the compound lane so the code tab stays close to the upstream mental model.")
                .test_id_prefix("ui-gallery-ai-attachments-usage")
                .code_rust_from_file_region(snippets::attachments_usage::SOURCE, "example"),
            DocSection::build(cx, "Grid Variant", grid)
                .description("Best for message attachments with thumbnail-style previews and hover-remove affordances.")
                .test_id_prefix("ui-gallery-ai-attachments-grid")
                .code_rust_from_file_region(snippets::attachments_grid::SOURCE, "example"),
            DocSection::build(cx, "Inline Variant", inline)
                .description("Compact badges with hover previews and remove affordances on the `fret_ui_ai` hover-card surface.")
                .test_id_prefix("ui-gallery-ai-attachments-inline")
                .code_rust_from_file_region(snippets::attachments_inline::SOURCE, "example"),
            DocSection::build(cx, "List Variant", list)
                .description("Row layout for filenames and metadata, including an image-preview example without gallery-only helpers.")
                .test_id_prefix("ui-gallery-ai-attachments-list")
                .code_rust_from_file_region(snippets::attachments_list::SOURCE, "example"),
            DocSection::build(cx, "Empty State", empty)
                .description("Fallback content when there are no attachments.")
                .test_id_prefix("ui-gallery-ai-attachments-empty")
                .code_rust_from_file_region(snippets::attachments_empty::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("High-signal parity notes against the official AI Elements Attachments page.")
                .no_shell(),
            DocSection::build(cx, "Parts & Props", parts)
                .description("Which surface owns layout, composition, and preview behavior in the Rust API.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Parity findings and layering decision for Attachments."),
        ],
    );

    vec![body.into_element(cx)]
}
