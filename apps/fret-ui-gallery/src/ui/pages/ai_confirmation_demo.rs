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
        "Compound slot composition now matches the upstream AI Elements preview scripts, including `ConfirmationTitle` wrapping the request/accepted/rejected slots.",
        "Keep the request state actionable and keep the accepted/rejected states read-only so the page mirrors the official preview scripts and docs examples.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Preview keeps the live approval workflow while mirroring the official AI Elements preview scripts, including title-wrapped confirmation slots and state-specific actions.",
        ),
        vec![
            DocSection::build(cx, "Workflow Demo", workflow)
                .test_id_prefix("ui-gallery-ai-confirmation-demo")
                .code_rust_from_file_region(snippets::confirmation_demo::SOURCE, "example"),
            DocSection::build(cx, "Approval Request State", request)
                .description("Preview-script-aligned approval-requested state with action buttons.")
                .test_id_prefix("ui-gallery-ai-confirmation-request")
                .code_rust_from_file_region(snippets::confirmation_request::SOURCE, "example"),
            DocSection::build(cx, "Approved State", accepted)
                .description(
                    "Preview-script-aligned approval-responded state with accepted feedback.",
                )
                .test_id_prefix("ui-gallery-ai-confirmation-accepted")
                .code_rust_from_file_region(snippets::confirmation_accepted::SOURCE, "example"),
            DocSection::build(cx, "Rejected State", rejected)
                .description("Preview-script-aligned output-denied state with rejected feedback.")
                .test_id_prefix("ui-gallery-ai-confirmation-rejected")
                .code_rust_from_file_region(snippets::confirmation_rejected::SOURCE, "example"),
            DocSection::build(cx, "Notes", notes)
                .description("Layering and parity notes for Confirmation."),
        ],
    );

    vec![
        body.test_id("ui-gallery-page-ai-confirmation-demo")
            .into_element(cx),
    ]
}
