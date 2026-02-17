use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    let legacy_content = super::super::preview_data_table_legacy(cx, state);
    let guide_demo = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |_cx| legacy_content,
    )
    .test_id("ui-gallery-data-table-guide-demo");

    let notes = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "API reference: `ecosystem/fret-ui-shadcn/src/data_table.rs` (recipes) and `ecosystem/fret-ui-shadcn/src/table.rs` (primitives).",
                ),
                shadcn::typography::muted(
                    cx,
                    "Data Table in shadcn is a guide recipe, not a single fixed widget; treat this page as a living parity surface.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Prefer small, explicit recipe surfaces (toolbar/pagination/column header) that can be reused by apps and gated by diag scripts.",
                ),
                shadcn::typography::muted(
                    cx,
                    "When extending this page, keep stable test IDs so diag scripts can gate regressions.",
                ),
            ]
        },
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "shadcn Data Table is a guide recipe (TanStack + Table primitives). This page renders a guide-aligned demo backed by Fret's headless engine.",
        ),
        vec![
            DocSection::new("Guide Demo", guide_demo)
                .description("Guide-aligned demo (selection, sorting, row actions) backed by the headless engine.")
                .code(
                    "rust",
                    r#"let table = shadcn::DataTable::new()
    .row_height(Px(36.0))
    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(280.0)))
    .into_element(cx, data, 1, state, columns, row_key, col_key, render_cell, render_header);"#,
                )
                .max_w(Px(900.0)),
            DocSection::new("Notes", notes)
                .description("API reference pointers and authoring notes.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-data-table-component")]
}
