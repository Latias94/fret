fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn checkbox_page_keeps_docs_order_before_fret_followups() {
    let source = include_str!("../src/ui/pages/checkbox.rs");

    for needle in [
        "Preview starts with the current shadcn Checkbox `Demo` and `Usage`, surfaces checked/invalid state and registry-shaped composition follow-ups, records the source-aligned snapshot/action story in `API Reference`, then keeps `Label Association` and `With Title` as focused Fret follow-ups.",
        "The top-level `Demo` now mirrors the upstream four-row composite preview (`Label`, description, disabled, and wrapped title/content) instead of collapsing that teaching surface into a single shortcut row.",
        "The follow-up `Description`, `Group`, `Required Disabled Group`, `Table`, and `RTL` sections keep registry/example-shaped composition, fieldset framing, and mixed select-all behavior visible without hiding them behind unrelated shortcuts.",
        "`Label Association` and `With Title` stay after the docs/registry path because they document Fret-specific control-registry and wrapped-field composition patterns.",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Checked State\", checked_state)",
        "DocSection::build(cx, \"Invalid State\", invalid_state)",
        "DocSection::build(cx, \"Basic\", basic)",
        "DocSection::build(cx, \"Description\", description_section)",
        "DocSection::build(cx, \"Disabled\", disabled_section)",
        "DocSection::build(cx, \"Group\", group)",
        "DocSection::build(cx, \"Required Disabled Group\", required_disabled_group)",
        "DocSection::build(cx, \"Table\", table)",
        "DocSection::build(cx, \"RTL\", rtl_section)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Label Association (Fret)\", label)",
        "DocSection::build(cx, \"With Title (Fret)\", with_title_section)",
    ] {
        assert!(
            source.contains(needle),
            "checkbox page should document the ordered docs path and Fret follow-ups; missing `{needle}`",
        );
    }

    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            checked_state,
            invalid_state,
            basic,
            description_section,
            disabled_section,
            group,
            required_disabled_group,
            table,
            rtl_section,
            api_reference,
            label,
            with_title_section,
        ]
        "#,
    );
    assert!(
        normalize_ws(source).contains(&ordered_sections),
        "checkbox page should render the shadcn docs/registry path through API Reference before the Fret-only label/title follow-ups",
    );
}

