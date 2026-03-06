use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;

fn parts_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let header = shadcn::TableHeader::new([shadcn::TableRow::new(
        2,
        [
            shadcn::TableHead::new("Part").into_element(cx),
            shadcn::TableHead::new("Notes").into_element(cx),
        ],
    )
    .into_element(cx)])
    .into_element(cx);

    let body = shadcn::TableBody::new([
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("Context")).into_element(cx),
                shadcn::TableCell::new(cx.text(
                    "Root owns used/max tokens, optional usage costs, hover delays, and the provider scope.",
                ))
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("ContextTrigger")).into_element(cx),
                shadcn::TableCell::new(cx.text(
                    "Default trigger reads provider state; use `.children(...)` for a custom trigger surface.",
                ))
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("ContextContent")).into_element(cx),
                shadcn::TableCell::new(cx.text(
                    "Hover card content shell with divider, min width, and overflow defaults matching the docs card.",
                ))
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("Header / Body / Footer")).into_element(cx),
                shadcn::TableCell::new(cx.text(
                    "Each part supports custom children; footer remains app-owned for cost values today.",
                ))
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("Usage rows")).into_element(cx),
                shadcn::TableCell::new(cx.text(
                    "Input / Output / Reasoning / Cache rows auto-hide on zero tokens and accept custom children.",
                ))
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("Compound composition")).into_element(cx),
                shadcn::TableCell::new(cx.text(
                    "Use `into_element_with_children(...)` when parts need the nearest provider, which is the Rust equivalent of the upstream compound-children API.",
                ))
                .into_element(cx),
            ],
        )
        .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Table::new([header, body]).into_element(cx)
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
            "Compound component structure matches the official AI Elements docs: trigger, content shell, header, body, footer, and per-usage rows.",
            "Compact token formatting now follows the upstream style more closely (`100K` instead of `100.0K`).",
            "Gallery examples intentionally keep all four usage rows non-zero so every part stays visible and copyable in one page visit.",
            "Known `model_id` aliases now estimate costs automatically; explicit `ContextUsage::*_cost_usd` values still win when apps have exact billing data.",
            "The current Fret surface already supports composable children; the main difference is Rust uses `into_element_with_children(...)` instead of React JSX nesting.",
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
            "Preview keeps the official AI Elements compound example first, then shows Fret's default convenience API for the same surface.",
        ),
        vec![
            DocSection::new("Compound API", compound)
                .description("Docs-aligned provider-scoped composition using `ContextTrigger`, `ContextContent`, and the usage parts.")
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
