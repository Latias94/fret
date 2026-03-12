use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::accordion as snippets;

pub(super) fn preview_accordion(
    cx: &mut UiCx<'_>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let _ = value;

    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let multiple = snippets::multiple::render(cx);
    let disabled = snippets::disabled::render(cx);
    let borders = snippets::borders::render(cx);
    let card = snippets::card::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Gallery sections mirror shadcn Accordion docs directly: Demo, Usage, Basic, Multiple, Disabled, Borders, Card, RTL.",
            "API reference: `ecosystem/fret-ui-shadcn/src/accordion.rs`.",
            "The legacy builder-style API remains available as a compact Fret shorthand, but the docs `Usage` section now prefers the composable Radix-shaped surface for parity.",
            "Measured-height motion and roving focus behavior are framework-owned; these gallery sections focus on the public authoring surface and recipe outcomes.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "A vertically stacked set of interactive headings that each reveal a section of content.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Upstream shadcn AccordionDemo structure.")
                .test_id_prefix("ui-gallery-accordion-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Minimal usage mirroring the upstream docs example.")
                .test_id_prefix("ui-gallery-accordion-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("A basic accordion that shows one item at a time.")
                .test_id_prefix("ui-gallery-accordion-basic")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Multiple", multiple)
                .description("Use the `multiple` mode to allow multiple items to stay open.")
                .test_id_prefix("ui-gallery-accordion-multiple")
                .code_rust_from_file_region(snippets::multiple::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Disable individual items with `AccordionItem::disabled(true)`.")
                .test_id_prefix("ui-gallery-accordion-disabled")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Borders", borders)
                .description(
                    "Wrap the accordion in an outer bordered container for the bordered recipe.",
                )
                .test_id_prefix("ui-gallery-accordion-borders")
                .code_rust_from_file_region(snippets::borders::SOURCE, "example"),
            DocSection::new("Card", card)
                .description("Wrap the accordion in a card surface.")
                .test_id_prefix("ui-gallery-accordion-card")
                .code_rust_from_file_region(snippets::card::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Accordion layout should work under an RTL direction provider.")
                .test_id_prefix("ui-gallery-accordion-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .test_id_prefix("ui-gallery-accordion-notes")
                .description("Parity notes and references."),
        ],
    );

    vec![body.test_id("ui-gallery-accordion")]
}
