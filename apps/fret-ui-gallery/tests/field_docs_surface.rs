fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn field_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/field.rs");

    for needle in [
        "Reference stack: shadcn Field docs, base/radix field recipes, and the corresponding base/new-york example sets.",
        "Headless mechanism cross-check: Base UI defines a dedicated field/label/control/description/error split, while Radix Primitives does not ship a standalone generic `Field` primitive.",
        "`Field::new([...])` is the core wrapper for a single field; `orientation(...)` covers the documented `vertical`, `horizontal`, and the upstream `responsive` outcome via `FieldOrientation::ContainerAdaptive`.",
        "No extra generic root `compose()` / `asChild` / `children(...)` API is needed here",
        "docs/public-surface drift rather than a `fret-ui` mechanism bug",
        ".max_w(Px(980.0))",
        "Preview mirrors the upstream shadcn Field docs path first after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Anatomy`, `Form`, `Input`, `Textarea`, `Select`, `Slider`, `Fieldset`, `Checkbox`, `Radio`, `Switch`, `Choice Card`, `Field Group`, `RTL`, `Responsive Layout`, `Validation and Errors`, `Accessibility`, and `API Reference`.",
    ] {
        assert!(
            source.contains(needle),
            "field page should document source axes and the generic-children decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            anatomy,
            form,
            input,
            textarea,
            select,
            slider,
            fieldset,
            checkbox,
            radio,
            switch,
            choice_card,
            field_group,
            rtl,
            responsive,
            validation_and_errors,
            accessibility,
            api_reference,
            composable_label,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "field page should keep the full upstream docs path before the Fret-only composable-children follow-up",
    );
}

