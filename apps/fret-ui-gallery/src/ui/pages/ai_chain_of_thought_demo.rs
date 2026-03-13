use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

pub(super) fn preview_ai_chain_of_thought_demo(
    cx: &mut UiCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let usage = snippets::chain_of_thought_demo::render(cx);
    let composable = snippets::chain_of_thought_composable::render(cx);

    let features = doc_layout::notes_block([
            "Collapsible reasoning trace aligned with the official AI Elements ChainOfThought example.",
            "Controlled / uncontrolled state parity via `open_model(...)` and `default_open(...)`.",
            "Docs-style compound composition via `ChainOfThought::header(...).content(...)` or `children([...])`, so the Gallery code stays close to the official examples.",
            "Rich slot composition for header copy, step labels, and step descriptions via child elements.",
            "Search results and image captions reuse the same shadcn badge + muted caption treatment as upstream.",
        ])
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
            DocSection::build(cx, "Features", features)
                .description("Behavior and composition notes mapped from the official docs."),
            DocSection::build(cx, "Props", props)
                .description("Fret API surface for `fret_ui_ai::ChainOfThought*` builders."),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-chain-of-thought-demo")]
}

fn chain_of_thought_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "ChainOfThought",
                "open_model",
                "Model<bool>",
                "None",
                "Controlled open state.",
            ],
            [
                "ChainOfThought",
                "default_open",
                "bool",
                "false",
                "Initial open state for uncontrolled usage.",
            ],
            [
                "ChainOfThought",
                "header / content",
                "builder methods",
                "-",
                "Docs-style chained composition for the two compound parts used by the official examples.",
            ],
            [
                "ChainOfThought",
                "children",
                "IntoIterator<Item = Header | Content>",
                "-",
                "Lower-level typed child list when you prefer building the compound parts as a batch.",
            ],
            [
                "ChainOfThought",
                "into_element_with_children",
                "FnOnce(&mut ElementContext) -> Vec<AnyElement>",
                "-",
                "Lower-level escape hatch when child construction must happen inside a live scope.",
            ],
            [
                "ChainOfThought",
                "test_id_root / gap / refine_layout",
                "builder methods",
                "w_full + gap 4",
                "Root diagnostics id, vertical spacing, and layout refinement.",
            ],
            [
                "ChainOfThoughtHeader",
                "children",
                "IntoIterator<Item = AnyElement>",
                "\"Chain of Thought\"",
                "Overrides the default header label with composed children.",
            ],
            [
                "ChainOfThoughtContent",
                "new(children)",
                "IntoIterator<Item = AnyElement>",
                "-",
                "Wraps step content in the collapsible body.",
            ],
            [
                "ChainOfThoughtStep",
                "new(label)",
                "impl Into<Arc<str>>",
                "status = complete, icon = dot",
                "Creates a step with upstream-aligned defaults.",
            ],
            [
                "ChainOfThoughtStep",
                "label_children / description_children",
                "IntoIterator<Item = AnyElement>",
                "None",
                "Rich slot APIs for custom label and description content.",
            ],
            [
                "ChainOfThoughtStep",
                "status / icon / children",
                "builder methods",
                "complete / dot / empty",
                "Visual status, leading icon, and trailing custom body content.",
            ],
            [
                "ChainOfThoughtSearchResults",
                "new(children)",
                "IntoIterator<Item = AnyElement>",
                "gap 2 + wrap",
                "Wraps badges like the official search result row.",
            ],
            [
                "ChainOfThoughtSearchResult",
                "new(label)",
                "impl Into<Arc<str>>",
                "secondary badge",
                "Badge-shaped search result pill with normal label weight.",
            ],
            [
                "ChainOfThoughtImage",
                "new(children) / caption",
                "builder methods",
                "caption = None",
                "Muted rounded image frame with optional caption text.",
            ],
        ],
        true,
    )
}
