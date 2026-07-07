#[test]
fn form_page_keeps_docs_order_and_field_ownership_notes() {
    let source = include_str!("../src/ui/pages/form.rs");

    for needle in [
        "Gallery starts with the upstream-aligned Form Demo, then keeps copyable Usage and submit-validation sections plus Disabled Field, Input, Textarea, Checkbox + Switch, Fieldset, RTL, and Notes as Form follow-ups while preserving field-level required semantics and `FormState`-driven invalid decoration on `FormField`.",
        "DocSection::build(cx, \"Form Demo\", upstream_demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Submit Validation\", submit_validation)",
        "DocSection::build(cx, \"Disabled Field\", disabled_field)",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Input\", input)",
        "DocSection::build(cx, \"Textarea\", textarea)",
        "DocSection::build(cx, \"Checkbox + Switch\", controls)",
        "DocSection::build(cx, \"Fieldset\", fieldset)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"Notes\", notes)",
        "field-level `required` ownership on `FormField::required(true)`",
        "wrapper-owned invalid decoration from `FormState`",
        "concrete disabled Input owns disabled semantics",
        "Focused Fret RTL follow-up",
    ] {
        assert!(
            source.contains(needle),
            "form page should document the copyable order and ownership notes; missing `{needle}`",
        );
    }

    let render_order = source
        .find(
            r#"vec![
            upstream_demo,
            usage,
            submit_validation,
            disabled_field,
            demo,
            input,
            textarea,
            controls,
            fieldset,
            rtl,
            notes,
        ]"#,
        )
        .expect("form page should render sections in the documented order");
    let docs_path_note = source
        .find("Gallery starts with the upstream-aligned Form Demo")
        .expect("form page should label the ordered docs/follow-up surface");
    assert!(
        docs_path_note < render_order,
        "form page should explain the ordered docs surface before rendering it",
    );
}

#[test]
fn form_snippets_keep_curated_form_field_surface_without_raw_typography() {
    let notes = include_str!("../src/ui/snippets/form/notes.rs");
    let upstream_demo = include_str!("../src/ui/snippets/form/upstream_demo.rs");
    let usage = include_str!("../src/ui/snippets/form/usage.rs");
    let submit_validation = include_str!("../src/ui/snippets/form/submit_validation.rs");
    let disabled_field = include_str!("../src/ui/snippets/form/disabled_field.rs");

    for needle in [
        "shadcn::FormDescription::new(",
        "Reference baseline: the upstream internal form demo.",
        "FormControl` stays a transparent single-control wrapper rather than a layout column.",
        "Field-level required semantics belong on `FormField::required(true)`",
        "Invalid decoration also belongs to `FormField`",
        "There is no standalone upstream `Form` RTL component page/example",
    ] {
        assert!(
            notes.contains(needle),
            "form Notes snippet should teach ownership through the curated Form surface; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::FormField::new(",
        ".label(\"Username\")",
        ".required(true)",
        ".description(\"This is your public display name.\")",
        "shadcn::Form::new([",
        ".test_id(\"ui-gallery-form-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "form Usage snippet should stay on the default Form/FormField lane; missing `{needle}`",
        );
    }

    for needle in [
        "FormRegistry::new().options(FormRegistryOptions {",
        "validate_mode: FormValidateMode::OnSubmit",
        "registry.submit_action_host(host, &form_state)",
        "shadcn::FormField::new(",
        "decl_text::text_control_readout(cx, format!(\"status={result_status}\"))",
        ".test_id(\"ui-gallery-form-submit-validation\")",
    ] {
        assert!(
            submit_validation.contains(needle),
            "form Submit Validation snippet should keep the FormState runtime lane; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::Field::new(ui::children![",
        ".disabled(true)",
        ".test_id(\"ui-gallery-form-disabled-field-control\")",
        ".test_id(\"ui-gallery-form-disabled-field-enabled-control\")",
    ] {
        assert!(
            disabled_field.contains(needle),
            "form Disabled Field snippet should keep field shell and concrete control semantics separate; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::FieldDescription::new(",
        "You can manage your mobile notifications in the mobile settings page.",
        "shadcn::FieldTitle::new(\"Sidebar\")",
        "Select the items you want to display in the sidebar.",
        "Receive emails about new products, features, and more.",
        "Receive emails about your account security.",
        "shadcn::FieldTitle::new(\"Email Notifications\")",
        ".decorate_control(false)",
    ] {
        assert!(
            upstream_demo.contains(needle),
            "form upstream demo should keep description/title copy on curated Field surfaces; missing `{needle}`",
        );
    }

    let combined = [
        notes,
        upstream_demo,
        usage,
        submit_validation,
        disabled_field,
    ]
    .join("\n");
    for forbidden in ["shadcn::raw::", "advanced::", "compose()", "asChild"] {
        assert!(
            !combined.contains(forbidden),
            "form copyable docs snippets should not promote `{forbidden}` on the default Form lane",
        );
    }
}

#[test]
fn form_diag_scripts_cover_docs_and_stateful_followups() {
    let docs =
        include_str!("../../../tools/diag-scripts/ui-gallery/form/ui-gallery-form-docs-smoke.json");
    let submit = include_str!(
        "../../../tools/diag-scripts/ui-gallery/form/ui-gallery-form-submit-validation-semantics.json"
    );
    let disabled = include_str!(
        "../../../tools/diag-scripts/ui-gallery/form/ui-gallery-form-disabled-field-action-state.json"
    );

    for needle in [
        "\"ui-gallery-form-docs-smoke\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"form\"",
        "\"type_text\", \"text\": \"form\"",
        "\"ui-gallery-nav-form\"",
        "\"ui-gallery-form\"",
        "\"ui-gallery-form-demo\"",
    ] {
        assert!(
            docs.contains(needle),
            "form docs smoke script should keep the page and upstream demo reachable; missing `{needle}`",
        );
    }

    for needle in [
        "\"FRET_UI_GALLERY_START_SECTION\": \"Submit Validation\"",
        "\"ui-gallery-form-submit-validation\"",
        "\"ui-gallery-form-submit-validation-username-control\"",
        "\"required_is\"",
        "\"invalid_is\"",
        "\"status=invalid\"",
        "\"status=valid\"",
        "\"ui-gallery-form-submit-validation-semantics\"",
    ] {
        assert!(
            submit.contains(needle),
            "form submit-validation script should gate FormState semantics; missing `{needle}`",
        );
    }

    for needle in [
        "\"FRET_UI_GALLERY_START_SECTION\": \"Disabled Field\"",
        "\"ui-gallery-form-disabled-field-control\"",
        "\"ui-gallery-form-disabled-field-enabled-control\"",
        "\"disabled_is\"",
        "\"semantics_action_is\"",
        "\"action\": \"focus\"",
        "\"action\": \"set_value\"",
        "\"ui-gallery-form-disabled-field-action-state\"",
    ] {
        assert!(
            disabled.contains(needle),
            "form disabled-field script should gate field shell/control action-state ownership; missing `{needle}`",
        );
    }
}
