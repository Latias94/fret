use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::data_table as snippets;

pub(super) fn preview_data_table(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let default_demo = snippets::default_demo::render(cx);
    let basic_demo = snippets::basic_demo::render(cx);
    let guide_demo = snippets::guide_demo::render(cx);
    let rtl_demo = snippets::rtl_demo::render(cx);
    let code_preview = snippets::code_outline::render(cx);

    let api_reference = doc_layout::notes_block([
        "Data Table in shadcn is a guide recipe (TanStack Table + table primitives), not a single fixed widget; treat this page as a living parity surface.",
        "Default Recipe (Fret) here means: explicit TableState + TableViewOutput + one toolbar + one footer, without pretending business-table state can be hidden.",
        "Everything after Default Recipe (Fret) should be read as guide-aligned reference material, not as the baseline authoring path.",
        "Prefer small, explicit recipe surfaces (toolbar/pagination/column header) that can be reused by apps and gated by diag scripts.",
        "Ownership: recipe owns chrome/row heights/selection/pagination/column-action menus; callers own data shape, filtering rules, and page width/height negotiation.",
        "Selection-column examples now keep typed `.action(...)` / `.action_payload(...)` on the checkbox surface and grouped `cx.actions().models::<A>(...)` / `payload_models::<A>(...)` at the view layer instead of routing through root command ids.",
        "`DataGrid` remains the canvas-first option for dense editor surfaces; use `experimental::DataGridElement` when you need richer per-cell UI than the guide-style table surface.",
    ]);
    let notes = doc_layout::notes_block([
        "This page intentionally starts with a curated Fret default recipe, then mirrors the shadcn guide with Basic Table, Guide Demo, RTL, and Guide Outline follow-ups.",
        "Prefer Default Recipe (Fret) when you want a copyable business-table baseline; reach for the guide-aligned sections when you need explicit sorting, selection, actions, or pagination composition.",
        "Layout and diagnostics regressions here usually come from viewport ownership, column sizing, or row chrome, so keep page-scoped `ui-gallery-data-table-*` ids stable when extending the page.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .description("Extension-surface summary and ownership notes.")
        .no_shell()
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-data-table-api-reference");
    let notes = DocSection::build(cx, "Notes", notes)
        .description("Usage guidance and parity notes.")
        .no_shell()
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-data-table-notes");
    let default_recipe = DocSection::build(cx, "Default Recipe (Fret)", default_demo)
        .description(
            "Curated business-table golden path for this repo: explicit state/output plus one toolbar and one footer.",
        )
        .max_w(Px(980.0))
        .code_rust_from_file_region(snippets::default_demo::SOURCE, "example");
    let basic_table = DocSection::build(cx, "Basic Table", basic_demo)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-data-table-basic-table")
        .description(
            "Minimal payments table aligned with the guide's first reusable table extraction.",
        )
        .code_rust_from_file_region(snippets::basic_demo::SOURCE, "example");
    let guide_demo = DocSection::build(cx, "Guide Demo", guide_demo)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-data-table-guide-demo")
        .description(
            "Integrated sorting, filtering, visibility, row selection, row actions, and pagination recipe backed by Fret's headless table state.",
        )
        .code_rust_from_file_region(snippets::guide_demo::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl_demo)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-data-table-rtl")
        .description("Guide-aligned data table layout under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl_demo::SOURCE, "example");
    let guide_outline = DocSection::build(cx, "Guide Outline", code_preview)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-data-table-guide-outline")
        .description(
            "Compact map of the reusable pieces that correspond to the upstream guide chapters.",
        )
        .code_rust_from_file_region(snippets::code_outline::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "shadcn Data Table is a guide recipe (TanStack + Table primitives). This page starts with a curated Fret default recipe, then follows with compressed guide-aligned sections backed by Fret's headless engine.",
        ),
        vec![
            default_recipe,
            basic_table,
            guide_demo,
            rtl,
            guide_outline,
            api_reference,
            notes,
        ],
    );

    let body = body.test_id("ui-gallery-data-table-component");
    vec![body.into_element(cx)]
}
