fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn field_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/field.rs");

    for needle in [
        "repo-ref/ui/apps/v4/content/docs/components/base/field.mdx",
        "repo-ref/ui/apps/v4/content/docs/components/radix/field.mdx",
        "repo-ref/ui/apps/v4/registry/bases/base/ui/field.tsx",
        "repo-ref/ui/apps/v4/registry/bases/radix/ui/field.tsx",
        "repo-ref/ui/apps/v4/examples/base/field-{demo,input,textarea,select,slider,fieldset,checkbox,radio,switch,choice-card,group,rtl,responsive}.tsx",
        "repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-{demo,input,textarea,select,slider,fieldset,checkbox,radio,switch,choice-card,group,rtl,responsive}.tsx",
        "repo-ref/base-ui/packages/react/src/field/index.parts.ts",
        "repo-ref/base-ui/packages/react/src/field/root/FieldRoot.tsx",
        "`repo-ref/primitives` does not ship a standalone generic `Field` primitive",
        "No extra generic root `compose()` / `asChild` / `children(...)` API is needed here",
        "docs/public-surface drift rather than a `fret-ui` mechanism bug",
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
fn field_diag_scripts_cover_docs_smoke_and_responsive_follow_up() {
    let docs_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/field/ui-gallery-field-docs-smoke.json"
    );
    let responsive_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/field/ui-gallery-field-responsive-orientation-container-md.json"
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
}
