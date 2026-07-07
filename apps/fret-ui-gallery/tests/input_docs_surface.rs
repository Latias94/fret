fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn input_page_keeps_upstream_docs_order_before_fret_followups() {
    let source = include_str!("../src/ui/pages/input.rs");

    for needle in [
        "A text input component for forms and user data entry with built-in styling and accessibility features. This page mirrors the shadcn Input docs path first, then adds focused Fret follow-ups for label association, ownership notes, and diagnostics guidance.",
        "This page follows the upstream Input docs path first: Demo, Usage, Basic, Field, Field Group, Disabled, Invalid, File, Inline, Grid, Required, Badge, Input Group, Button Group, Form, RTL.",
        "The dedicated `Label Association` section remains the focused regression-friendly follow-up that proves label clicks, `labelled-by`, and `described-by` survive on the gallery page.",
        "`Input` stays a leaf control; labels/descriptions/errors belong in `Field`, and inline adornments belong in `InputGroup` / `ButtonGroup`, so no generic `children(...)` / `asChild` surface is needed here.",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Basic\", basic)",
        "DocSection::build(cx, \"Field\", field)",
        "DocSection::build(cx, \"Field Group\", field_group)",
        "DocSection::build(cx, \"Disabled\", disabled)",
        "DocSection::build(cx, \"Invalid\", invalid)",
        "DocSection::build(cx, \"File\", file)",
        "DocSection::build(cx, \"Inline\", inline)",
        "DocSection::build(cx, \"Grid\", grid)",
        "DocSection::build(cx, \"Required\", required)",
        "DocSection::build(cx, \"Badge\", badge)",
        "DocSection::build(cx, \"Input Group\", input_group)",
        "DocSection::build(cx, \"Button Group\", button_group)",
        "DocSection::build(cx, \"Form\", form)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"Label Association\", label)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Notes\", notes)",
    ] {
        assert!(
            source.contains(needle),
            "input page should document the docs-path order and Fret follow-ups; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            basic,
            field,
            field_group,
            disabled,
            invalid,
            file,
            inline,
            grid,
            required,
            badge,
            input_group,
            button_group,
            form,
            rtl,
            label,
            api_reference,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "input page should keep the upstream docs path before Fret-only follow-ups",
    );
}

#[test]
fn input_docs_diag_scripts_cover_docs_path_and_runtime_followups() {
    let docs = include_str!(
        "../../../tools/diag-scripts/ui-gallery/input/ui-gallery-input-docs-screenshots.json"
    );
    let label = include_str!(
        "../../../tools/diag-scripts/ui-gallery/input/ui-gallery-input-label-click-focus.json"
    );
    let file = include_str!(
        "../../../tools/diag-scripts/ui-gallery/input/ui-gallery-input-file-browse-mocked.json"
    );
    let label_stub =
        include_str!("../../../tools/diag-scripts/ui-gallery-input-label-click-focus.json");
    let file_stub =
        include_str!("../../../tools/diag-scripts/ui-gallery-input-file-browse-mocked.json");

    for needle in [
        "\"ui-gallery-input-demo-content\"",
        "\"ui-gallery-input-usage-content\"",
        "\"ui-gallery-input-field-group-content\"",
        "\"ui-gallery-input-file-section-content\"",
        "\"ui-gallery-input-form-content\"",
        "\"ui-gallery-input-label-content\"",
        "\"ui-gallery-input-api-reference-content\"",
        "\"ui-gallery-input-docs\"",
    ] {
        assert!(
            docs.contains(needle),
            "input docs screenshot script should cover docs-path and follow-up anchors; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-input-label-label\"",
        "\"ui-gallery-input-label-control\"",
        "\"value_equals\"",
        "\"text\": \"alice\"",
        "\"ui-gallery-input-label-click-focus\"",
    ] {
        assert!(
            label.contains(needle),
            "input label click-focus script should gate label/control focus and value mutation; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-input-file\"",
        "\"ui-gallery-input-file-browse\"",
        "\"ui-gallery-input-file-selected\"",
        "\"ui-gallery-input-file-browse-mocked\"",
    ] {
        assert!(
            file.contains(needle),
            "input file browse script should keep the deterministic file composition anchors; missing `{needle}`",
        );
    }

    assert!(
        label_stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/input/ui-gallery-input-label-click-focus.json\""
        ),
        "input label redirect stub should point at the canonical input script",
    );
    assert!(
        file_stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/input/ui-gallery-input-file-browse-mocked.json\""
        ),
        "input file redirect stub should point at the canonical input script",
    );
}
