fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn table_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/table.rs");

    for needle in [
        "repo-ref/ui/apps/v4/content/docs/components/base/table.mdx",
        "repo-ref/ui/apps/v4/registry/new-york-v4/ui/table.tsx",
        "repo-ref/ui/apps/v4/registry/new-york-v4/examples/table-demo.tsx",
        "repo-ref/ui/apps/v4/examples/base/table-footer.tsx",
        "repo-ref/ui/apps/v4/examples/base/table-actions.tsx",
        "repo-ref/ui/apps/v4/examples/base/table-rtl.tsx",
        "`TableHead` and `TableCaption` expose focused composable helpers (`table_head_children(...)` and `table_caption_children(...)`) for the upstream-shaped children pressure, while `TableCell` intentionally remains a single-child root surface.",
        "No broader generic root `children(...)` / `compose()` API is warranted here",
        "Unlike overlay/listbox components, this pass did not find a separate Radix/Base UI primitive contract to port for `Table`; the remaining drift was recipe/docs-surface work rather than a missing `fret-ui` mechanism.",
        "Preview mirrors the shadcn Table docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Footer`, `Actions`, `Data Table`, `RTL`, and `API Reference`; `Children (Fret)` and `Notes` stay as focused follow-ups for the composable-children decision and remaining public-surface guidance.",
        "`Children (Fret)` stays after `API Reference` as an explicit follow-up for the focused `table_head_children(...)` / `table_caption_children(...)` lane instead of widening the whole table family to a generic root children API.",
        "This pass did not identify a `fret-ui` mechanism or default-style regression: the remaining drift lived in `fret-ui-shadcn` recipe semantics and the UI Gallery teaching surface.",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Children (Fret)\", children)",
    ] {
        assert!(
            source.contains(needle),
            "table page should document source axes and the focused children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            footer,
            actions,
            data_table,
            rtl,
            api_reference,
            children,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "table page should keep the docs-path sections before the Fret-only follow-ups",
    );
}

#[test]
fn table_snippets_keep_default_lane_and_focused_children_followup() {
    let usage = include_str!("../src/ui/snippets/table/usage.rs");
    let actions = include_str!("../src/ui/snippets/table/actions.rs");
    let children = include_str!("../src/ui/snippets/table/children.rs");

    for needle in [
        "shadcn::table(",
        "shadcn::table_header(",
        "shadcn::table_body(",
        "shadcn::table_caption(\"A list of your recent invoices.\")",
        "shadcn::table_head(\"Amount\").text_align_end()",
    ] {
        assert!(
            usage.contains(needle),
            "table usage snippet should keep the default docs-shaped lane; missing `{needle}`",
        );
    }

    for needle in [
        "\"Wireless Mouse\"",
        "\"Mechanical Keyboard\"",
        "\"USB-C Hub\"",
        "shadcn::DropdownMenu::from_open(open_model.clone())",
        ".align(shadcn::DropdownMenuAlign::End)",
        "shadcn::table_cell(dropdown).text_align_end()",
    ] {
        assert!(
            actions.contains(needle),
            "table actions snippet should keep the upstream actions story copyable; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::table_head_children(|cx|",
        "shadcn::table_caption_children(|cx|",
        "shadcn::Badge::new(\"Live\")",
        "\"Use the children helpers when the compact text constructors are too narrow.\"",
    ] {
        assert!(
            children.contains(needle),
            "table children snippet should keep the focused head/caption children lane explicit; missing `{needle}`",
        );
    }

    assert!(
        !children.contains(".children(["),
        "table children follow-up should not widen into a generic root children API",
    );
    assert!(
        !children.contains(".compose("),
        "table children follow-up should not introduce an unnecessary compose lane",
    );
}

#[test]
fn table_docs_diag_script_covers_docs_path_and_fret_followups() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/table/ui-gallery-table-docs-smoke.json"
    );

    for needle in [
        "ui-gallery-table-demo-content",
        "ui-gallery-table-usage-content",
        "ui-gallery-table-footer-content",
        "ui-gallery-table-actions-content",
        "ui-gallery-table-actions-trigger-wireless-mouse",
        "ui-gallery-table-data-table-content",
        "ui-gallery-table-rtl-content",
        "ui-gallery-table-api-reference-content",
        "ui-gallery-table-children-content",
        "ui-gallery-table-notes-content",
    ] {
        assert!(
            script.contains(needle),
            "table docs diag script should cover the docs path and Fret follow-ups; missing `{needle}`",
        );
    }
}
