use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{AppComponentCx, UiChild};

fn tool_state_mapping_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["ToolStatus", "Badge", "Typical surface", "Notes"],
        [
            [
                "InputStreaming",
                "Pending",
                "Input is still streaming",
                "Matches the official `input-streaming` example and stays collapsed by default.",
            ],
            [
                "InputAvailable",
                "Running",
                "Tool is executing with parameters available",
                "Matches the official `input-available` example.",
            ],
            [
                "OutputAvailable",
                "Completed",
                "Successful result is ready",
                "Gallery opens this state by default so the docs-shaped result lane is visible immediately.",
            ],
            [
                "OutputError",
                "Error",
                "Execution failed",
                "Matches the official `output-error` example and surfaces destructive chrome.",
            ],
            [
                "ApprovalRequested / ApprovalResponded / OutputDenied",
                "Awaiting Approval / Responded / Denied",
                "Transcript and approval flows",
                "Supported in `fret-ui-ai` even though this page keeps the main preview focused on the four official Tool docs states.",
            ],
        ],
        true,
    )
}

fn tool_props_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "Tool",
                "root()",
                "builder root",
                "-",
                "Docs-shaped compound root aligned with upstream `<Tool>...</Tool>` composition.",
            ],
            [
                "Tool",
                "new(header, content)",
                "ToolHeader + ToolContent",
                "-",
                "Legacy explicit assembly lane. Still valid when both parts are already built.",
            ],
            [
                "Tool",
                "children([...])",
                "ToolChild",
                "-",
                "Typed child list for `ToolHeader` + `ToolContent`; closest Fret analogue of nested JSX children.",
            ],
            [
                "Tool",
                "header / content",
                "builder methods",
                "-",
                "Part-specific sugar when you prefer a chained builder over an explicit child list.",
            ],
            [
                "Tool",
                "default_open / open_model",
                "bool / Model<bool>",
                "false / None",
                "Uncontrolled or controlled open state mapped to the underlying shadcn/Radix collapsible policy surface.",
            ],
            [
                "ToolHeader",
                "new(name, status)",
                "impl Into<Arc<str>> + ToolStatus",
                "-",
                "Builds the trigger row; labels derived from `tool-*` names still strip the prefix by default.",
            ],
            [
                "ToolContent",
                "new(children)",
                "IntoIterator<Item = AnyElement>",
                "-",
                "Collapsible body container with docs-aligned spacing and padding.",
            ],
            [
                "ToolInput",
                "new(input)",
                "ToolCallPayload",
                "-",
                "Structured parameters lane rendered as JSON-ish code.",
            ],
            [
                "ToolOutput",
                "new(output, error_text)",
                "Option<ToolCallPayload> + Option<Arc<str>>",
                "-",
                "Model-driven payload lane used by transcript/tool-call plumbing.",
            ],
            [
                "ToolOutput",
                "custom(children)",
                "IntoIterator<Item = AnyElement>",
                "-",
                "Rich output lane for docs-shaped custom result rendering such as `MessageResponse` or a custom code block.",
            ],
        ],
        true,
    )
}

pub(super) fn preview_ai_tool_demo(cx: &mut AppComponentCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let usage = snippets::tool_demo::render(cx);
    let state_mapping = tool_state_mapping_table(cx).test_id("ui-gallery-ai-tool-demo-states");
    let props = tool_props_table(cx).test_id("ui-gallery-ai-tool-demo-props");
    let notes = doc_layout::notes_block([
        "Mechanism health looks fine here: the existing tool toggle/screenshot diag scripts already prove the collapsible interaction path, so the primary drift was not in `crates/fret-ui`.",
        "The real mismatch was public-surface and docs-surface parity. Upstream teaches nested children under `<Tool>`, while Fret previously only exposed `Tool::new(header, content)` in the first-party example.",
        "This page now teaches the docs-shaped compound lane first via `Tool::root().children([...])`, while `Tool::new(...)` remains the explicit assembly seam for lower-level call sites such as transcript plumbing.",
        "`ToolOutput::custom([...])` now covers the official rich-output lane, so docs-shaped examples can render `MessageResponse`-style results without dropping down to raw containers.",
        "The AI Tool page is gated behind `gallery-dev`, which is also required for the wider `fret-ui-ai` gallery surface.",
    ])
    .test_id("ui-gallery-ai-tool-demo-notes");

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned Tool coverage for AI Elements: collapsible tool-call chrome, status mapping, and the public authoring surface that sits above shadcn/Radix collapsible primitives.",
        ),
        vec![
            DocSection::build(cx, "Usage", usage)
                .description(
                    "Copyable Rust/Fret analogue of the official AI Elements Tool docs, using compound children composition and a rich output lane.",
                )
                .test_id_prefix("ui-gallery-ai-tool-demo")
                .code_rust_from_file_region(snippets::tool_demo::SOURCE, "example"),
            DocSection::build(cx, "State Mapping", state_mapping)
                .description("How `fret-ui-ai` maps tool lifecycle states onto the docs-facing badge and disclosure chrome.")
                .no_shell(),
            DocSection::build(cx, "Builder Surface", props)
                .description("Current Fret API surface for `Tool`, including the docs-shaped compound root and the rich output lane.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Parity findings, layering decision, and why this pass stayed in `fret-ui-ai` + UI Gallery instead of the runtime mechanism layer.")
                .no_shell(),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
