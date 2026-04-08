fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn progress_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/progress.rs");

    for needle in [
        "Reference stack: shadcn Progress docs on the Radix and Base UI lanes, plus the default visual baseline.",
        "Secondary structure references: the shadcn radix/base registry variants, Radix Primitives Progress, and Base UI Progress.",
        "generic composable children / `compose()` API",
        "Base UI's `ProgressLabel` / `ProgressValue` children API",
        "`mirror_in_rtl(true)`",
    ] {
        assert!(
            source.contains(needle),
            "progress page should document source axes and the children-api decision; missing `{}`",
            needle
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![demo, usage, label, controlled, rtl, api_reference, notes]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "progress page should keep the docs-path sections before `API Reference` and `Notes`"
    );
}

#[test]
fn progress_snippets_stay_copyable_and_docs_aligned() {
    let usage = include_str!("../src/ui/snippets/progress/usage.rs");
    let demo = include_str!("../src/ui/snippets/progress/demo.rs");
    let label = include_str!("../src/ui/snippets/progress/label.rs");
    let rtl = include_str!("../src/ui/snippets/progress/rtl.rs");

    for needle in [
        "use fret::{UiChild, UiCx};",
        "use fret_ui_shadcn::facade as shadcn;",
        "shadcn::Progress::from_value(33.0)",
        ".a11y_label(\"Progress\")",
    ] {
        assert!(
            usage.contains(needle),
            "progress usage snippet should remain a complete copyable snapshot-value example; missing `{}`",
            needle
        );
    }

    for needle in [
        "let value = cx.local_model_keyed(\"value\", || 13.0);",
        "Duration::from_millis(500)",
        "*v = 66.0",
        ".refine_layout(LayoutRefinement::default().w_percent(60.0))",
    ] {
        assert!(
            demo.contains(needle),
            "progress demo snippet should stay aligned with the upstream 13 -> 66 timer demo; missing `{}`",
            needle
        );
    }

    let normalized_label = normalize_ws(label);
    assert!(
        normalized_label.contains(&normalize_ws(
            "shadcn::FieldLabel::new(\"Upload progress\") .test_id(\"ui-gallery-progress-label-title\")"
        )),
        "progress label snippet should keep the upstream label title surface visible",
    );
    assert!(
        normalized_label.contains(&normalize_ws(
            "shadcn::FieldLabel::new(\"66%\") .refine_layout(LayoutRefinement::default().ml_auto()) .test_id(\"ui-gallery-progress-label-value\")"
        )),
        "progress label snippet should keep the percentage value in the trailing auto-margin lane",
    );

    for needle in [
        "shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl)",
        ".mirror_in_rtl(true)",
        "\"٦٦%\"",
        "\"تقدم الرفع\"",
    ] {
        assert!(
            rtl.contains(needle),
            "progress RTL snippet should keep the caller-owned RTL mirror bridge and localized copy; missing `{}`",
            needle
        );
    }
    let normalized_rtl = normalize_ws(rtl);
    assert!(
        normalized_rtl.contains(&normalize_ws(
            "shadcn::FieldLabel::new(\"تقدم الرفع\") .refine_layout(LayoutRefinement::default().order(1)) .test_id(\"ui-gallery-progress-rtl-title\")"
        )),
        "progress RTL snippet should keep the localized label in source order while moving it to the visual inline end",
    );
    assert!(
        normalized_rtl.contains(&normalize_ws(
            "shadcn::FieldLabel::new(\"٦٦%\") .refine_layout(LayoutRefinement::default().order(0).mr_auto()) .test_id(\"ui-gallery-progress-rtl-value\")"
        )),
        "progress RTL snippet should keep the localized percentage on the visual inline start lane",
    );
}
