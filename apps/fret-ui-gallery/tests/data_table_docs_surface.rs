fn canonicalize_rust_fragment(fragment: &str) -> String {
    let mut canonical = fragment.split_whitespace().collect::<String>();
    loop {
        let next = canonical.replace(",)", ")");
        if next == canonical {
            return next;
        }
        canonical = next;
    }
}

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

#[test]
fn data_table_snippets_keep_fixed_cell_text_on_table_role() {
    let module_source = include_str!("../src/ui/snippets/data_table/mod.rs");
    let module_canonical = canonicalize_rust_fragment(module_source);

    for needle in [
        "pub(super) fn table_cell_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement",
        "fret_ui_kit::declarative::text::text_table_cell(cx, text)",
    ] {
        let needle = canonicalize_rust_fragment(needle);
        assert!(
            module_canonical.contains(&needle),
            "data-table snippets should share a directory helper backed by text_table_cell(...); missing `{needle}`"
        );
    }

    for (name, source, required, forbidden) in [
        (
            "default_demo",
            include_str!("../src/ui/snippets/data_table/default_demo.rs"),
            &[
                "super::table_cell_text(cx, row.status.clone())",
                "super::table_cell_text(cx, row.customer_email.clone())",
            ][..],
            &[
                "cx.text(row.status.as_ref())",
                "cx.text(row.customer_email.as_ref())",
            ][..],
        ),
        (
            "basic_demo",
            include_str!("../src/ui/snippets/data_table/basic_demo.rs"),
            &[
                "super::table_cell_text(cx, row.status.clone())",
                "super::table_cell_text(cx, row.email.clone())",
            ][..],
            &[
                "cx.text(row.status.as_ref())",
                "cx.text(row.email.as_ref())",
            ][..],
        ),
        (
            "guide_demo",
            include_str!("../src/ui/snippets/data_table/guide_demo.rs"),
            &[
                "super::table_cell_text(cx, row.name.clone())",
                "super::table_cell_text(cx, row.status.clone())",
                "super::table_cell_text(cx, format!(\"{}%\", row.cpu))",
                "super::table_cell_text(cx, format!(\"{} MB\", row.mem_mb))",
            ][..],
            &[
                "cx.text(row.name.as_ref())",
                "cx.text(row.status.as_ref())",
                "cx.text(format!(\"{}%\", row.cpu))",
                "cx.text(format!(\"{} MB\", row.mem_mb))",
            ][..],
        ),
        (
            "rtl_demo",
            include_str!("../src/ui/snippets/data_table/rtl_demo.rs"),
            &[
                "super::table_cell_text(cx, row.status.clone())",
                "super::table_cell_text(cx, row.email.clone())",
            ][..],
            &[
                "cx.text(row.status.as_ref())",
                "cx.text(row.email.as_ref())",
            ][..],
        ),
        (
            "code_outline",
            include_str!("../src/ui/snippets/data_table/code_outline.rs"),
            &[
                "super::table_cell_text(cx, row.status.clone()).test_id(reusable_cell_test_id(row, \"status\"))",
                "super::table_cell_text(cx, row.email.clone()).test_id(reusable_cell_test_id(row, \"email\"))",
            ][..],
            &[".text(row.status.as_ref())", ".text(row.email.as_ref())"][..],
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);
        for marker in required {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed table-cell text through the shared table-cell role; missing `{marker}`"
            );
        }
        for marker in forbidden {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                !canonical.contains(&marker),
                "{name} reintroduced bare table-cell text: `{marker}`"
            );
        }
        assert!(
            !canonical.contains("=>cx.text(\"?\")"),
            "{name} should route fallback table-cell text through the shared table-cell role"
        );
    }
}
