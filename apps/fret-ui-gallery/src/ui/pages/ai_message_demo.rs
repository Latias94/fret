use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_message_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let usage = snippets::message_usage::render(cx);
    let demo = snippets::message_demo::render(cx);
    let notes = doc_layout::notes_block([
        "The underlying behavior looks healthy: alignment, width constraints, and markdown rendering all live in `fret-ui-ai` recipe/policy code rather than `crates/fret-ui` mechanisms.",
        "The main upstream drift is API ergonomics, not rendering correctness: Fret currently passes `MessageRole` to both `Message` and `MessageContent` because styling is resolved eagerly in a self-rendering tree instead of via DOM parent selectors.",
        "Usage examples in Fret intentionally keep user messages on the plain-text path and reserve `MessageResponse` for assistant markdown, matching the current `fret-ui-ai` content model.",
        "This page now mirrors the official AI Elements docs shape more closely: a full usage example first, then focused surface demos. Branching remains available on the dedicated Message Branch page when `gallery-dev` is enabled.",
    ]);

    let mut sections = vec![
        DocSection::build(cx, "Usage with Conversation + PromptInput", usage)
            .description("Rust/Fret analogue of the official AI Elements Message usage example.")
            .test_id_prefix("ui-gallery-ai-message-usage")
            .code_rust_from_file_region(snippets::message_usage::SOURCE, "example"),
        DocSection::build(cx, "Core Surface", demo)
            .description("Focused alignment + bubble + actions + markdown response surface.")
            .test_id_prefix("ui-gallery-ai-message-demo")
            .code_rust_from_file_region(snippets::message_demo::SOURCE, "example"),
    ];

    #[cfg(feature = "gallery-dev")]
    {
        let branch = snippets::message_branch_demo::render(cx);
        sections.push(
            DocSection::build(cx, "Branching", branch)
                .description("Message branching is part of the upstream Message suite and stays available as a dedicated demo as well.")
                .test_id_prefix("ui-gallery-ai-message-branch-inline")
                .code_rust_from_file_region(snippets::message_branch_demo::SOURCE, "example"),
        );
    }

    sections.push(
        DocSection::build(cx, "Notes", notes)
            .description("Parity findings and current API trade-offs for Message."),
    );

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned Message suite coverage for AI Elements: complete usage flow first, then focused surface examples and parity notes.",
        ),
        sections,
    );

    vec![body.into_element(cx)]
}
