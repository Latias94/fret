fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn radio_group_page_keeps_docs_order_and_fret_followups_explicit() {
    let source = include_str!("../src/ui/pages/radio_group.rs");

    for needle in [
        "Preview mirrors the current shadcn Radio Group docs path first: Demo and Usage. Description, Choice Card, Fieldset, Disabled, Required Disabled, Invalid, RTL, API Reference, and Label Association are Fret follow-ups. The composed rows use `into_element_parts(...)` for source-shaped control/label layout.",
        "`RadioGroup::into_element_parts(cx, |cx, parts| ...)` is the typed docs-parity seam for rows that need external `Field`, `Label`, `FieldLabel::for_control(...)`, or `FieldDescription` composition around the radio control.",
        "`RadioGroup::required(true)` now marks the group root as required, matching the upstream group-level required semantics instead of scattering the state across individual radio items.",
        "`Required Disabled` keeps required semantics on the radio group root while disabled semantics and action suppression stay on the disabled item control and its `FieldLabel::for_control(...)` bridge.",
        "`RadioGroupItem::child(...)` / `children(...)` and `variant(RadioGroupItemVariant::ChoiceCard)` remain recipe-owned shorthands for full-row content overrides, but the docs-path rows on this page now prefer `into_element_parts(...)` + `parts.control(...)`.",
        "The `RTL` preview keeps the translated upstream three-row example shape. `DirectionProvider(Rtl)` plus `into_element_parts(...)`, `Field`, and `FieldContent` keep the label/description on the logical side and the indicator on the opposite edge without extra physical alignment props.",
        "`Label Association` stays after the upstream docs path because it documents the Fret-specific `control_id(...)` bridge rather than an upstream section heading.",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Description\", description)",
        "DocSection::build(cx, \"Choice Card\", choice_card)",
        "DocSection::build(cx, \"Fieldset\", fieldset)",
        "DocSection::build(cx, \"Disabled\", disabled)",
        "DocSection::build(cx, \"Required Disabled\", required_disabled)",
        "DocSection::build(cx, \"Invalid\", invalid)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Label Association (Fret)\", label)",
    ] {
        assert!(
            source.contains(needle),
            "radio-group page should document the ordered docs path and Fret follow-ups; missing `{needle}`",
        );
    }

    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            description,
            choice_card,
            fieldset,
            disabled,
            required_disabled,
            invalid,
            rtl,
            api_reference,
            label,
        ]
        "#,
    );
    assert!(
        normalize_ws(source).contains(&ordered_sections),
        "radio-group page should render Demo and Usage before explicit Fret follow-ups, with API Reference before Label Association",
    );
}

