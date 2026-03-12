use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::typography as snippets;

pub(super) fn preview_typography(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let h1 = snippets::h1::render(cx);
    let h2 = snippets::h2::render(cx);
    let h3 = snippets::h3::render(cx);
    let h4 = snippets::h4::render(cx);
    let p = snippets::p::render(cx);
    let blockquote = snippets::blockquote::render(cx);
    let table = snippets::table::render(cx);
    let list = snippets::list::render(cx);
    let inline_code = snippets::inline_code::render(cx);
    let lead = snippets::lead::render(cx);
    let large = snippets::large::render(cx);
    let small = snippets::small::render(cx);
    let muted = snippets::muted::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/typography.rs` and `ecosystem/fret-ui-shadcn/src/table.rs`.",
            "Typography in shadcn is utility-driven; keep heading hierarchy semantic and consistent.",
            "Use `lead` for intros, `muted` for hints, and avoid overusing large text in dense panels.",
            "For long-form content, combine typography helpers with table/list blocks for readability.",
            "Validate RTL and narrow viewport wrapping before shipping document-like surfaces.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Typography page follows shadcn docs order and shows one focused sample per section."),
        vec![
            DocSection::new("Demo", demo)
                .description("A long-form story sample combining headings, paragraphs, and lists.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("h1", h1)
                .description("Top-level heading.")
                .code_rust_from_file_region(snippets::h1::SOURCE, "example"),
            DocSection::new("h2", h2)
                .description("Section heading.")
                .code_rust_from_file_region(snippets::h2::SOURCE, "example"),
            DocSection::new("h3", h3)
                .description("Sub-section heading.")
                .code_rust_from_file_region(snippets::h3::SOURCE, "example"),
            DocSection::new("h4", h4)
                .description("Low-level heading for grouped content.")
                .code_rust_from_file_region(snippets::h4::SOURCE, "example"),
            DocSection::new("p", p)
                .description("Body paragraph text.")
                .code_rust_from_file_region(snippets::p::SOURCE, "example"),
            DocSection::new("blockquote", blockquote)
                .description("Quoted callout text.")
                .code_rust_from_file_region(snippets::blockquote::SOURCE, "example"),
            DocSection::new("table", table)
                .description("Tabular content using shadcn Table parts.")
                .code_rust_from_file_region(snippets::table::SOURCE, "example"),
            DocSection::new("list", list)
                .description("Bulleted/ordered list content.")
                .code_rust_from_file_region(snippets::list::SOURCE, "example"),
            DocSection::new("Inline Code", inline_code)
                .description("Inline code styling for commands and identifiers.")
                .code_rust_from_file_region(snippets::inline_code::SOURCE, "example"),
            DocSection::new("Lead", lead)
                .description("Intro lead paragraph for sections.")
                .code_rust_from_file_region(snippets::lead::SOURCE, "example"),
            DocSection::new("Large", large)
                .description("Emphasis text for short callouts.")
                .code_rust_from_file_region(snippets::large::SOURCE, "example"),
            DocSection::new("Small", small)
                .description("Helper text and metadata.")
                .code_rust_from_file_region(snippets::small::SOURCE, "example"),
            DocSection::new("Muted", muted)
                .description("De-emphasized hint/explanation text.")
                .code_rust_from_file_region(snippets::muted::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Direction provider sample to validate RTL wrapping/alignment.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .test_id_prefix("ui-gallery-typography-notes")
                .description("API reference pointers and authoring notes."),
        ],
    );

    vec![body.test_id("ui-gallery-typography")]
}
