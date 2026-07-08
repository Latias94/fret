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

#[test]
fn typography_interactive_links_keep_runtime_action_state_anchors() {
    let page = include_str!("../src/ui/pages/typography.rs");
    let snippet = include_str!("../src/ui/snippets/typography/interactive_links.rs");

    for needle in [
        "DocSection::build(cx, \"Interactive Links\", interactive_links)",
        "Fret follow-up showing the copyable app-facing `p_rich(...).on_activate_link(...)` lane.",
    ] {
        assert!(
            page.contains(needle),
            "typography page should keep the Interactive Links doc section observable; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::typography::p_rich([",
        "shadcn::typography::inline_link(\"a brilliant plan\", \"https://example.com/kings-plan\")",
        ".on_activate_link(Arc::new({",
        "activation.tag.clone()",
        ".test_id(\"ui-gallery-typography-interactive-links-paragraph\")",
        ".test_id(\"ui-gallery-typography-interactive-links-status-active\")",
        ".test_id(\"ui-gallery-typography-interactive-links-status-idle\")",
        ".test_id(\"ui-gallery-typography-interactive-links\")",
    ] {
        assert!(
            snippet.contains(needle),
            "typography interactive-links snippet should keep runtime-observable inline-link anchors; missing `{needle}`",
        );
    }
}

#[test]
fn typography_interactive_links_diag_script_gates_inline_span_activation() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-interactive-links-activation.json"
    );
    let stub = include_str!(
        "../../../tools/diag-scripts/ui-gallery-typography-interactive-links-activation.json"
    );
    let suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-typography-inline-link-action-state/suite.json"
    );

    for needle in [
        "\"ui-gallery-typography-interactive-links-activation\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"typography\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"Interactive Links\"",
        "\"ui-gallery-page-typography\"",
        "\"docsec-interactive-links-content\"",
        "\"ui-gallery-typography-interactive-links\"",
        "\"ui-gallery-typography-interactive-links-paragraph\"",
        "\"role_is\"",
        "\"value_contains\"",
        "\"semantics_action_is\"",
        "\"set_text_selection\"",
        "\"semantics_inline_span_includes\"",
        "\"role\": \"link\"",
        "\"tag\": \"https://example.com/kings-plan\"",
        "\"click_selectable_text_span_stable\"",
        "\"ui-gallery-typography-interactive-links-status-active\"",
        "\"capture_layout_sidecar\"",
    ] {
        assert!(
            script.contains(needle),
            "typography interactive-links script should gate inline span activation; missing `{needle}`",
        );
    }

    assert!(
        stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-interactive-links-activation.json\""
        ),
        "typography interactive-links redirect stub should point at the canonical script",
    );
    assert!(
        suite.contains(
            "tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-interactive-links-activation.json"
        ),
        "typography inline-link action-state suite should reference the promoted script",
    );
}
