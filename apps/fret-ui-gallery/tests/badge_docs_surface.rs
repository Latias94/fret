#[test]
fn badge_link_render_keeps_runtime_action_state_anchors() {
    let link = include_str!("../src/ui/snippets/badge/link.rs");

    for needle in [
        ".render(shadcn::BadgeRender::Link",
        ".on_activate(Arc::new(|_host, _acx, _reason| {}))",
        ".test_id(\"ui-gallery-badge-link\")",
        ".test_id(\"ui-gallery-badge-link-row\")",
    ] {
        assert!(
            link.contains(needle),
            "badge link snippet should keep runtime-observable link anchors; missing `{needle}`",
        );
    }
}

#[test]
fn badge_link_diag_script_gates_action_state() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-link-render.json"
    );
    let stub = include_str!("../../../tools/diag-scripts/ui-gallery-badge-link-render.json");
    let suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-badge-link-action-state/suite.json"
    );

    for needle in [
        "\"ui-gallery-badge-link-render\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"badge\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"Link\"",
        "\"ui-gallery-badge-link-row\"",
        "\"ui-gallery-badge-link\"",
        "\"role_is\"",
        "\"semantics_action_is\"",
        "\"focus_is\"",
        "\"role\": \"link\"",
        "\"text\": \"Open Link\"",
    ] {
        assert!(
            script.contains(needle),
            "badge link-render script should gate the expected runtime path; missing `{needle}`",
        );
    }

    assert!(
        stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-link-render.json\""
        ),
        "badge link-render redirect stub should point at the canonical badge script",
    );
    assert!(
        suite.contains("tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-link-render.json"),
        "badge link action-state suite should reference the promoted script",
    );
}

fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn badge_page_keeps_upstream_docs_order_before_counts_followup() {
    let source = include_str!("../src/ui/pages/badge.rs");

    for needle in [
        "Preview mirrors the shadcn Badge docs path first: Demo, Usage, Variants, With Icon, With Spinner, Link, Custom Colors, RTL, and API Reference. `Counts (Fret)` stays as an explicit follow-up.",
        "Badge doc snippets own the centered preview rows because upstream places `justify-center` on the example call site, not the component source.",
        "`Counts (Fret)` intentionally stays after the upstream path so compact numeric badge diagnostics remain stable without polluting the docs-aligned example sequence.",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Variants\", variants)",
        "DocSection::build(cx, \"With Icon\", with_icon)",
        "DocSection::build(cx, \"With Spinner\", with_spinner)",
        "DocSection::build(cx, \"Link\", link)",
        "DocSection::build(cx, \"Custom Colors\", colors)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Counts (Fret)\", counts)",
    ] {
        assert!(
            source.contains(needle),
            "badge page should document the docs-path order and Counts follow-up; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            variants,
            with_icon,
            with_spinner,
            link,
            colors,
            rtl,
            api_reference,
            counts,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "badge page should keep the upstream docs path through API Reference before the Fret-only Counts follow-up",
    );
}