#[test]
fn radio_group_snippets_keep_root_parts_and_followup_lanes_separate() {
    let demo = include_str!("../src/ui/snippets/radio_group/demo.rs");
    let usage = include_str!("../src/ui/snippets/radio_group/usage.rs");
    let description = include_str!("../src/ui/snippets/radio_group/description.rs");
    let choice_card = include_str!("../src/ui/snippets/radio_group/choice_card.rs");
    let fieldset = include_str!("../src/ui/snippets/radio_group/fieldset.rs");
    let disabled = include_str!("../src/ui/snippets/radio_group/disabled.rs");
    let required_disabled = include_str!("../src/ui/snippets/radio_group/required_disabled.rs");
    let invalid = include_str!("../src/ui/snippets/radio_group/invalid.rs");
    let rtl = include_str!("../src/ui/snippets/radio_group/rtl.rs");
    let label = include_str!("../src/ui/snippets/radio_group/label.rs");

    for needle in [
        "shadcn::RadioGroup::uncontrolled(Some(\"comfortable\"))",
        ".item(shadcn::RadioGroupItem::new(\"default\", \"Default\"))",
        ".item(shadcn::RadioGroupItem::new(\"comfortable\", \"Comfortable\"))",
        ".item(shadcn::RadioGroupItem::new(\"compact\", \"Compact\"))",
        ".into_element(cx)",
        ".test_id(\"ui-gallery-radio-group-demo\")",
    ] {
        assert!(
            demo.contains(needle),
            "radio-group Demo snippet should keep the compact root lane copyable; missing `{needle}`",
        );
    }
    assert!(
        !demo.contains(".into_element_parts("),
        "radio-group Demo should not expose the parts adapter lane",
    );

    for needle in [
        "shadcn::RadioGroup::uncontrolled(Some(\"option-one\"))",
        ".item(shadcn::RadioGroupItem::new(\"option-one\", \"Option One\").control_id(option_one_id))",
        ".item(shadcn::RadioGroupItem::new(\"option-two\", \"Option Two\").control_id(option_two_id))",
        ".into_element_parts(cx, |cx, parts| {",
        "parts.control(cx, \"option-one\")",
        "shadcn::Label::new(\"Option One\")",
        ".for_control(option_one_id)",
        "parts.control(cx, \"option-two\")",
        ".test_id(\"ui-gallery-radio-group-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "radio-group Usage snippet should keep the source-shaped row composition lane; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::FieldContent::new([",
        "shadcn::FieldLabel::new(\"Comfortable\")",
        "shadcn::FieldDescription::new(\"More space between elements.\")",
        "parts.control(cx, \"compact\")",
        ".test_id(\"ui-gallery-radio-group-description\")",
    ] {
        assert!(
            description.contains(needle),
            "radio-group Description follow-up should keep field content composition visible; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::FieldLabel::new(\"Starter Plan\")",
        ".for_control(starter_id)",
        ".wrap([shadcn::Field::new([",
        "shadcn::FieldTitle::new(\"Pro Plan\")",
        "parts.control(cx, \"pro\")",
        ".test_id(\"ui-gallery-radio-group-choice-card\")",
    ] {
        assert!(
            choice_card.contains(needle),
            "radio-group Choice Card follow-up should keep wrapped FieldLabel composition copyable; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::field_set(|cx| {",
        "shadcn::FieldLegend::new(\"Subscription Plan\")",
        "shadcn::FieldDescription::new(\"Yearly and lifetime plans offer significant savings.\")",
        "parts.control(cx, \"monthly\")",
        ".test_id(\"ui-gallery-radio-group-fieldset\")",
    ] {
        assert!(
            fieldset.contains(needle),
            "radio-group Fieldset follow-up should keep fieldset framing caller-owned; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::RadioGroupItem::new(\"option1\", \"Disabled\")",
        ".disabled(true)",
        ".control_id(disabled_id)",
        "parts.control(cx, \"option1\")",
        "shadcn::FieldLabel::new(\"Disabled\")",
        ".for_control(disabled_id)",
        ".test_id(\"ui-gallery-radio-group-disabled\")",
    ] {
        assert!(
            disabled.contains(needle),
            "radio-group Disabled follow-up should keep disabled state on the concrete item and field row; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::RadioGroup::new(value)",
        ".required(true)",
        ".test_id_prefix(\"ui-gallery-radio-group-required-disabled\")",
        "shadcn::RadioGroupItem::new(\"self-service\", \"Self-service\")",
        ".disabled(true)",
        ".control_id(self_service_id)",
        ".test_id(\"ui-gallery-radio-group-required-disabled-item-0-label\")",
        "Disabled rows keep disabled action-state on the concrete radio item.",
        ".test_id(\"ui-gallery-radio-group-required-disabled\")",
    ] {
        assert!(
            required_disabled.contains(needle),
            "radio-group Required Disabled follow-up should keep group required and item disabled state separate; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::RadioGroupItem::new(\"email\", \"Email only\")",
        ".aria_invalid(true)",
        ".control_id(email_id)",
        ".invalid(true)",
        "shadcn::FieldLabel::new(\"Both Email & SMS\")",
        ".test_id(\"ui-gallery-radio-group-invalid\")",
    ] {
        assert!(
            invalid.contains(needle),
            "radio-group Invalid follow-up should keep aria-invalid on items and Field invalid state caller-owned; missing `{needle}`",
        );
    }

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl, |cx| {",
        "shadcn::RadioGroup::uncontrolled(Some(\"comfortable\"))",
        ".into_element_parts(cx, |cx, parts| {",
        "parts.control(cx, \"default\")",
        "shadcn::FieldContent::new([",
        "shadcn::FieldLabel::new(\"\u{0627}\u{0641}\u{062a}\u{0631}\u{0627}\u{0636}\u{064a}\")",
        "\"\u{062a}\u{0628}\u{0627}\u{0639}\u{062f} \u{0642}\u{064a}\u{0627}\u{0633}\u{064a} \u{0644}\u{0645}\u{0639}\u{0638}\u{0645} \u{062d}\u{0627}\u{0644}\u{0627}\u{062a} \u{0627}\u{0644}\u{0627}\u{0633}\u{062a}\u{062e}\u{062f}\u{0627}\u{0645}.\"",
        "\"\u{0645}\u{0633}\u{0627}\u{062d}\u{0629} \u{0623}\u{0643}\u{0628}\u{0631} \u{0628}\u{064a}\u{0646} \u{0627}\u{0644}\u{0639}\u{0646}\u{0627}\u{0635}\u{0631}.\"",
        "\"\u{062a}\u{0628}\u{0627}\u{0639}\u{062f} \u{0623}\u{062f}\u{0646}\u{0649} \u{0644}\u{0644}\u{062a}\u{062e}\u{0637}\u{064a}\u{0637}\u{0627}\u{062a} \u{0627}\u{0644}\u{0643}\u{062b}\u{064a}\u{0641}\u{0629}.\"",
        ".test_id(\"ui-gallery-radio-group-rtl\")",
    ] {
        assert!(
            rtl.contains(needle),
            "radio-group RTL follow-up should keep the translated three-row parts lane; missing `{needle}`",
        );
    }
    assert!(
        !rtl.contains("\"Default\""),
        "radio-group RTL follow-up should not drift back to English labels",
    );

    for needle in [
        "let control_id = ControlId::from(\"ui-gallery-radio-group-label\");",
        "shadcn::radio_group(",
        ".control_id(control_id.clone())",
        ".test_id_prefix(\"ui-gallery-radio-group-label\")",
        "shadcn::field_group(|cx| {",
        "shadcn::FieldLabel::new(\"Plan\")",
        ".for_control(control_id.clone())",
        ".test_id(\"ui-gallery-radio-group-label-label\")",
    ] {
        assert!(
            label.contains(needle),
            "radio-group Label Association follow-up should stay focused on the Fret control-id bridge; missing `{needle}`",
        );
    }

    let combined = [
        demo,
        usage,
        description,
        choice_card,
        fieldset,
        disabled,
        required_disabled,
        invalid,
        rtl,
        label,
    ]
    .join("\n");
    for forbidden in [
        "shadcn::raw::",
        "advanced::",
        "RadioGroup::compose(",
        "RadioGroup::build_parts(",
        "RadioGroup::children(",
        "RadioGroupItem::child(",
        "RadioGroupItem::children(",
        "field_state_prim::",
        "interactivity_gate(",
        "asChild",
    ] {
        assert!(
            !combined.contains(forbidden),
            "radio-group copyable docs snippets should not promote `{forbidden}` on the default docs surface",
        );
    }
}

#[test]
fn radio_group_diag_scripts_cover_docs_path_semantics_and_followups() {
    let checked = include_str!(
        "../../../tools/diag-scripts/ui-gallery/radio-group/ui-gallery-radio-group-checked-state-mutation.json"
    );
    let required_disabled = include_str!(
        "../../../tools/diag-scripts/ui-gallery/radio-group/ui-gallery-radio-group-required-disabled-action-state.json"
    );
    let label_click = include_str!(
        "../../../tools/diag-scripts/ui-gallery/radio-group/ui-gallery-radio-group-label-click-focus.json"
    );
    let description_layout = include_str!(
        "../../../tools/diag-scripts/ui-gallery/radio-group/ui-gallery-radio-group-description-layout.json"
    );
    let choice_card_rtl = include_str!(
        "../../../tools/diag-scripts/ui-gallery/radio-group/ui-gallery-radio-group-choice-card-and-rtl.json"
    );

    for needle in [
        "\"name\": \"ui-gallery-radio-group-checked-state-mutation\"",
        "\"type_text\", \"text\": \"radio group\"",
        "\"ui-gallery-nav-radio-group\"",
        "\"ui-gallery-page-radio-group\"",
        "\"ui-gallery-radio-group-label-item-0\"",
        "\"ui-gallery-radio-group-label-item-1\"",
        "\"ui-gallery-radio-group-label-item-2\"",
        "\"kind\": \"checked_is\"",
        "\"label\": \"ui-gallery-radio-group-checked-state-mutation\"",
    ] {
        assert!(
            checked.contains(needle),
            "radio-group checked-state diagnostic should keep selection mutation evidence reachable; missing `{needle}`",
        );
    }

    for needle in [
        "\"FRET_UI_GALLERY_START_PAGE\": \"radio_group\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"Required Disabled\"",
        "\"role\": \"radio_group\", \"name\": \"Support plan\"",
        "\"kind\": \"required_is\"",
        "\"required\": true",
        "\"kind\": \"disabled_is\"",
        "\"disabled\": true",
        "\"kind\": \"semantics_action_is\"",
        "\"action\": \"invoke\"",
        "\"enabled\": false",
        "\"ui-gallery-radio-group-required-disabled-item-0-label\"",
        "\"ui-gallery-radio-group-required-disabled-item-2-label\"",
        "\"label\": \"ui-gallery-radio-group-required-disabled-action-state\"",
    ] {
        assert!(
            required_disabled.contains(needle),
            "radio-group required-disabled diagnostic should keep required and disabled action-state evidence; missing `{needle}`",
        );
    }

    for needle in [
        "\"name\": \"ui-gallery-radio-group-label-click-focus\"",
        "\"type_text\", \"text\": \"radio_group\"",
        "\"ui-gallery-radio-group-label-label\"",
        "\"kind\": \"focus_is\"",
        "\"ui-gallery-radio-group-label-item-0\"",
        "\"label\": \"ui-gallery-radio-group-label-click-focus\"",
    ] {
        assert!(
            label_click.contains(needle),
            "radio-group label-click diagnostic should keep the control-id focus bridge covered; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-radio-group-description\"",
        "\"kind\": \"bounds_min_size\"",
        "\"min_w_px\": 160.0",
        "\"label\": \"ui-gallery-radio-group-description-layout\"",
    ] {
        assert!(
            description_layout.contains(needle),
            "radio-group description diagnostic should keep layout evidence for composed rows; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-radio-group-choice-card\"",
        "\"label\": \"ui-gallery-radio-group-choice-card\"",
        "\"ui-gallery-radio-group-rtl\"",
        "\"label\": \"ui-gallery-radio-group-choice-card-and-rtl\"",
        "\"label\": \"ui-gallery-radio-group-rtl\"",
    ] {
        assert!(
            choice_card_rtl.contains(needle),
            "radio-group choice-card/RTL diagnostic should keep visual and RTL follow-up evidence; missing `{needle}`",
        );
    }
}
