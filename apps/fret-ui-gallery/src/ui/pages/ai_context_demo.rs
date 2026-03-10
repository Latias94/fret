use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;

fn parts_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let row = |part: &'static str, notes: &'static str| {
        shadcn::TableRow::build(2, move |cx, out| {
            out.push_ui(cx, shadcn::TableCell::build(ui::text(part)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(notes)));
        })
    };

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(2, |cx, out| {
                        out.push(shadcn::TableHead::new("Part").into_element(cx));
                        out.push(shadcn::TableHead::new("Notes").into_element(cx));
                    })
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push_ui(cx, row("Context", "Root owns used/max tokens, optional usage costs, hover delays, and the provider scope."));
                out.push_ui(cx, row("ContextTrigger", "Default trigger reads provider state; use `.children(...)` for a custom trigger surface."));
                out.push_ui(cx, row("ContextContent", "Hover card content shell with divider, min width, and overflow defaults matching the docs card."));
                out.push_ui(cx, row("Header / Body / Footer", "Each part supports custom children; the default footer renders total cost and apps can still override it with custom children or explicit cost values."));
                out.push_ui(cx, row("Usage rows", "Input / Output / Reasoning / Cache rows auto-hide on zero tokens and accept custom children."));
                out.push_ui(cx, row("Compound composition", "`Context::children([...])` now mirrors the upstream compound ordering directly; `into_element_with_children(...)` remains the late-binding escape hatch."));
            }),
        );
    })
    .into_element(cx)
}

pub(super) fn preview_ai_context_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let compound = snippets::context_demo::render(cx);
    let default_api = snippets::context_default::render(cx);
    let features = doc_layout::notes(
        cx,
        [
            "Compound component structure now matches the official AI Elements docs more closely: `Context::children([...])` can directly nest trigger + content parts in the same order.",
            "Compact token formatting now follows the upstream style more closely (`100K` instead of `100.0K`).",
            "Gallery examples intentionally keep all four usage rows non-zero so every part stays visible and copyable in one page visit.",
            "Known `model_id` aliases now estimate costs automatically; explicit `ContextUsage::*_cost_usd` values still win when apps have exact billing data.",
            "Part-level `.children(...)` customization still works, and the root keeps `into_element_with_children(...)` for cases where you need to construct parts from the nearest provider scope.",
        ],
    );
    let notes = doc_layout::notes(
        cx,
        [
            "Layering check: this is a component/policy-layer AI element built on top of `HoverCard`, not a runtime mechanism change.",
            "Built-in auto pricing is best-effort for the common provider/model aliases used in gallery demos; unknown aliases still rely on app-supplied costs.",
            "Keep `ui-ai-context-demo-*` test IDs stable; `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-context-demo-hover.json` depends on them.",
        ],
    );
    let parts = parts_table(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Preview starts with the docs-style compound example, then shows Fret's default convenience API for the same surface.",
        ),
        vec![
            DocSection::new("Compound API", compound)
                .description("Docs-aligned direct composition using `Context::children([...])`, `ContextTrigger`, `ContextContent`, and the usage parts.")
                .test_id_prefix("ui-gallery-ai-context-demo")
                .code_rust_from_file_region(snippets::context_demo::SOURCE, "example"),
            DocSection::new("Default API", default_api)
                .description("Fret convenience API renders the default trigger/content without repeating the part tree.")
                .test_id_prefix("ui-gallery-ai-context-default")
                .code_rust_from_file_region(snippets::context_default::SOURCE, "example"),
            DocSection::new("Features", features)
                .description("High-signal parity notes against the official docs.")
                .no_shell(),
            DocSection::new("Parts & Props", parts)
                .description("Which surface owns what, and where the Rust composition differs from JSX.")
                .no_shell(),
            DocSection::new("Notes", notes)
                .description("Layering + remaining parity gap notes.")
                .no_shell(),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-context-demo")]
}
