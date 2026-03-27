#[test]
fn data_table_page_documents_guide_mapping_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/data_table.rs");

    for needle in [
        "DocSection::build(cx, \"Guide Coverage\", guide_coverage)",
        "Cell Formatting",
        "Row Actions, Pagination, Sorting, Filtering, Visibility, and Row Selection",
        "DataTableColumnHeader",
        "No extra root `children` API is required here",
        "DataTableToolbar::trailing(...)",
    ] {
        assert!(
            source.contains(needle),
            "data_table page should document guide coverage and the root-children API decision; missing `{needle}`"
        );
    }
}

#[test]
fn data_table_guide_demo_uses_tasks_responsive_toolbar_surface() {
    let source = include_str!("../src/ui/snippets/data_table/guide_demo.rs");

    for needle in [
        "viewport_width_at_least",
        "viewport_tailwind::LG",
        "let filter_width = if viewport_lg { Px(250.0) } else { Px(150.0) };",
        ".show_columns_menu(viewport_lg)",
    ] {
        assert!(
            source.contains(needle),
            "guide demo should keep the tasks-style responsive toolbar surface; missing `{needle}`"
        );
    }
}
