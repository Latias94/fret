use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_confirmation_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let workflow = snippets::confirmation_demo::render(cx);
    let request = snippets::confirmation_request::render(cx);
    let accepted = snippets::confirmation_accepted::render(cx);
    let rejected = snippets::confirmation_rejected::render(cx);

    let notes = doc_layout::notes_block([
        "Confirmation still lives in the AI Elements/component layer: it composes Alert + Button outcomes rather than changing fret-ui runtime contracts.",
        "The workflow section now mirrors the official docs teaching surface more closely: direct `Confirmation` children for the end-to-end flow, then focused state sections for the preview-script variants.",
        "The focused request/accepted/rejected examples keep the upstream preview-script shape where `ConfirmationTitle` wraps the state-specific slots.",
        "`ConfirmationAction` now supports custom visible children while preserving action-first authoring and the base label for semantics.",
    ]);
    let workflow_section = DocSection::build(cx, "Usage with Tool Approval Workflow", workflow)
        .description(
            "Rust/Fret analogue of the official AI Elements Confirmation usage example, including the final output state after approval or rejection.",
        )
        .description(
            "Uses direct `Confirmation` children like the docs surface while keeping stable `test_id` hooks for diagnostics.",
        )
        .test_id_prefix("ui-gallery-ai-confirmation-demo")
        .code_rust_from_file_region(snippets::confirmation_demo::SOURCE, "example");
    let request_section = DocSection::build(cx, "Approval Request State", request)
        .description(
            "Preview-script-aligned `approval-requested` state using the optional `ConfirmationTitle` wrapper around slot content.",
        )
        .test_id_prefix("ui-gallery-ai-confirmation-request")
        .code_rust_from_file_region(snippets::confirmation_request::SOURCE, "example");
    let accepted_section = DocSection::build(cx, "Approved State", accepted)
        .description(
            "Preview-script-aligned accepted state. The same accepted slot composition also applies when the tool reaches `output-available`.",
        )
        .test_id_prefix("ui-gallery-ai-confirmation-accepted")
        .code_rust_from_file_region(snippets::confirmation_accepted::SOURCE, "example");
    let rejected_section = DocSection::build(cx, "Rejected State", rejected)
        .description(
            "Preview-script-aligned `output-denied` state with rejected feedback and no remaining action row.",
        )
        .test_id_prefix("ui-gallery-ai-confirmation-rejected")
        .code_rust_from_file_region(snippets::confirmation_rejected::SOURCE, "example");
    let notes_section = DocSection::build(cx, "Notes", notes)
        .description("Layering and parity notes for Confirmation.");

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned Confirmation coverage for AI Elements: a complete approval workflow first, then focused state examples that track the official preview scripts.",
        ),
        vec![
            workflow_section,
            request_section,
            accepted_section,
            rejected_section,
            notes_section,
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
