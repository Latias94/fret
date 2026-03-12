use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::data_table as snippets;

pub(super) fn preview_data_table(
    cx: &mut UiCx<'_>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    let default_demo = snippets::default_demo::render(cx);
    let basic_demo = snippets::basic_demo::render(cx);
    let demo = snippets::guide_demo::render(cx, state);
    let rtl_demo = snippets::rtl_demo::render(cx);
    let code_preview = snippets::code_outline::render(cx);

    let api_reference = doc_layout::notes(
        cx,
        [
            "Data Table in shadcn is a guide recipe (TanStack Table + table primitives), not a single fixed widget; treat this page as a living parity surface.",
            "Default recipe here means: explicit TableState + TableViewOutput + one toolbar + one footer, without pretending business-table state can be hidden.",
            "Everything below Default Recipe should be read as advanced reference material, not as the baseline authoring path.",
            "Prefer small, explicit recipe surfaces (toolbar/pagination/column header) that can be reused by apps and gated by diag scripts.",
            "Ownership: recipe owns chrome/row heights/selection/pagination/column-action menus; callers own data shape, filtering rules, and page width/height negotiation.",
            "`DataGrid` remains the canvas-first option for dense editor surfaces; use `experimental::DataGridElement` when you need richer per-cell UI than the guide-style table surface.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "shadcn Data Table is a guide recipe (TanStack + Table primitives). This page starts with a curated default recipe, then keeps denser business-table variants as advanced reference material backed by Fret's headless engine.",
        ),
        vec![
            DocSection::new("Default Recipe", default_demo)
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::default_demo::SOURCE, "example"),
            DocSection::new("Advanced Reference", basic_demo)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-data-table-basic-table")
                .description("Minimal payments table aligned with the guide's first reusable table extraction.")
                .code_rust_from_file_region(snippets::basic_demo::SOURCE, "example"),
            DocSection::new("Advanced Guide", demo)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-data-table-guide-demo")
                .description("Integrated sorting, selection, actions, and pagination recipe backed by Fret's headless table state.")
                .code_rust_from_file_region(snippets::guide_demo::SOURCE, "example"),
            DocSection::new("Advanced RTL", rtl_demo)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-data-table-rtl")
                .description("Guide-aligned data table layout under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl_demo::SOURCE, "example"),
            DocSection::new("Reference Outline", code_preview)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-data-table-guide-outline")
                .description("Compact map of the reusable pieces that correspond to the upstream guide chapters.")
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
