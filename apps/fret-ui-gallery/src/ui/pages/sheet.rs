use super::super::*;

pub(super) fn preview_sheet(
    cx: &mut ElementContext<'_, App>,
    _open: Model<bool>,
) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};
    use crate::ui::snippets::sheet as snippets;

    let demo = snippets::demo::render(cx);
    let parts = snippets::parts::render(cx);
    let side = snippets::side::render(cx);
    let no_close_button = snippets::no_close_button::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Sheet demo (new-york-v4).",
            "Part surface adapters exist for shadcn-style call sites (SheetTrigger/SheetPortal/SheetOverlay).",
            "Fret renders a default top-right close affordance in `SheetContent` (disable via `show_close_button(false)`).",
            "Fret also exposes `SheetClose` for additional close affordances.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Extends dialog to display side-aligned panels for supplementary tasks."),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Parts", parts)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-parts")
                .code_rust_from_file_region(snippets::parts::SOURCE, "example"),
            DocSection::new("Side", side)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-side")
                .code_rust_from_file_region(snippets::side::SOURCE, "example"),
            DocSection::new("No Close Button", no_close_button)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-no-close")
                .code_rust_from_file_region(snippets::no_close_button::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-sheet-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-sheet")]
}
