use super::super::*;

pub(super) fn preview_popover(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};
    use crate::ui::snippets::popover as snippets;

    let demo = snippets::demo::render(cx);
    let basic = snippets::basic::render(cx);
    let align = snippets::align::render(cx);
    let with_form = snippets::with_form::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Popover demo (new-york-v4).",
            "Keep content width explicit (e.g. 320px) for predictable layout.",
            "For dense input rows, prefer `Field`/`FieldGroup` recipes to keep spacing consistent with other form surfaces.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Popover demo: Dimensions form (w=320px)."),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-popover-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .test_id_prefix("ui-gallery-popover-basic")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Align", align)
                .test_id_prefix("ui-gallery-popover-align")
                .code_rust_from_file_region(snippets::align::SOURCE, "example"),
            DocSection::new("With Form", with_form)
                .test_id_prefix("ui-gallery-popover-with-form")
                .code_rust_from_file_region(snippets::with_form::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-popover-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes),
        ],
    );

    vec![body.test_id("ui-gallery-popover")]
}
