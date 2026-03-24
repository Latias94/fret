use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

pub(super) fn preview_ai_agent_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::agent_demo::render(cx);
    let features = doc_layout::notes_block([
        "Root border, header chrome, spacing, and schema disclosure outcomes already align with the upstream AI Elements `Agent` component.",
        "Tool schemas render through shadcn accordion semantics, so the expandable disclosure behavior stays in the component/policy layer instead of `crates/fret-ui`.",
        "The copyable Fret example now follows the same compound structure as the official docs: `Agent -> AgentHeader + AgentContent -> AgentInstructions / AgentTools / AgentOutput`.",
        "Existing eager constructors and the raw accordion-based `AgentTools::multiple_uncontrolled(...)` lane remain available as escape hatches when a caller needs lower-level control.",
    ])
    .test_id("ui-gallery-ai-agent-features");
    let props = agent_parts_props_table(cx).test_id("ui-gallery-ai-agent-props");
    let notes = doc_layout::notes_block([
        "Mechanism looks healthy here: the remaining drift was not in `fret-ui`, but in the component-layer authoring surface and the UI Gallery docs page coverage.",
        "`Agent`, `AgentContent`, and `AgentTools` now expose a docs-shaped `empty().children(...)` lane, so the Rust example maps more directly to the official JSX composition without pretending this is a DOM port.",
        "We intentionally keep this surface in `fret-ui-ai`: `agent` is UI chrome, not a runtime contract, and it does not need provider-owned closure state to match the upstream outcome.",
        "The tool-definition seam stays explicit in Rust. Upstream AI SDK `Tool` maps to `AgentToolDefinition` so callers can pass JSON Schema data without hiding serialization details behind framework magic.",
    ])
    .test_id("ui-gallery-ai-agent-notes");

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned Agent coverage for AI Elements: the same compound composition story as the official page, adapted to Fret's explicit Rust builder surface and JSON-schema tool definitions.",
        ),
        vec![
            DocSection::build(cx, "Usage with AI SDK-style Tool Definitions", demo)
                .description(
                    "Rust/Fret analogue of the official AI Elements Agent usage example.",
                )
                .description(
                    "The snippet uses the docs-shaped children lane for `Agent`, `AgentContent`, and `AgentTools`, while keeping tool schema data explicit and copyable.",
                )
                .test_id_prefix("ui-gallery-ai-agent-demo")
                .code_rust_from_file_region(snippets::agent_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("Behavior and default-value outcomes preserved while aligning with the official Agent docs surface.")
                .no_shell(),
            DocSection::build(cx, "Builder Surface", props)
                .description("Current Fret builder surface for the `Agent` family, including the new docs-shaped children lane and the accordion escape hatch.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Parity findings and layering decision for Agent."),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}

fn agent_parts_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "Agent",
                "empty().children(parts) / new(children)",
                "builder / IntoIterator<Item = AnyElement>",
                "w_full + min_w_0",
                "Root chrome aligned with upstream `<Agent>`; `empty().children(...)` mirrors the official compound structure, while `new(children)` stays as the eager shorthand.",
            ],
            [
                "Agent",
                "test_id / refine_layout / refine_style",
                "builder methods",
                "None / default layout / default chrome",
                "Diagnostics selector plus explicit layout and chrome overrides.",
            ],
            [
                "AgentHeader",
                "new(name) + model(model)",
                "impl Into<Arc<str>>",
                "model = None",
                "Header row with bot icon, title, and optional model badge.",
            ],
            [
                "AgentContent",
                "empty().children(parts) / new(children)",
                "builder / IntoIterator<Item = AnyElement>",
                "w_full + min_w_0",
                "Content wrapper aligned with upstream `<AgentContent>` spacing and section stacking.",
            ],
            [
                "AgentInstructions",
                "new(text)",
                "impl Into<Arc<str>>",
                "-",
                "Instruction text block; upstream string `children` maps to an explicit text payload in Rust.",
            ],
            [
                "AgentTools",
                "empty().children(tools)",
                "builder / IntoIterator<Item = AgentTool>",
                "multiple accordion, closed",
                "Docs-shaped compound lane mirroring `<AgentTools><AgentTool ... /></AgentTools>`.",
            ],
            [
                "AgentTools",
                "multiple_uncontrolled(items)",
                "IntoIterator<Item = AccordionItem>",
                "-",
                "Escape hatch when the caller wants to supply raw shadcn accordion items directly.",
            ],
            [
                "AgentTool",
                "new(value, definition) + trigger_test_id(...)",
                "AgentToolDefinition",
                "description fallback = \"No description\"",
                "Single tool disclosure row; schema comes from `json_schema` or `input_schema`.",
            ],
            [
                "AgentOutput",
                "new(schema)",
                "impl Into<Arc<str>>",
                "-",
                "Structured output schema block with TypeScript syntax highlighting.",
            ],
        ],
        true,
    )
}
