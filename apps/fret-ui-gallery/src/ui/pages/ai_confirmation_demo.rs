use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_confirmation_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let workflow = snippets::confirmation_demo::render(cx);
    let request = snippets::confirmation_request::render(cx);
    let accepted = snippets::confirmation_accepted::render(cx);
    let rejected = snippets::confirmation_rejected::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Confirmation belongs in the AI Elements/component layer: it composes Alert + Button semantics rather than changing fret-ui runtime behavior.",
            "Prefer rendering the slot children from the Confirmation root so Request / Accepted / Rejected / Actions can consume the nearest approval context.",
            "Keep the request state actionable and keep the accepted/rejected states read-only so the page mirrors the official docs examples.",
        ],
    );

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Preview keeps the live approval workflow while adding the three docs-aligned AI Elements Confirmation states for copyable examples.",
        ),
        vec![
            DocSection::new("Workflow Demo", workflow)
                .test_id_prefix("ui-gallery-ai-confirmation-demo")
                .code_rust_from_file_region(snippets::confirmation_demo::SOURCE, "example"),
            DocSection::new("Approval Request State", request)
                .description("Docs-aligned approval-requested state with action buttons.")
                .test_id_prefix("ui-gallery-ai-confirmation-request")
                .code_rust_from_file_region(snippets::confirmation_request::SOURCE, "example"),
            DocSection::new("Approved State", accepted)
                .description("Docs-aligned approval-responded state with accepted feedback.")
                .test_id_prefix("ui-gallery-ai-confirmation-accepted")
                .code_rust_from_file_region(snippets::confirmation_accepted::SOURCE, "example"),
            DocSection::new("Rejected State", rejected)
                .description("Docs-aligned output-denied state with rejected feedback.")
                .test_id_prefix("ui-gallery-ai-confirmation-rejected")
                .code_rust_from_file_region(snippets::confirmation_rejected::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .description("Layering and parity notes for Confirmation."),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-confirmation-demo")]
}