#[test]
fn checkbox_snippets_keep_leaf_control_and_field_composition_surface() {
    let demo = include_str!("../src/ui/snippets/checkbox/demo.rs");
    let usage = include_str!("../src/ui/snippets/checkbox/usage.rs");
    let checked_state = include_str!("../src/ui/snippets/checkbox/checked_state.rs");
    let invalid_state = include_str!("../src/ui/snippets/checkbox/invalid_state.rs");
    let description = include_str!("../src/ui/snippets/checkbox/description.rs");
    let disabled = include_str!("../src/ui/snippets/checkbox/disabled.rs");
    let group = include_str!("../src/ui/snippets/checkbox/group.rs");
    let required_disabled = include_str!("../src/ui/snippets/checkbox/required_disabled_group.rs");
    let table = include_str!("../src/ui/snippets/checkbox/table.rs");
    let rtl = include_str!("../src/ui/snippets/checkbox/rtl.rs");
    let label = include_str!("../src/ui/snippets/checkbox/label.rs");
    let with_title = include_str!("../src/ui/snippets/checkbox/with_title.rs");

    for needle in [
        "shadcn::field_group(|cx| {",
        "shadcn::Checkbox::new(basic)",
        "shadcn::FieldContent::new([",
        "shadcn::FieldDescription::new(",
        "shadcn::FieldLabel::new(\"Enable notifications\")",
        ".wrap([shadcn::Field::new([",
        "shadcn::FieldTitle::new(\"Enable notifications\")",
        ".test_id(\"ui-gallery-checkbox-demo\")",
    ] {
        assert!(
            demo.contains(needle),
            "checkbox Demo snippet should keep the upstream four-row field composition; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::Checkbox::new(checked_controlled)",
        "shadcn::Checkbox::new_optional(checked_optional)",
        "shadcn::Checkbox::from_checked(checked_snapshot_now)",
        ".action(act::ToggleSnapshot)",
        ".test_id(\"ui-gallery-checkbox-checked-state\")",
    ] {
        assert!(
            checked_state.contains(needle),
            "checkbox Checked State snippet should keep model-backed and snapshot/action lanes copyable; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::Checkbox::new(invalid)",
        ".aria_invalid(!invalid_checked)",
        ".invalid(!invalid_checked)",
        ".test_id(\"ui-gallery-checkbox-invalid-field\")",
    ] {
        assert!(
            invalid_state.contains(needle),
            "checkbox Invalid State snippet should keep invalid ownership on the control plus caller-owned field shell; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::field_set(|cx| {",
        "shadcn::FieldLegend::new(\"Show these items on the desktop:\")",
        "shadcn::FieldDescription::new(\"Select the items you want to show on the desktop.\")",
        ".checkbox_group()",
        ".test_id(\"ui-gallery-checkbox-group\")",
    ] {
        assert!(
            group.contains(needle),
            "checkbox Group snippet should keep fieldset and field-group framing visible; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::Checkbox::new(value)",
        ".required(true)",
        "checkbox.disabled(true)",
        "FieldLegend::new(\"Required desktop items\")",
        ".test_id(\"ui-gallery-checkbox-required-disabled-group\")",
    ] {
        assert!(
            required_disabled.contains(needle),
            "checkbox Required Disabled Group snippet should keep required/disabled state on concrete controls; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::Checkbox::from_checked_state(select_all_state)",
        ".action(act::ToggleAllRows)",
        ".test_id(\"ui-gallery-checkbox-table-all\")",
        "\"ui-gallery-checkbox-table-row-2\"",
        ".test_id(\"ui-gallery-checkbox-table\")",
    ] {
        assert!(
            table.contains(needle),
            "checkbox Table snippet should keep the mixed-state select-all action path copyable; missing `{needle}`",
        );
    }

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl, |cx| {",
        "shadcn::field_group(|cx| {",
        "ui-gallery-checkbox-rtl-basic",
        "ui-gallery-checkbox-rtl-description",
        "ui-gallery-checkbox-rtl-disabled",
        "ui-gallery-checkbox-rtl-with-title",
    ] {
        assert!(
            rtl.contains(needle),
            "checkbox RTL snippet should keep the translated four-row docs preview; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::Checkbox::new(checked)",
        "shadcn::FieldLabel::new(\"Accept terms and conditions\")",
        ".for_control(\"ui-gallery-checkbox-label-control\")",
        ".test_id(\"ui-gallery-checkbox-label\")",
    ] {
        assert!(
            label.contains(needle),
            "checkbox Label Association follow-up should stay focused on control registry wiring; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::FieldLabel::new(\"Enable notifications\")",
        ".wrap([shadcn::Field::new([",
        "shadcn::FieldTitle::new(\"Enable notifications\")",
        ".test_id(\"ui-gallery-checkbox-with-title-section\")",
    ] {
        assert!(
            with_title.contains(needle),
            "checkbox With Title follow-up should stay on wrapped field composition; missing `{needle}`",
        );
    }

    assert!(
        usage.contains("shadcn::Checkbox::new(checked)")
            && usage.contains(".control_id(\"ui-gallery-checkbox-usage\")")
            && usage.contains(".for_control(\"ui-gallery-checkbox-usage\")"),
        "checkbox Usage snippet should stay on the minimal curated Checkbox lane",
    );
    assert!(
        description.contains("shadcn::FieldDescription::new(")
            && description.contains(".test_id(\"ui-gallery-checkbox-description-field\")"),
        "checkbox Description snippet should keep helper text caller-owned through FieldDescription",
    );
    assert!(
        disabled.contains(".disabled(true)")
            && disabled.contains(".test_id(\"ui-gallery-checkbox-disabled-field\")"),
        "checkbox Disabled snippet should keep disabled semantics on the concrete checkbox",
    );

    let combined = [
        demo,
        usage,
        checked_state,
        invalid_state,
        description,
        disabled,
        group,
        required_disabled,
        table,
        rtl,
        label,
        with_title,
    ]
    .join("\n");
    for forbidden in [
        "shadcn::raw::",
        "advanced::",
        "Checkbox::compose(",
        "Checkbox::children(",
        "Checkbox::build_parts(",
        "CheckboxPart",
        "field_state_prim::",
        "interactivity_gate(",
        "asChild",
    ] {
        assert!(
            !combined.contains(forbidden),
            "checkbox copyable docs snippets should not promote `{forbidden}` on the default docs lane",
        );
    }
}

#[test]
fn checkbox_diag_scripts_cover_docs_path_semantics_and_followups() {
    let disabled = include_str!(
        "../../../tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-disabled-action-state.json"
    );
    let required_disabled = include_str!(
        "../../../tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-required-disabled-group-action-state.json"
    );
    let table = include_str!(
        "../../../tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-table-mixed-state-action.json"
    );
    let label_click = include_str!(
        "../../../tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-label-click-toggles.json"
    );
    let rtl = include_str!(
        "../../../tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-rtl-and-checked-wrap.json"
    );
    let suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-checkbox-semantics/suite.json");

    for needle in [
        "\"FRET_UI_GALLERY_START_PAGE\": \"checkbox\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"Disabled\"",
        "\"ui-gallery-checkbox-disabled\"",
        "\"ui-gallery-checkbox-disabled-label\"",
        "\"disabled_is\"",
        "\"semantics_action_is\"",
        "\"action\": \"invoke\"",
        "\"ui-gallery-checkbox-disabled-action-state\"",
    ] {
        assert!(
            disabled.contains(needle),
            "checkbox disabled action-state diag should gate concrete disabled control semantics; missing `{needle}`",
        );
    }

    for needle in [
        "\"FRET_UI_GALLERY_START_SECTION\": \"Required Disabled Group\"",
        "\"ui-gallery-checkbox-required-disabled-group\"",
        "\"ui-gallery-checkbox-required-disabled-analytics\"",
        "\"ui-gallery-checkbox-required-disabled-backups\"",
        "\"required_is\"",
        "\"disabled_is\"",
        "\"semantics_action_is\"",
    ] {
        assert!(
            required_disabled.contains(needle),
            "checkbox required-disabled diag should gate required and disabled semantics independently; missing `{needle}`",
        );
    }

    for needle in [
        "\"FRET_UI_GALLERY_START_SECTION\": \"Table\"",
        "\"ui-gallery-checkbox-table\"",
        "\"ui-gallery-checkbox-table-all\"",
        "\"ui-gallery-checkbox-table-row-2\"",
        "\"checked_state_is\"",
        "\"state\": \"mixed\"",
        "\"ui-gallery-checkbox-table-mixed-state-action\"",
    ] {
        assert!(
            table.contains(needle),
            "checkbox table diag should gate mixed select-all semantics and row mutation; missing `{needle}`",
        );
    }

    for needle in [
        "\"name\": \"ui-gallery-checkbox-label-click-toggles\"",
        "\"ui-gallery-checkbox-controlled-label\"",
        "\"ui-gallery-checkbox-controlled\"",
        "\"ui-gallery-checkbox-label-click-toggles\"",
    ] {
        assert!(
            label_click.contains(needle),
            "checkbox label-click diag should keep the control-label association observable; missing `{needle}`",
        );
    }

    for needle in [
        "\"id\": \"ui-gallery-checkbox-checked-state\"",
        "\"label\": \"ui-gallery-checkbox-checked-state-wrap\"",
        "\"id\": \"ui-gallery-checkbox-rtl\"",
        "\"label\": \"ui-gallery-checkbox-rtl-and-checked-wrap\"",
    ] {
        assert!(
            rtl.contains(needle),
            "checkbox RTL/checked-wrap diag should keep docs-path visual anchors observable; missing `{needle}`",
        );
    }

    for script in [
        "ui-gallery-checkbox-disabled-action-state.json",
        "ui-gallery-checkbox-required-disabled-group-action-state.json",
        "ui-gallery-checkbox-table-mixed-state-action.json",
    ] {
        assert!(
            suite.contains(script),
            "checkbox semantics suite should include `{script}`",
        );
    }
}
