use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret_ui_shadcn as shadcn;

pub(super) fn preview_ai_chain_of_thought_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let usage = snippets::chain_of_thought_demo::render(cx);
    let composable = snippets::chain_of_thought_composable::render(cx);

    let features = doc_layout::notes(
        cx,
        [
            "Collapsible reasoning trace aligned with the official AI Elements ChainOfThought example.",
            "Controlled / uncontrolled state parity via `open_model(...)` and `default_open(...)`.",
            "Docs-style compound composition via `ChainOfThought::header(...).content(...)` or `children([...])`, so the Gallery code stays close to the official examples.",
            "Rich slot composition for header copy, step labels, and step descriptions via child elements.",
            "Search results and image captions reuse the same shadcn badge + muted caption treatment as upstream.",
        ],
    )
    .test_id("ui-gallery-ai-chain-of-thought-features");

    let props = chain_of_thought_props_table(cx).test_id("ui-gallery-ai-chain-of-thought-props");

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the official AI Elements Chain of Thought example, then documents the Fret-specific composition model used to keep compound parts safe in a move-only declarative tree.",
        ),
        vec![
            DocSection::new("Usage", usage)
                .description("Official-example-aligned usage with the same search-result and image steps.")
                .test_id_prefix("ui-gallery-ai-chain-of-thought-demo")
                .code_rust_from_file_region(snippets::chain_of_thought_demo::SOURCE, "example"),
            DocSection::new("Composable Slots", composable)
                .description("Demonstrates custom header content plus rich step label / description children.")
                .test_id_prefix("ui-gallery-ai-chain-of-thought-composable")
                .code_rust_from_file_region(
                    snippets::chain_of_thought_composable::SOURCE,
                    "example",
                ),
            DocSection::new("Features", features)
                .description("Behavior and composition notes mapped from the official docs."),
            DocSection::new("Props", props)
                .description("Fret API surface for `fret_ui_ai::ChainOfThought*` builders."),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-chain-of-thought-demo")]
}

fn chain_of_thought_props_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let row = |cx: &mut ElementContext<'_, App>,
               part: &'static str,
               method: &'static str,
               ty: &'static str,
               default: &'static str,
               desc: &'static str| {
        shadcn::TableRow::build(5, move |cx, out| {
            out.push_ui(cx, shadcn::TableCell::build(ui::text(part)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(method)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(ty)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(default)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(desc)));
        })
        .border_bottom(true)
    };

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(5, |cx, out| {
                        out.push(shadcn::TableHead::new("Part").into_element(cx));
                        out.push(shadcn::TableHead::new("Method").into_element(cx));
                        out.push(shadcn::TableHead::new("Type").into_element(cx));
                        out.push(shadcn::TableHead::new("Default").into_element(cx));
                        out.push(shadcn::TableHead::new("Description").into_element(cx));
                    })
                    .border_bottom(true)
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push_ui(cx, row(cx, "ChainOfThought", "open_model", "Model<bool>", "None", "Controlled open state."));
                out.push_ui(cx, row(cx, "ChainOfThought", "default_open", "bool", "false", "Initial open state for uncontrolled usage."));
                out.push_ui(cx, row(cx, "ChainOfThought", "header / content", "builder methods", "-", "Docs-style chained composition for the two compound parts used by the official examples."));
                out.push_ui(cx, row(cx, "ChainOfThought", "children", "IntoIterator<Item = Header | Content>", "-", "Lower-level typed child list when you prefer building the compound parts as a batch."));
                out.push_ui(cx, row(cx, "ChainOfThought", "into_element_with_children", "FnOnce(&mut ElementContext) -> Vec<AnyElement>", "-", "Lower-level escape hatch when child construction must happen inside a live scope."));
                out.push_ui(cx, row(cx, "ChainOfThought", "test_id_root / gap / refine_layout", "builder methods", "w_full + gap 4", "Root diagnostics id, vertical spacing, and layout refinement."));
                out.push_ui(cx, row(cx, "ChainOfThoughtHeader", "children", "IntoIterator<Item = AnyElement>", "\"Chain of Thought\"", "Overrides the default header label with composed children."));
                out.push_ui(cx, row(cx, "ChainOfThoughtContent", "new(children)", "IntoIterator<Item = AnyElement>", "-", "Wraps step content in the collapsible body."));
                out.push_ui(cx, row(cx, "ChainOfThoughtStep", "new(label)", "impl Into<Arc<str>>", "status = complete, icon = dot", "Creates a step with upstream-aligned defaults."));
                out.push_ui(cx, row(cx, "ChainOfThoughtStep", "label_children / description_children", "IntoIterator<Item = AnyElement>", "None", "Rich slot APIs for custom label and description content."));
                out.push_ui(cx, row(cx, "ChainOfThoughtStep", "status / icon / children", "builder methods", "complete / dot / empty", "Visual status, leading icon, and trailing custom body content."));
                out.push_ui(cx, row(cx, "ChainOfThoughtSearchResults", "new(children)", "IntoIterator<Item = AnyElement>", "gap 2 + wrap", "Wraps badges like the official search result row."));
                out.push_ui(cx, row(cx, "ChainOfThoughtSearchResult", "new(label)", "impl Into<Arc<str>>", "secondary badge", "Badge-shaped search result pill with normal label weight."));
                out.push_ui(cx, row(cx, "ChainOfThoughtImage", "new(children) / caption", "builder methods", "caption = None", "Muted rounded image frame with optional caption text."));
            }),
        );
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
