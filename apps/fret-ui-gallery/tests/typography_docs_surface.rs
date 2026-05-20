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
fn typography_table_snippets_keep_fixed_cell_text_on_table_role() {
    let module_source = include_str!("../src/ui/snippets/typography/mod.rs");
    let module_canonical = canonicalize_rust_fragment(module_source);

    for needle in [
        "pub(super) fn table_cell_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement",
        "fret_ui_kit::declarative::text::text_table_cell(cx, text)",
    ] {
        let needle = canonicalize_rust_fragment(needle);
        assert!(
            module_canonical.contains(&needle),
            "typography snippets should share a directory helper backed by table-cell text roles; missing `{needle}`"
        );
    }

    for (name, source, required, forbidden) in [
        (
            "table",
            include_str!("../src/ui/snippets/typography/table.rs"),
            &[
                "super::table_cell_text(cx, \"Empty\")",
                "super::table_cell_text(cx, \"Overflowing\")",
                "super::table_cell_text(cx, \"Modest\")",
                "super::table_cell_text(cx, \"Satisfied\")",
                "super::table_cell_text(cx, \"Full\")",
                "super::table_cell_text(cx, \"Ecstatic\")",
            ][..],
            &[
                "shadcn::table_cell(ui::text(\"Empty\"))",
                "shadcn::table_cell(ui::text(\"Overflowing\"))",
                "shadcn::table_cell(ui::text(\"Modest\"))",
                "shadcn::table_cell(ui::text(\"Satisfied\"))",
                "shadcn::table_cell(ui::text(\"Full\"))",
                "shadcn::table_cell(ui::text(\"Ecstatic\"))",
            ][..],
        ),
        (
            "demo",
            include_str!("../src/ui/snippets/typography/demo.rs"),
            &[
                "super::table_cell_text(cx, \"Empty\")",
                "super::table_cell_text(cx, \"Overflowing\")",
                "super::table_cell_text(cx, \"Modest\")",
                "super::table_cell_text(cx, \"Satisfied\")",
                "super::table_cell_text(cx, \"Full\")",
                "super::table_cell_text(cx, \"Ecstatic\")",
            ][..],
            &[
                "shadcn::table_cell(ui::text(\"Empty\"))",
                "shadcn::table_cell(ui::text(\"Overflowing\"))",
                "shadcn::table_cell(ui::text(\"Modest\"))",
                "shadcn::table_cell(ui::text(\"Satisfied\"))",
                "shadcn::table_cell(ui::text(\"Full\"))",
                "shadcn::table_cell(ui::text(\"Ecstatic\"))",
            ][..],
        ),
        (
            "rtl",
            include_str!("../src/ui/snippets/typography/rtl.rs"),
            &[
                "super::table_cell_text(cx, \"فارغة\")",
                "super::table_cell_text(cx, \"فائضة\")",
                "super::table_cell_text(cx, \"متواضعة\")",
                "super::table_cell_text(cx, \"راضٍ\")",
                "super::table_cell_text(cx, \"ممتلئة\")",
                "super::table_cell_text(cx, \"منتشٍ\")",
            ][..],
            &[
                "shadcn::table_cell(ui::text(\"فارغة\"))",
                "shadcn::table_cell(ui::text(\"فائضة\"))",
                "shadcn::table_cell(ui::text(\"متواضعة\"))",
                "shadcn::table_cell(ui::text(\"راضٍ\"))",
                "shadcn::table_cell(ui::text(\"ممتلئة\"))",
                "shadcn::table_cell(ui::text(\"منتشٍ\"))",
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
