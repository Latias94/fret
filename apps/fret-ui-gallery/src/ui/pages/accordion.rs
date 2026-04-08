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

    let api_reference = doc_layout::notes_block([
        "`accordion_single_uncontrolled(cx, default, |cx| ..)` and `accordion_multiple_uncontrolled(cx, default, |cx| ..)` stay the terse builder helpers for the upstream Demo/Basic/Multiple lanes.",
        "`AccordionRoot::children([...])` plus `AccordionItemPart`, `AccordionTriggerPart`, and `AccordionContentPart` now provide the curated typed children lane on the facade, so the copyable `Usage` section no longer needs the raw `shadcn::raw::accordion::composable` escape hatch.",
        "Measured-height motion, roving focus, and the trigger/content accessibility relationships remain primitive-owned; caller-owned width, card shells, and bordered wrappers stay explicit page-level composition.",
        "Radix and Base UI remain the semantics truth for trigger-expanded state, region labelling, and roving navigation; the remaining accordion drift here is first-party docs/public-surface alignment rather than a `fret-ui` mechanism bug.",
        "A broader untyped JSX-style heterogeneous children API is not currently warranted beyond the typed root lane: accordion still only needs explicit item/trigger/content ownership, and widening further would add more naming ambiguity than capability.",
        "The raw composable module still exists for source-alignment work, but first-party docs should prefer the curated facade aliases unless they are intentionally documenting a lower-level seam.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-accordion-api-reference")
        .description("Public surface summary and ownership notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Official shadcn `AccordionDemo` structure.")
        .test_id_prefix("ui-gallery-accordion-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable typed children usage on the shadcn facade via `AccordionRoot::children([...])` and the part aliases.")
        .test_id_prefix("ui-gallery-accordion-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("Basic single-open accordion with the first item expanded by default.")
        .test_id_prefix("ui-gallery-accordion-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let multiple = DocSection::build(cx, "Multiple", multiple)
        .description("Use `accordion_multiple_uncontrolled` when multiple items may stay open.")
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
            "Preview mirrors the shadcn Accordion docs path first: Demo, Usage, Basic, Multiple, Disabled, Borders, Card, RTL, and API Reference. The usage lane keeps the typed `AccordionRoot::children([...])` surface copyable on the curated facade, while the builder helpers cover the compact docs examples.",
        ),
        vec![
            demo,
            usage,
            basic,
            multiple,
            disabled,
            borders,
            card,
            rtl,
            api_reference,
        ],
    );

    vec![body.test_id("ui-gallery-accordion").into_element(cx)]
}
