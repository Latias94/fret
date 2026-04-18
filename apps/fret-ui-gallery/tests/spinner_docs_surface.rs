fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn spinner_page_documents_source_axes_and_leaf_api_decision() {
    let source = include_str!("../src/ui/pages/spinner.rs");

    for needle in [
        "Reference stack: shadcn Spinner docs, the default registry recipe, the demo/custom/size/button/badge/input-group/empty examples, and the RTL example.",
        "Secondary shadcn recipe references: the base and radix registry variants plus their spinner examples all keep Spinner as a leaf `svg` recipe plus host-owned slot composition.",
        "leaf `svg` recipe plus host-owned slot composition",
        "Neither Radix Primitives nor Base UI defines a dedicated Spinner primitive",
        "did not identify a missing `fret-ui` mechanism bug",
        "no extra generic `compose()` or composable children API is needed here",
        "Preview mirrors the shadcn Spinner docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`",
        "DocSection::build(cx, \"Extras\", extras)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
    ] {
        assert!(
            source.contains(needle),
            "spinner page should document source axes and the leaf-API decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            customization,
            sizes,
            buttons,
            badges,
            input_group,
            empty,
            rtl,
            extras,
            api_reference,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "spinner page should keep the docs-path sections before the Fret-only `Extras` / `API Reference` follow-ups",
    );
}

#[test]
fn spinner_snippets_stay_copyable_and_docs_aligned() {
    let usage = include_str!("../src/ui/snippets/spinner/usage.rs");
    let customization = include_str!("../src/ui/snippets/spinner/customization.rs");
    let sizes = include_str!("../src/ui/snippets/spinner/sizes.rs");
    let buttons = include_str!("../src/ui/snippets/spinner/buttons.rs");
    let input_group = include_str!("../src/ui/snippets/spinner/input_group.rs");

    for needle in [
        "use fret::{UiChild, AppComponentCx};",
        "use fret_ui_shadcn::facade as shadcn;",
        "shadcn::Spinner::new()",
        ".test_id(\"ui-gallery-spinner-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "spinner usage snippet should remain a complete copyable leaf example; missing `{needle}`",
        );
    }

    for needle in [
        "fn project_spinner() -> shadcn::Spinner",
        ".icon(fret_icons::ids::ui::SETTINGS)",
    ] {
        assert!(
            customization.contains(needle),
            "spinner customization snippet should keep the local-wrapper customization story copyable; missing `{needle}`",
        );
    }
    assert!(
        normalize_ws(customization).contains(&normalize_ws("project_spinner().into_element(cx)")),
        "spinner customization snippet should render the project-local spinner directly",
    );
    assert!(
        !customization.contains("shadcn::Spinner::new().into_element(cx)"),
        "spinner customization snippet should mirror the upstream single-custom-spinner preview instead of a before/after row",
    );

    for needle in [
        ".w_px(Px(12.0)).h_px(Px(12.0))",
        ".w_px(Px(24.0)).h_px(Px(24.0))",
        ".w_px(Px(32.0)).h_px(Px(32.0))",
        ".test_id(\"ui-gallery-spinner-sizes-8\")",
    ] {
        assert!(
            sizes.contains(needle),
            "spinner size snippet should keep the upstream size-3/4/6/8 lane copyable; missing `{needle}`",
        );
    }

    for needle in [
        ".disabled(true)",
        ".size(shadcn::ButtonSize::Sm)",
        ".leading_children([shadcn::Spinner::new().into_element(cx)])",
        "button(cx, \"Processing\", Some(shadcn::ButtonVariant::Secondary))",
    ] {
        assert!(
            buttons.contains(needle),
            "spinner button snippet should keep the disabled button loading examples copyable; missing `{needle}`",
        );
    }

    for needle in [
        ".trailing([shadcn::Spinner::new().into_element(cx)])",
        "\"Validating...\"",
        ".block_end([actions])",
        "ui-gallery-spinner-extras-textarea-actions",
    ] {
        assert!(
            input_group.contains(needle),
            "spinner input-group snippet should keep the inline-end and block-end examples copyable; missing `{needle}`",
        );
    }

    let combined = [usage, customization, sizes, buttons, input_group].join("\n");
    assert!(
        !combined.contains(".children(["),
        "spinner docs snippets should not widen the leaf surface into a generic children API",
    );
    assert!(
        !combined.contains("compose()"),
        "spinner docs snippets should stay on the leaf + host-slot composition lane",
    );

    let rtl = include_str!("../src/ui/snippets/spinner/rtl.rs");
    for needle in [
        "جاري معالجة الدفع...",
        "١٠٠.٠٠ دولار",
        "with_direction_provider",
    ] {
        assert!(
            rtl.contains(needle),
            "spinner rtl snippet should mirror the upstream RTL content lane more closely; missing `{needle}`",
        );
    }
    assert!(
        !rtl.contains("Processing payment..."),
        "spinner rtl snippet should not fall back to the LTR English copy once the RTL example is aligned",
    );
}

#[test]
fn spinner_docs_diag_script_covers_docs_path_and_follow_ups() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/spinner/ui-gallery-spinner-docs-screenshots.json"
    );

    for needle in [
        "\"ui-gallery-spinner-demo-content\"",
        "\"ui-gallery-spinner-usage-content\"",
        "\"ui-gallery-spinner-customization-content\"",
        "\"ui-gallery-spinner-size-content\"",
        "\"ui-gallery-spinner-button-content\"",
        "\"ui-gallery-spinner-badge-content\"",
        "\"ui-gallery-spinner-input-group-content\"",
        "\"ui-gallery-spinner-empty-content\"",
        "\"ui-gallery-spinner-rtl-content\"",
        "\"ui-gallery-spinner-extras-content\"",
        "\"ui-gallery-spinner-api-reference-content\"",
        "\"ui-gallery-spinner-docs.11-api-reference\"",
    ] {
        assert!(
            script.contains(needle),
            "spinner docs diag script should cover the docs path and Fret follow-ups; missing `{needle}`",
        );
    }
}
