use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::icons as snippets;

pub(super) fn preview_icons(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let grid = snippets::grid::render(cx);
    let spinner_row = snippets::spinner::render(cx);

    let notes = doc_layout::notes_block([
        "Prefer stable icon IDs (e.g. `lucide.search`) so demos remain predictable across updates.",
        "Icon size should be explicit in docs to avoid token drift.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes).description("Usage notes.");
    let grid = DocSection::build(cx, "Icons", grid)
        .max_w(Px(980.0))
        .description("Icons rendered via `fret_icons` IDs.")
        .code_rust_from_file_region(snippets::grid::SOURCE, "example");
    let spinner_row = DocSection::build(cx, "Spinner", spinner_row)
        .description("Spinner can be animated or static.")
        .code_rust_from_file_region(snippets::spinner::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some("Sample icons and spinners used across the gallery."),
        vec![grid, spinner_row, notes],
    );

    vec![body.test_id("ui-gallery-icons").into_element(cx)]
}
