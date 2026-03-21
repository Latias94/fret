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

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/typography.rs` and `ecosystem/fret-ui-shadcn/src/table.rs`.",
        "Heading helpers attach `SemanticsRole::Heading` with levels 1-4; keep the document hierarchy intentional.",
        "Typography remains a docs/helper surface. Do not widen it with a generic `children(...)` API until inline rich-text composition has a stable contract.",
        "Use `lead` for intros, `muted` for hints, and keep width/alignment decisions owned by the surrounding page.",
        "For long-form content, combine typography helpers with table/list blocks and validate RTL plus narrow-viewport wrapping before shipping.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes)
        .test_id_prefix("ui-gallery-typography-notes")
        .description("API reference pointers and authoring notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Full story sample following the upstream docs flow; the inline-link sentence stays plain text on the raw helper lane.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let h1 = DocSection::build(cx, "h1", h1)
        .description("Top-level heading example matching the upstream docs title.")
        .code_rust_from_file_region(snippets::h1::SOURCE, "example");
    let h2 = DocSection::build(cx, "h2", h2)
        .description("Second-level section heading.")
        .code_rust_from_file_region(snippets::h2::SOURCE, "example");
    let h3 = DocSection::build(cx, "h3", h3)
        .description("Third-level subsection heading.")
        .code_rust_from_file_region(snippets::h3::SOURCE, "example");
    let h4 = DocSection::build(cx, "h4", h4)
        .description("Fourth-level heading for grouped content.")
        .code_rust_from_file_region(snippets::h4::SOURCE, "example");
    let p = DocSection::build(cx, "p", p)
        .description("Body paragraph text.")
        .code_rust_from_file_region(snippets::p::SOURCE, "example");
    let blockquote = DocSection::build(cx, "blockquote", blockquote)
        .description("Quoted callout text.")
        .code_rust_from_file_region(snippets::blockquote::SOURCE, "example");
    let table = DocSection::build(cx, "table", table)
        .description("Tabular content using shadcn Table parts.")
        .code_rust_from_file_region(snippets::table::SOURCE, "example");
    let list = DocSection::build(cx, "list", list)
        .description("Bulleted/ordered list content.")
        .code_rust_from_file_region(snippets::list::SOURCE, "example");
    let inline_code = DocSection::build(cx, "Inline code", inline_code)
        .description("Inline code styling for commands and identifiers.")
        .code_rust_from_file_region(snippets::inline_code::SOURCE, "example");
    let lead = DocSection::build(cx, "Lead", lead)
        .description("Intro lead paragraph for sections.")
        .code_rust_from_file_region(snippets::lead::SOURCE, "example");
    let large = DocSection::build(cx, "Large", large)
        .description("Emphasis text for short callouts.")
        .code_rust_from_file_region(snippets::large::SOURCE, "example");
    let small = DocSection::build(cx, "Small", small)
        .description("Helper text and metadata.")
        .code_rust_from_file_region(snippets::small::SOURCE, "example");
    let muted = DocSection::build(cx, "Muted", muted)
        .description("De-emphasized hint/explanation text.")
        .code_rust_from_file_region(snippets::muted::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Direction-provider version of the full upstream typography story.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Typography page follows the shadcn docs order first; Notes remains the focused Fret follow-up after the upstream path.",
        ),
        vec![
            demo,
            h1,
            h2,
            h3,
            h4,
            p,
            blockquote,
            table,
            list,
            inline_code,
            lead,
            large,
            small,
            muted,
            rtl,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-typography").into_element(cx)]
}
