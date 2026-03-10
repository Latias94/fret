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
    let code_preview = snippets::code_outline::render(cx);

    let api_reference = doc_layout::notes(
        cx,
        [
            "shadcn Data Table is a guide recipe over TanStack Table plus table-related recipes, not a tiny `registry:ui` leaf component.",
            "`fret_ui_shadcn::DataTable` is the integrated business-table surface, while headless state lives in `fret_ui_headless::table::TableState` and companion recipes stay reusable.",
            "Table chrome, row heights, selection affordances, pagination controls, and column-action menus remain recipe-owned; app-specific columns, data shape, filtering rules, and page width/height negotiation remain caller-owned.",
            "`DataGrid` remains the canvas-first option for dense editor surfaces; use `experimental::DataGridElement` when you need richer per-cell UI than the guide-style table surface.",
            "This pass is docs/public-surface parity for an extension surface, not a mechanism-layer fix.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "shadcn Data Table is a multi-chapter guide rather than a single leaf component. This page keeps the guide compression explicit with `Basic Table`, `Guide Demo`, `RTL`, `Guide Outline`, and `API Reference`.",
        ),
        vec![
            DocSection::new("Basic Table", basic_demo)
                .description("Minimal payments table aligned with the guide's first reusable table extraction.")
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-data-table-basic-table")
                .code_rust_from_file_region(snippets::basic_demo::SOURCE, "example"),
            DocSection::new("Guide Demo", demo)
                .description("Integrated sorting, selection, actions, and pagination recipe backed by Fret's headless table state.")
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-data-table-guide-demo")
                .code_rust_from_file_region(snippets::guide_demo::SOURCE, "example"),
            DocSection::new("RTL", rtl_demo)
                .description("Guide-aligned data table layout under an RTL direction provider.")
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-data-table-rtl")
                .code_rust_from_file_region(snippets::rtl_demo::SOURCE, "example"),
            DocSection::new("Guide Outline", code_preview)
                .description("Compact map of the reusable pieces that correspond to the upstream guide chapters.")
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-data-table-guide-outline")
                .code_rust_from_file_region(snippets::code_outline::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .description("Extension-surface summary and ownership notes.")
                .no_shell()
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-data-table-api-reference"),
        ],
    );

    vec![body.test_id("ui-gallery-data-table-component")]
}
