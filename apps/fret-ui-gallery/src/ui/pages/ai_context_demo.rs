use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

fn parts_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Notes"],
        [
            [
                "Context",
                "Root owns used/max tokens, optional usage costs, hover delays, and the provider scope.",
            ],
            [
                "ContextTrigger",
                "Default trigger reads provider state; use `.children(...)` for a custom trigger surface.",
            ],
            [
                "ContextContent",
                "Hover card content shell with divider, min width, and overflow defaults matching the docs card.",
            ],
            [
                "Header / Body / Footer",
                "Each part supports custom children; the default footer renders total cost and apps can still override it with custom children or explicit cost values.",
            ],
            [
                "Usage rows",
                "Input / Output / Reasoning / Cache rows auto-hide on zero tokens and accept custom children.",
            ],
            [
                "Compound composition",
                "`Context::children([...])` now mirrors the upstream compound ordering directly; `into_element_with_children(...)` remains the late-binding escape hatch.",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_context_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let compound = snippets::context_demo::render(cx);
    let default_api = snippets::context_default::render(cx);
    let features = doc_layout::notes_block([
        "Compound component structure now matches the official AI Elements docs more closely: `Context::children([...])` can directly nest trigger + content parts in the same order.",
        "Compact token formatting now follows the upstream style more closely (`100K` instead of `100.0K`).",
        "Gallery examples intentionally keep all four usage rows non-zero so every part stays visible and copyable in one page visit.",
        "Known `model_id` aliases now estimate costs automatically; explicit `ContextUsage::*_cost_usd` values still win when apps have exact billing data.",
        "Part-level `.children(...)` customization still works, and the root keeps `into_element_with_children(...)` for cases where you need to construct parts from the nearest provider scope.",
    ]);
    let notes = doc_layout::notes_block([
        "Layering check: this is a component/policy-layer AI element built on top of `HoverCard`, not a runtime mechanism change.",
        "Built-in auto pricing is best-effort for the common provider/model aliases used in gallery demos; unknown aliases still rely on app-supplied costs.",
        "Keep `ui-ai-context-demo-*` test IDs stable; `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-context-demo-hover.json` depends on them.",
    ]);
    let parts = parts_table(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Preview starts with the docs-style compound example, then shows Fret's default convenience API for the same surface.",
        ),
        vec![
            DocSection::build(cx, "Compound API", compound)
                .description("Docs-aligned direct composition using `Context::children([...])`, `ContextTrigger`, `ContextContent`, and the usage parts.")
                .test_id_prefix("ui-gallery-ai-context-demo")
                .code_rust_from_file_region(snippets::context_demo::SOURCE, "example"),
            DocSection::build(cx, "Default API", default_api)
                .description("Fret convenience API renders the default trigger/content without repeating the part tree.")
                .test_id_prefix("ui-gallery-ai-context-default")
                .code_rust_from_file_region(snippets::context_default::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("High-signal parity notes against the official docs.")
                .no_shell(),
            DocSection::build(cx, "Parts & Props", parts)
                .description("Which surface owns what, and where the Rust composition differs from JSX.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Layering + remaining parity gap notes.")
                .no_shell(),
        ],
    );

    vec![
        body.test_id("ui-gallery-page-ai-context-demo")
            .into_element(cx),
    ]
}
