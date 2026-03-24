use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_plan_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::plan_demo::render(cx);
    let notes = doc_layout::notes_block([
        "The preview now mirrors the official AI Elements docs composition more closely: direct `Plan` children instead of a controller-only authoring seam.",
        "`PlanTrigger` now stamps collapsible `expanded` + `controls` semantics, and `PlanContent` uses the shared measured-height collapsible motion path instead of abrupt hide/show behavior.",
    ]);
    let usage_section = DocSection::build(cx, "Usage", demo)
        .description("Rust/Fret analogue of the official AI Elements Plan preview.")
        .description(
            "Matches the upstream direct-children composition while keeping stable `test_id` hooks for diagnostics.",
        )
        .test_id_prefix("ui-gallery-ai-plan-demo")
        .code_rust_from_file_region(snippets::plan_demo::SOURCE, "example");
    let notes_section =
        DocSection::build(cx, "Notes", notes).description("Layering and parity notes for Plan.");

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned Plan coverage for AI Elements: collapsible execution plans with streaming shimmer on title and description.",
        ),
        vec![usage_section, notes_section],
        cx,
    );

    vec![body.into_element(cx)]
}
