use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::skeleton as snippets;

pub(super) fn preview_skeleton(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let avatar = snippets::avatar::render(cx);
    let card = snippets::card::render(cx);
    let text_section = snippets::text::render(cx);
    let form = snippets::form::render(cx);
    let table = snippets::table::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Use Skeleton for loading placeholders, not empty states.",
            "Prefer consistent sizes and spacing so content doesn't jump when loaded.",
            "Keep semantics grouped so screen readers can skip placeholder-only regions.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Skeleton docs examples."),
        vec![
            DocSection::new("Demo", demo)
                .description("Avatar row with two text lines.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Avatar", avatar)
                .description("Smaller avatar placeholder.")
                .code_rust_from_file_region(snippets::avatar::SOURCE, "example"),
            DocSection::new("Card", card)
                .description("Skeletons inside a card layout.")
                .code_rust_from_file_region(snippets::card::SOURCE, "example"),
            DocSection::new("Text", text_section)
                .description("Multiple lines with varying widths.")
                .code_rust_from_file_region(snippets::text::SOURCE, "example"),
            DocSection::new("Form", form)
                .description("Form-like blocks.")
                .code_rust_from_file_region(snippets::form::SOURCE, "example"),
            DocSection::new("Table", table)
                .description("Row skeletons.")
                .code_rust_from_file_region(snippets::table::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Direction provider should not break layout.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-skeleton")]
}
