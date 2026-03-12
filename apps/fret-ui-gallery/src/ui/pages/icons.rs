use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::icons as snippets;

pub(super) fn preview_icons(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let grid = snippets::grid::render(cx);
    let spinner_row = snippets::spinner::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Prefer stable icon IDs (e.g. `lucide.search`) so demos remain predictable across updates.",
            "Icon size should be explicit in docs to avoid token drift.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Sample icons and spinners used across the gallery."),
        vec![
            DocSection::new("Icons", grid)
                .max_w(Px(980.0))
                .description("Icons rendered via `fret_icons` IDs.")
                .code_rust_from_file_region(snippets::grid::SOURCE, "example"),
            DocSection::new("Spinner", spinner_row)
                .description("Spinner can be animated or static.")
                .code_rust_from_file_region(snippets::spinner::SOURCE, "example"),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-icons")]
}
