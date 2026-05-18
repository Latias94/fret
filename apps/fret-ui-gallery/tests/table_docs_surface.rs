fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

fn canonicalize_rust_fragment(fragment: &str) -> String {
    let mut canonical = fragment.split_whitespace().collect::<String>();
    loop {
        let next = canonical.replace(",)", ")");
        if next == canonical {
            return canonical;
        }
        canonical = next;
    }
}

#[test]
fn table_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/table.rs");

    for needle in [
        "Reference baseline: shadcn base Table docs.",
        "Visual/chrome baseline: the default shadcn registry table plus the demo, footer, actions, and RTL examples.",
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
fn table_snippets_keep_fixed_cell_text_on_table_roles() {
    let module_source = include_str!("../src/ui/snippets/table/mod.rs");
    let module_canonical = canonicalize_rust_fragment(module_source);

    for needle in [
        "pub(super) fn table_cell_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement",
        "fret_ui_kit::declarative::text::text_table_cell(cx, text)",
        "pub(super) fn table_cell_text_emphasis<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement",
        "fret_ui_kit::declarative::text::text_table_cell_emphasis(cx, text)",
    ] {
        let needle = canonicalize_rust_fragment(needle);
        assert!(
            module_canonical.contains(&needle),
            "table snippets should share directory helpers backed by table-cell text roles; missing `{needle}`"
        );
    }

    for (name, source, required, forbidden) in [
        (
            "demo",
            include_str!("../src/ui/snippets/table/demo.rs"),
            &[
                "super::table_cell_text_emphasis(cx, invoice)",
                "super::table_cell_text(cx, status)",
                "super::table_cell_text(cx, method)",
                "super::table_cell_text(cx, amount)",
                "super::table_cell_text(cx, \"Total\")",
                "super::table_cell_text(cx, \"$2,500.00\")",
            ][..],
            &[
                "ui::text(invoice)",
                "ui::text(status)",
                "ui::text(method)",
                "ui::text(amount)",
                "ui::text(\"Total\")",
                "ui::text(\"$2,500.00\")",
            ][..],
        ),
        (
            "usage",
            include_str!("../src/ui/snippets/table/usage.rs"),
            &[
                "super::table_cell_text_emphasis(cx, \"INV001\")",
                "super::table_cell_text(cx, \"Paid\")",
                "super::table_cell_text(cx, \"Credit Card\")",
                "super::table_cell_text(cx, \"$250.00\")",
            ][..],
            &[
                "ui::text(\"INV001\")",
                "ui::text(\"Paid\")",
                "ui::text(\"Credit Card\")",
                "ui::text(\"$250.00\")",
            ][..],
        ),
        (
            "footer",
            include_str!("../src/ui/snippets/table/footer.rs"),
            &[
                "super::table_cell_text_emphasis(cx, invoice)",
                "super::table_cell_text(cx, status)",
                "super::table_cell_text(cx, method)",
                "super::table_cell_text(cx, amount)",
                "super::table_cell_text(cx, \"Total\")",
                "super::table_cell_text(cx, \"$2,500.00\")",
            ][..],
            &[
                "ui::text(invoice)",
                "ui::text(status)",
                "ui::text(method)",
                "ui::text(amount)",
                "ui::text(\"Total\")",
                "ui::text(\"$2,500.00\")",
            ][..],
        ),
        (
            "rtl",
            include_str!("../src/ui/snippets/table/rtl.rs"),
            &[
                "super::table_cell_text_emphasis(cx, invoice)",
                "super::table_cell_text(cx, status)",
                "super::table_cell_text(cx, method)",
                "super::table_cell_text(cx, amount)",
                "super::table_cell_text(cx, \"المجموع\")",
                "super::table_cell_text(cx, \"$2,500.00\")",
            ][..],
            &[
                "ui::text(invoice)",
                "ui::text(status)",
                "ui::text(method)",
                "ui::text(amount)",
                "ui::text(\"المجموع\")",
                "ui::text(\"$2,500.00\")",
            ][..],
        ),
        (
            "actions",
            include_str!("../src/ui/snippets/table/actions.rs"),
            &[
                "super::table_cell_text_emphasis(cx, product)",
                "super::table_cell_text(cx, price)",
            ][..],
            &["ui::text(product)", "ui::text(price)"][..],
        ),
        (
            "children",
            include_str!("../src/ui/snippets/table/children.rs"),
            &[
                "super::table_cell_text_emphasis(cx, \"INV101\")",
                "super::table_cell_text(cx, \"Paid\")",
                "super::table_cell_text(cx, \"Credit Card\")",
                "super::table_cell_text(cx, \"$120.00\")",
                "super::table_cell_text_emphasis(cx, \"INV102\")",
                "super::table_cell_text(cx, \"Pending\")",
                "super::table_cell_text(cx, \"Wire Transfer\")",
                "super::table_cell_text(cx, \"$340.00\")",
            ][..],
            &[
                "shadcn::table_cell(ui::text(\"INV101\")",
                "shadcn::table_cell(ui::text(\"Paid\"))",
                "shadcn::table_cell(ui::text(\"Credit Card\"))",
                "shadcn::table_cell(ui::text(\"$120.00\"))",
                "shadcn::table_cell(ui::text(\"INV102\")",
                "shadcn::table_cell(ui::text(\"Pending\"))",
                "shadcn::table_cell(ui::text(\"Wire Transfer\"))",
                "shadcn::table_cell(ui::text(\"$340.00\"))",
            ][..],
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
                "{name} reintroduced bare fixed table-cell text: `{marker}`"
            );
        }
    }
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

    let parsed: serde_json::Value = serde_json::from_str(script).expect("valid table diag script");
    let steps = parsed["steps"].as_array().expect("steps array");
    for title_id in [
        "ui-gallery-table-actions-title",
        "ui-gallery-table-rtl-title",
    ] {
        let step = steps
            .iter()
            .find(|step| {
                step.get("type").and_then(|v| v.as_str()) == Some("scroll_into_view")
                    && step
                        .get("target")
                        .and_then(|target| target.get("id"))
                        .and_then(|v| v.as_str())
                        == Some(title_id)
            })
            .unwrap_or_else(|| panic!("missing scroll_into_view step for {title_id}"));
        assert_eq!(
            step.get("require_fully_within_container")
                .and_then(|v| v.as_bool()),
            Some(true),
            "{title_id} scroll step should keep the heading fully inside the scroll container",
        );
        assert_eq!(
            step.get("require_fully_within_window")
                .and_then(|v| v.as_bool()),
            Some(true),
            "{title_id} scroll step should keep the heading fully inside the window before the strict wait",
        );
    }
}
