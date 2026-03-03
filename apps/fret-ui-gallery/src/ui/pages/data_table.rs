use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::data_table as snippets;

pub(super) fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    let basic_demo = snippets::basic_demo::render(cx);
    let demo = snippets::guide_demo::render(cx, state);
    let rtl_demo = snippets::rtl_demo::render(cx);

    let notes_stack = doc_layout::notes(
        cx,
        [
            "Data Table in shadcn is a guide recipe, not a single fixed widget; treat this page as a living parity surface.",
            "Prefer small, explicit recipe surfaces (toolbar/pagination/column header) that can be reused by apps and gated by diag scripts.",
            "When extending this page, prefer deterministic state rows and stable test IDs so diag scripts can gate regressions.",
            "Future refactor can split column/header/view-options into reusable subcomponents mirroring upstream guide chapters.",
        ],
    );

    let code_preview = snippets::code_outline::render(cx);

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "shadcn Data Table is a guide recipe (TanStack + Table primitives). This page renders a guide-aligned demo backed by Fret's headless engine.",
        ),
        vec![
            DocSection::new("Basic Demo", basic_demo)
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::basic_demo::SOURCE, "example"),
            DocSection::new("Guide Demo", demo)
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::guide_demo::SOURCE, "example"),
            DocSection::new("RTL", rtl_demo)
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::rtl_demo::SOURCE, "example"),
            DocSection::new("Code", code_preview)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-data-table-code")
                .code_rust_from_file_region(snippets::code_outline::SOURCE, "example"),
            DocSection::new("Notes", notes_stack).max_w(Px(980.0)),
        ],
    );

    vec![body.test_id("ui-gallery-data-table-component")]
}
