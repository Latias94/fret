fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn progress_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/progress.rs");

    for needle in [
        "repo-ref/ui/apps/v4/content/docs/components/radix/progress.mdx",
        "repo-ref/ui/apps/v4/content/docs/components/base/progress.mdx",
        "repo-ref/ui/apps/v4/registry/new-york-v4/ui/progress.tsx",
        "repo-ref/ui/apps/v4/registry/bases/radix/ui/progress.tsx",
        "repo-ref/ui/apps/v4/registry/bases/base/ui/progress.tsx",
        "repo-ref/primitives/packages/react/progress/src/progress.tsx",
        "repo-ref/base-ui/packages/react/src/progress/*",
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
}
