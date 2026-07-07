fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn button_group_page_keeps_upstream_docs_order_before_fret_followups() {
    let source = include_str!("../src/ui/pages/button_group.rs");

    for needle in [
        "Preview mirrors the shadcn Button Group docs order first, then appends Fret-specific follow-ups for `ButtonGroupText` and caller-owned flex growth.",
        "Gallery sections now mirror shadcn Button Group docs first: Demo, Usage, Accessibility, ButtonGroup vs ToggleGroup, examples, RTL, API Reference.",
        "`ButtonGroupText` and `Flex-1 items` remain after the upstream path as focused Fret follow-ups: one shows the explicit `new_children(...)` + `Label::for_control(...)` mapping for the upstream `asChild` label lane, the other demonstrates caller-owned flex negotiation.",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Accessibility\", accessibility)",
        "DocSection::build(cx, \"ButtonGroup vs ToggleGroup\", vs_toggle_group)",
        "DocSection::build(cx, \"Orientation\", orientation)",
        "DocSection::build(cx, \"Size\", size)",
        "DocSection::build(cx, \"Nested\", nested)",
        "DocSection::build(cx, \"Separator\", separator)",
        "DocSection::build(cx, \"Split\", split)",
        "DocSection::build(cx, \"Input\", input)",
        "DocSection::build(cx, \"Input Group\", input_group)",
        "DocSection::build(cx, \"Dropdown Menu\", dropdown)",
        "DocSection::build(cx, \"Select\", select)",
        "DocSection::build(cx, \"Popover\", popover)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"ButtonGroupText\", text)",
        "DocSection::build(cx, \"Flex-1 items (Fret)\", flex_1)",
    ] {
        assert!(
            source.contains(needle),
            "button group page should document the docs-path order and Fret follow-ups; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            accessibility,
            vs_toggle_group,
            orientation,
            size,
            nested,
            separator,
            split,
            input,
            input_group,
            dropdown,
            select,
            popover,
            rtl,
            api_reference,
            text,
            flex_1,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "button group page should keep the upstream docs path through API Reference before Fret-only follow-ups",
    );
}

#[test]
fn button_group_text_diag_script_gates_label_control_action_state() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-text-label-control-action-state.json"
    );
    let suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-button-group-text-label-control-action-state/suite.json"
    );

    for needle in [
        "\"FRET_UI_GALLERY_START_PAGE\": \"button_group\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"ButtonGroupText\"",
        "\"ui-gallery-button-group-text-content\"",
        "\"ui-gallery-button-group-text-prefix-label\"",
        "\"ui-gallery-button-group-text-control\"",
        "\"ui-gallery-button-group-text-suffix.label\"",
        "\"role_is\"",
        "\"label_contains\"",
        "\"semantics_action_is\"",
        "\"semantics_relation_includes\"",
        "\"focus_is\"",
        "\"value_equals\"",
        "\"action\": \"focus\"",
        "\"action\": \"set_value\"",
        "\"relation\": \"controls\"",
        "\"relation\": \"labelled_by\"",
        "\"text\": \"docs\"",
    ] {
        assert!(
            script.contains(needle),
            "button group text label/control script should gate the expected runtime path; missing `{needle}`",
        );
    }

    assert!(
        suite.contains(
            "tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-text-label-control-action-state.json"
        ),
        "button group text action-state suite should reference the canonical script",
    );
}
