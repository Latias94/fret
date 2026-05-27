fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn textarea_page_documents_source_axes_and_leaf_children_api_decision() {
    let source = include_str!("../src/ui/pages/textarea.rs");

    for needle in [
        "Reference stack: current shadcn Textarea docs, the new-york-v4 registry recipe, tracked new-york examples, and base/radix examples for Field follow-ups.",
        "Neither Radix Primitives nor Base UI defines a dedicated Textarea primitive or compound children contract",
        "did not identify a missing `fret-ui` mechanism bug",
        "No extra generic `compose()` / `asChild` / children API is needed here",
        "Preview mirrors the current upstream Textarea docs path first after collapsing the top `ComponentPreview` / `Default` duplicate into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Disabled`, `With Label`, `With Text`, and `With Button`.",
    ] {
        assert!(
            source.contains(needle),
            "textarea page should document source axes and the leaf-surface decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            disabled,
            with_label,
            with_text,
            button,
            api_reference,
            invalid,
            required,
            rtl,
            field,
            label,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "textarea page should keep the docs-path sections before the Fret follow-ups",
    );
}

#[test]
fn textarea_snippets_keep_the_docs_path_examples_and_leaf_surface() {
    let field = include_str!("../src/ui/snippets/textarea/field.rs");
    let with_label = include_str!("../src/ui/snippets/textarea/with_label.rs");
    let rtl = include_str!("../src/ui/snippets/textarea/rtl.rs");
    let usage = include_str!("../src/ui/snippets/textarea/usage.rs");
    let follow_ups = [
        include_str!("../src/ui/snippets/textarea/button.rs"),
        include_str!("../src/ui/snippets/textarea/label.rs"),
        include_str!("../src/ui/snippets/textarea/required.rs"),
        include_str!("../src/ui/snippets/textarea/with_text.rs"),
    ]
    .join("\n");

    for needle in [
        "shadcn::FieldLabel::new(\"Message\")",
        "shadcn::FieldDescription::new(\"Type your message and press enter to send.\")",
        ".placeholder(\"Type your message here.\")",
        ".rows(6)",
        ".test_id(\"ui-gallery-textarea-field\")",
    ] {
        assert!(
            field.contains(needle),
            "textarea field snippet should mirror the base/radix Field follow-up example; missing `{needle}`",
        );
    }
    let label_ix = field
        .find("shadcn::FieldLabel::new(\"Message\")")
        .expect("field label");
    let description_ix = field
        .find("shadcn::FieldDescription::new(\"Type your message and press enter to send.\")")
        .expect("field description");
    let textarea_ix = field
        .find("shadcn::Textarea::new(value)")
        .expect("field textarea");
    assert!(
        label_ix < textarea_ix && textarea_ix < description_ix,
        "textarea field snippet should keep the base/radix label -> control -> description order",
    );
    assert!(
        !field.contains(".rows(4)"),
        "textarea field snippet should not reuse the RTL four-row variant",
    );
    assert!(
        !field.contains("Feedback"),
        "textarea field snippet should not reuse the RTL feedback copy",
    );

    for needle in [
        "with_direction_provider",
        "shadcn::FieldLabel::new(\"التعليقات\")",
        ".rows(4)",
        "شاركنا أفكارك حول خدمتنا.",
    ] {
        assert!(
            rtl.contains(needle),
            "textarea RTL snippet should keep the translated feedback example; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::Label::new(\"Your message\")",
        ".for_control(id.clone())",
        "shadcn::Textarea::new(value)",
        ".control_id(id)",
        ".test_id(\"ui-gallery-textarea-with-label\")",
    ] {
        assert!(
            with_label.contains(needle),
            "textarea With Label snippet should mirror the current upstream label example; missing `{needle}`",
        );
    }

    for needle in [
        "use fret::{AppComponentCx, UiChild};",
        "shadcn::Textarea::new(value)",
        ".a11y_label(\"Message\")",
        ".placeholder(\"Type your message here.\")",
    ] {
        assert!(
            usage.contains(needle),
            "textarea usage snippet should remain a complete copyable leaf example; missing `{needle}`",
        );
    }

    let combined = [field, with_label, rtl, usage, &follow_ups].join("\n");
    assert!(
        !combined.contains(".children(["),
        "textarea snippets should not widen the leaf control into a generic children API",
    );
    assert!(
        !combined.contains("compose()"),
        "textarea snippets should stay on the leaf-control surface instead of inventing a compose() lane",
    );
    assert!(
        !combined.contains("asChild"),
        "textarea snippets should not teach an asChild-style root surface",
    );
}

#[test]
fn textarea_diag_scripts_cover_docs_path_and_label_follow_up() {
    let docs_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/textarea/ui-gallery-textarea-docs-screenshot.json"
    );
    let label_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/textarea/ui-gallery-textarea-label-click-focus.json"
    );

    for needle in [
        "\"ui-gallery-textarea-demo\"",
        "\"ui-gallery-textarea-with-label\"",
        "\"ui-gallery-textarea-with-text\"",
        "\"ui-gallery-textarea-button\"",
        "\"ui-gallery-textarea-api-reference-content\"",
        "\"ui-gallery-textarea-field\"",
        "\"ui-gallery-textarea-disabled\"",
        "\"ui-gallery-textarea-invalid\"",
        "\"ui-gallery-textarea-required\"",
        "\"ui-gallery-textarea-rtl\"",
        "\"ui-gallery-textarea-label-content\"",
        "\"ui-gallery-textarea-docs-screenshot\"",
    ] {
        assert!(
            docs_script.contains(needle),
            "textarea docs screenshot script should cover the docs path plus the API Reference / Label Association follow-ups; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-textarea-label-label\"",
        "\"ui-gallery-textarea-label-control\"",
        "\"ui-gallery-textarea-label-click-focus\"",
    ] {
        assert!(
            label_script.contains(needle),
            "textarea label diag script should cover the explicit label-association follow-up; missing `{needle}`",
        );
    }
}