#[test]
fn field_snippets_keep_docs_path_examples_and_the_existing_wrapped_label_lane() {
    let usage = include_str!("../src/ui/snippets/field/usage.rs");
    let input = include_str!("../src/ui/snippets/field/input.rs");
    let select = include_str!("../src/ui/snippets/field/select.rs");
    let composable_label = include_str!("../src/ui/snippets/field/composable_label.rs");

    for needle in [
        "shadcn::FieldLegend::new(\"Profile\")",
        "shadcn::FieldDescription::new(\"This appears on invoices and emails.\")",
        "shadcn::Field::new([",
        "shadcn::Switch::new(newsletter)",
        ".orientation(shadcn::FieldOrientation::Horizontal)",
        ".test_id(\"ui-gallery-field-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "field usage snippet should keep the upstream profile example shape; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::FieldLabel::new(\"Password\")",
        "shadcn::FieldDescription::new(\"Must be at least 8 characters long.\")",
        "shadcn::Input::new(password)",
        ".password()",
        ".test_id(\"ui-gallery-field-input-password\")",
    ] {
        assert!(
            input.contains(needle),
            "field input snippet should keep the upstream password-field ordering; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::Field::build(|cx, out| {",
        "out.push_ui(cx, shadcn::FieldLabel::new(\"Department\"));",
        "shadcn::Select::new(value, open)",
        "shadcn::FieldDescription::new(\"Select your department or area of work.\")",
        ".test_id(\"ui-gallery-field-select\")",
    ] {
        assert!(
            select.contains(needle),
            "field select snippet should keep the field-local builder lane explicit; missing `{needle}`",
        );
    }

    for needle in [
        "FieldLabel::new(\"Require manual approval\")",
        ".wrap([shadcn::Field::new([",
        "shadcn::FieldContent::new([",
        "shadcn::FieldTitle::new(\"Require manual approval\")",
        "shadcn::Switch::new(manual_review)",
        ".test_id(\"ui-gallery-field-composable-label\")",
    ] {
        assert!(
            composable_label.contains(needle),
            "field composable-label snippet should keep the wrapped-label follow-up lane; missing `{needle}`",
        );
    }

    let combined = [usage, input, select, composable_label].join("\n");
    assert!(
        !combined.contains("compose()"),
        "field snippets should not introduce a generic compose() root lane",
    );
    assert!(
        !combined.contains("asChild"),
        "field snippets should not introduce an asChild-style root API",
    );
    assert!(
        !combined.contains(".children(["),
        "field snippets should stay on Field::new / Field::build / FieldLabel::wrap rather than teaching a generic root children API",
    );
}

#[test]
fn field_demo_teaches_label_control_relations_without_direct_label_shadowing() {
    let demo = include_str!("../src/ui/snippets/field/demo.rs");

    for needle in [
        ".test_id(\"ui-gallery-field-demo-card-name-label\")",
        ".test_id(\"ui-gallery-field-demo-card-name\")",
        ".test_id(\"ui-gallery-field-demo-card-number-label\")",
        ".test_id(\"ui-gallery-field-demo-card-number\")",
        ".test_id(\"ui-gallery-field-demo-same-as-shipping-label\")",
        ".test_id(\"ui-gallery-field-demo-same-as-shipping\")",
        ".test_id(\"ui-gallery-field-demo-comments-label\")",
        ".test_id(\"ui-gallery-field-demo-comments\")",
        ".for_control(\"ui-gallery-field-demo-card-name\")",
        ".control_id(\"ui-gallery-field-demo-card-name\")",
        ".for_control(\"ui-gallery-field-demo-same-as-shipping\")",
        ".control_id(\"ui-gallery-field-demo-same-as-shipping\")",
        ".for_control(\"ui-gallery-field-demo-comments\")",
        ".control_id(\"ui-gallery-field-demo-comments\")",
    ] {
        assert!(
            demo.contains(needle),
            "field demo should keep explicit label-control wiring observable; missing `{needle}`",
        );
    }

    for stale_direct_label in [
        ".a11y_label(\"Name on Card\")",
        ".a11y_label(\"Card Number\")",
        ".a11y_label(\"CVV\")",
        ".a11y_label(\"Same as shipping address\")",
        ".a11y_label(\"Comments\")",
    ] {
        assert!(
            !demo.contains(stale_direct_label),
            "field demo controls should derive their accessible relation from FieldLabel::for_control instead of shadowing it with `{stale_direct_label}`",
        );
    }
}

#[test]
fn field_diag_scripts_cover_docs_smoke_and_responsive_follow_up() {
    let docs_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/field/ui-gallery-field-docs-smoke.json"
    );
    let responsive_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/field/ui-gallery-field-responsive-orientation-container-md.json"
    );
    let relation_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/field/ui-gallery-field-demo-label-control-action-state.json"
    );

    for needle in [
        "\"ui-gallery-page-field\"",
        "\"ui-gallery-field-demo\"",
        "\"ui-gallery-field-usage-tabs-trigger-preview\"",
        "\"ui-gallery-field-anatomy-tabs-trigger-preview\"",
        "\"ui-gallery-field-input\"",
        "\"ui-gallery-field-choice-card\"",
        "\"ui-gallery-field-composable-label\"",
        "\"ui-gallery-field-api-reference-content\"",
        "\"ui-gallery-field-docs-smoke\"",
    ] {
        assert!(
            docs_script.contains(needle),
            "field docs smoke script should cover the primary page sections and the post-docs follow-up anchor; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-field-responsive-width-switch\"",
        "\"ui-gallery-field-responsive-name-content\"",
        "\"ui-gallery-field-responsive-name-input\"",
        "\"ui-gallery-field-responsive-orientation-container-md\"",
    ] {
        assert!(
            responsive_script.contains(needle),
            "field responsive diag script should keep the container-width follow-up gate; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-field-demo-label-control-action-state\"",
        "\"ui-gallery-field-demo-card-name-label\"",
        "\"ui-gallery-field-demo-card-name\"",
        "\"ui-gallery-field-demo-same-as-shipping-label\"",
        "\"ui-gallery-field-demo-same-as-shipping\"",
        "\"ui-gallery-field-demo-comments-label\"",
        "\"ui-gallery-field-demo-comments\"",
        "\"semantics_relation_includes\"",
        "\"checked_state_is\"",
        "\"focus_is\"",
    ] {
        assert!(
            relation_script.contains(needle),
            "field relation/action-state script should gate label-control semantics; missing `{needle}`",
        );
    }
}
