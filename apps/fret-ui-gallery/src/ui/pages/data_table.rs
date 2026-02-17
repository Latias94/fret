use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    let legacy_content = super::super::preview_data_table_legacy(cx, state);
    let demo = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| legacy_content,
    )
    .test_id("ui-gallery-data-table-guide-demo");

    let notes_stack = doc_layout::notes(
        cx,
        [
            "Data Table in shadcn is a guide recipe, not a single fixed widget; treat this page as a living parity surface.",
            "Prefer small, explicit recipe surfaces (toolbar/pagination/column header) that can be reused by apps and gated by diag scripts.",
            "When extending this page, prefer deterministic state rows and stable test IDs so diag scripts can gate regressions.",
            "Future refactor can split column/header/view-options into reusable subcomponents mirroring upstream guide chapters.",
        ],
    );

    let code_preview = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![shadcn::typography::muted(
                cx,
                "Key snippets for the guide-aligned recipe surface.",
            )]
        },
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "shadcn Data Table is a guide recipe (TanStack + Table primitives). This page renders a guide-aligned demo backed by Fret's headless engine.",
        ),
        vec![
            DocSection::new("Guide Demo", demo).max_w(Px(900.0)).code(
                "rust",
                r#"// This page currently reuses the legacy recipe surface while the guide-aligned
// split (toolbar/pagination/columns) is stabilized.
let content = super::super::preview_data_table_legacy(cx, state.clone());

stack::vstack(
    cx,
    stack::VStackProps::default()
        .gap(Space::N3)
        .items_start()
        .layout(LayoutRefinement::default().w_full()),
    |_cx| content,
)
.into_element(cx);"#,
            ),
            DocSection::new("Code", code_preview)
                .max_w(Px(900.0))
                .test_id_prefix("ui-gallery-data-table-code")
                .code(
                    "rust",
                    r#"// Basic Table
let table = shadcn::DataTable::new()
    .row_height(Px(36.0))
    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(280.0)))
    .into_element(cx, data, 1, state, columns, row_key, col_key, render_cell, render_header);

// State + Sorting
let selected_count = models.read(&state, |st| st.row_selection.len())?;
let sorting = models.read(&state, |st| st.sorting.first().cloned())?;

// show selection/sorting summaries in a deterministic status row

// Docs Gap Markers
section_card("Filtering", Alert::new([...]))
section_card("Visibility", Alert::new([...]))
// keep unsupported guide sections explicit so parity work is traceable"#,
                ),
            DocSection::new("Notes", notes_stack).max_w(Px(900.0)),
        ],
    );

    vec![body.test_id("ui-gallery-data-table-component")]
}
