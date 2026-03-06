use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::dialog as snippets;

pub(super) fn preview_dialog(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let parts = snippets::parts::render(cx);
    let custom_close = snippets::custom_close_button::render(cx);
    let no_close = snippets::no_close_button::render(cx);
    let sticky_footer = snippets::sticky_footer::render(cx);
    let scrollable_content = snippets::scrollable_content::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Docs parity uses the same section sequence as upstream: custom close, no close, sticky footer, scrollable content, then RTL.",
            "`Dialog::compose()` is a recipe-level bridge for shadcn-style part composition without pushing children API concerns into the mechanism layer.",
            "Part surface adapters exist for shadcn-style call sites (DialogTrigger/DialogPortal/DialogOverlay).",
            "Current Fret API models close controls explicitly with DialogClose; omitting it is equivalent to showCloseButton={false} in shadcn docs.",
            "Content-local examples now prefer `DialogClose::from_scope()` so the close affordance stays close to shadcn composition without threading the same open model into every close icon.",
            "Scrollable examples isolate long content in ScrollArea so footer/header placement remains predictable under constrained viewport sizes.",
            "Each scenario has stable test IDs to support fretboard diag scripts and regression screenshots.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Dialog docs order: Demo, Custom Close Button, No Close Button, Sticky Footer, Scrollable Content, RTL (plus Parts adapter).",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Basic dialog with header, form fields, and footer actions.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable shadcn-style composition reference for Dialog.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Parts", parts)
                .description("shadcn-style part surface adapters (Trigger/Portal/Overlay).")
                .code_rust_from_file_region(snippets::parts::SOURCE, "example"),
            DocSection::new("Custom Close Button", custom_close)
                .description("Replace the close affordance with a custom footer action.")
                .code_rust_from_file_region(snippets::custom_close_button::SOURCE, "example"),
            DocSection::new("No Close Button", no_close)
                .description("Omit explicit close controls and rely on Escape or overlay dismissal.")
                .code_rust_from_file_region(snippets::no_close_button::SOURCE, "example"),
            DocSection::new("Sticky Footer", sticky_footer)
                .description("Footer stays visible while the content scrolls.")
                .code_rust_from_file_region(snippets::sticky_footer::SOURCE, "example"),
            DocSection::new("Scrollable Content", scrollable_content)
                .description("Long body scrolls while keeping the header visible.")
                .code_rust_from_file_region(snippets::scrollable_content::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Dialog layout should work under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description(
                "Keep test IDs stable so fretboard diag scripts and regression screenshots remain reusable.",
            ),
        ],
    );

    vec![body]
}
