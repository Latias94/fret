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

    let notes = doc_layout::notes_block([
        "Gallery sections mirror shadcn Accordion docs directly: Demo, Usage, Basic, Multiple, Disabled, Borders, Card, RTL.",
        "API reference: `ecosystem/fret-ui-shadcn/src/accordion.rs`.",
        "`accordion_single_uncontrolled(cx, default, |cx| ..)` and `accordion_multiple_uncontrolled(cx, default, |cx| ..)` are the default first-party helpers; the `Usage` section keeps the composable Radix-shaped surface as the explicit advanced seam.",
        "Measured-height motion and roving focus behavior are framework-owned; these gallery sections focus on the public authoring surface and recipe outcomes.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes)
        .test_id_prefix("ui-gallery-accordion-notes")
        .description("Parity notes and references.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Upstream shadcn AccordionDemo structure.")
        .test_id_prefix("ui-gallery-accordion-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Composable Radix-shaped usage for parity-heavy or slot-level composition.")
        .test_id_prefix("ui-gallery-accordion-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("A basic accordion using the builder-preserving single-open helper.")
        .test_id_prefix("ui-gallery-accordion-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let multiple = DocSection::build(cx, "Multiple", multiple)
        .description(
            "Use the builder-preserving multi-open helper when several items may stay open.",
        )
        .test_id_prefix("ui-gallery-accordion-multiple")
        .code_rust_from_file_region(snippets::multiple::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled", disabled)
        .description("Disable individual items with `AccordionItem::disabled(true)`.")
        .test_id_prefix("ui-gallery-accordion-disabled")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let borders = DocSection::build(cx, "Borders", borders)
        .description("Wrap the accordion in an outer bordered container for the bordered recipe.")
        .test_id_prefix("ui-gallery-accordion-borders")
        .code_rust_from_file_region(snippets::borders::SOURCE, "example");
    let card = DocSection::build(cx, "Card", card)
        .description("Wrap the accordion in a card surface.")
        .test_id_prefix("ui-gallery-accordion-card")
        .code_rust_from_file_region(snippets::card::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Accordion layout should work under an RTL direction provider.")
        .test_id_prefix("ui-gallery-accordion-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "A vertically stacked set of interactive headings that each reveal a section of content.",
        ),
        vec![
            demo, usage, basic, multiple, disabled, borders, card, rtl, notes,
        ],
    );

    vec![body.test_id("ui-gallery-accordion")]
}
