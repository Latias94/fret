use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_agent_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::agent_demo::render(cx);
    let notes = doc_layout::notes_block([
        "Mechanism looks healthy here: the remaining drift was not in `fret-ui`, but in the component-layer API shape and the docs page copy.",
        "`AgentTools` now has a direct item-based constructor, so the Rust example maps more closely to the official `AgentTools -> AgentTool*` composition without forcing callers to build a raw shadcn accordion first.",
        "We intentionally keep this in `fret-ui-ai`: `agent` is UI chrome, not a runtime contract, and it does not need a provider-driven closure API to match the upstream outcome.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned Agent example mirroring the official AI Elements composition: model badge, instructions, expandable tool schemas, and output schema.",
        ),
        vec![
            DocSection::new("Usage", demo)
                .description(
                    "Copyable example aligned with the upstream Agent docs, adapted to Fret's JSON-schema tool definition surface.",
                )
                .test_id_prefix("ui-gallery-ai-agent-demo")
                .code_rust_from_file_region(snippets::agent_demo::SOURCE, "example"),
            DocSection::build(cx, "Notes", notes)
                .description("Parity findings and layering decision for Agent."),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-agent-demo")]
}
