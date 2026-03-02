use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::typography as snippets;

pub(super) fn preview_typography(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
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
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/demo.rs"),
                    "example",
                ),
            DocSection::new("h1", h1)
                .description("Top-level heading.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/h1.rs"),
                    "example",
                ),
            DocSection::new("h2", h2)
                .description("Section heading.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/h2.rs"),
                    "example",
                ),
            DocSection::new("h3", h3)
                .description("Sub-section heading.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/h3.rs"),
                    "example",
                ),
            DocSection::new("h4", h4)
                .description("Low-level heading for grouped content.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/h4.rs"),
                    "example",
                ),
            DocSection::new("p", p)
                .description("Body paragraph text.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(include_str!("../snippets/typography/p.rs"), "example"),
            DocSection::new("blockquote", blockquote)
                .description("Quoted callout text.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/blockquote.rs"),
                    "example",
                ),
            DocSection::new("table", table)
                .description("Tabular content using shadcn Table parts.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/table.rs"),
                    "example",
                ),
            DocSection::new("list", list)
                .description("Bulleted/ordered list content.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/list.rs"),
                    "example",
                ),
            DocSection::new("Inline Code", inline_code)
                .description("Inline code styling for commands and identifiers.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/inline_code.rs"),
                    "example",
                ),
            DocSection::new("Lead", lead)
                .description("Intro lead paragraph for sections.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/lead.rs"),
                    "example",
                ),
            DocSection::new("Large", large)
                .description("Emphasis text for short callouts.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/large.rs"),
                    "example",
                ),
            DocSection::new("Small", small)
                .description("Helper text and metadata.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/small.rs"),
                    "example",
                ),
            DocSection::new("Muted", muted)
                .description("De-emphasized hint/explanation text.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/muted.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .description("Direction provider sample to validate RTL wrapping/alignment.")
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/typography/rtl.rs"),
                    "example",
                ),
            DocSection::new("Notes", notes)
                .description("API reference pointers and authoring notes.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-typography")]
}
