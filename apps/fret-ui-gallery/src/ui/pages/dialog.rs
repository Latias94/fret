use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::dialog as snippets;

pub(super) fn preview_dialog(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let parts = snippets::parts::render(cx);
    let custom_close = snippets::custom_close_button::render(cx);
    let no_close = snippets::no_close_button::render(cx);
    let sticky_footer = snippets::sticky_footer::render(cx);
    let scrollable_content = snippets::scrollable_content::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes_block([
        "Preview follows upstream shadcn Dialog docs order first: Demo, Usage, Custom Close Button, No Close Button, Sticky Footer, Scrollable Content, RTL; the `Parts` adapter section follows afterwards.",
        "`Dialog::compose()` is a recipe-level bridge for shadcn-style part composition without pushing children API concerns into the mechanism layer.",
        "`Usage` is the default copyable path; `Parts` stays as the advanced adapter section for explicit `DialogTrigger` / `DialogPortal` / `DialogOverlay` ownership.",
        "Part surface adapters exist for shadcn-style call sites (DialogTrigger/DialogPortal/DialogOverlay).",
        "Default close affordance now lives in `DialogContent`, matching upstream; disable it with `show_close_button(false)`.",
        "`DialogClose::from_scope()` remains available when a page wants an additional or fully custom close affordance inside dialog content.",
        "Scrollable examples isolate long content in ScrollArea so footer/header placement remains predictable under constrained viewport sizes.",
        "Each scenario has stable test IDs to support fretboard diag scripts and regression screenshots.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes)
        .description(
            "Keep test IDs stable so fretboard diag scripts and regression screenshots remain reusable.",
        )
        .test_id_prefix("ui-gallery-dialog-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Basic dialog with header, form fields, and footer actions.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable shadcn-style composition reference for Dialog.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let custom_close = DocSection::build(cx, "Custom Close Button", custom_close)
        .description("Replace the close affordance with a custom footer action.")
        .code_rust_from_file_region(snippets::custom_close_button::SOURCE, "example");
    let no_close = DocSection::build(cx, "No Close Button", no_close)
        .description("Hide the default close button and rely on Escape or overlay dismissal.")
        .code_rust_from_file_region(snippets::no_close_button::SOURCE, "example");
    let sticky_footer = DocSection::build(cx, "Sticky Footer", sticky_footer)
        .description("Footer stays visible while the content scrolls.")
        .code_rust_from_file_region(snippets::sticky_footer::SOURCE, "example");
    let scrollable_content = DocSection::build(cx, "Scrollable Content", scrollable_content)
        .description("Long body scrolls while keeping the header visible.")
        .code_rust_from_file_region(snippets::scrollable_content::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Dialog layout should work under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let parts = DocSection::build(cx, "Parts", parts)
        .description(
            "Advanced part surface adapters for explicit Trigger/Portal/Overlay ownership.",
        )
        .code_rust_from_file_region(snippets::parts::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Dialog docs order first, with the advanced Fret-specific `Parts` adapter section appended afterwards.",
        ),
        vec![
            demo,
            usage,
            custom_close,
            no_close,
            sticky_footer,
            scrollable_content,
            rtl,
            parts,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-dialog").into_element(cx)]
}
