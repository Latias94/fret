fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn skeleton_page_documents_source_axes_and_leaf_api_decision() {
    let source = include_str!("../src/ui/pages/skeleton.rs");

    for needle in [
        "repo-ref/ui/apps/v4/content/docs/components/base/skeleton.mdx",
        "repo-ref/ui/apps/v4/content/docs/components/radix/skeleton.mdx",
        "repo-ref/ui/apps/v4/registry/{new-york-v4,bases/base,bases/radix}/ui/skeleton.tsx",
        "Neither `repo-ref/primitives` nor `repo-ref/base-ui` defines a dedicated Skeleton primitive",
        "no extra generic `compose()` or composable children API is needed here",
        "Preview mirrors the shadcn Skeleton docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Notes\", notes)",
    ] {
        assert!(
            source.contains(needle),
            "skeleton page should document source axes and the leaf-primitive API decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            avatar,
            card,
            text_section,
            form,
            table,
            rtl,
            api_reference,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "skeleton page should keep the docs-path sections before the Fret-only `API Reference` and `Notes` follow-ups",
    );
}

#[test]
fn skeleton_snippets_stay_copyable_and_docs_aligned() {
    let usage = include_str!("../src/ui/snippets/skeleton/usage.rs");
    let demo = include_str!("../src/ui/snippets/skeleton/demo.rs");
    let card = include_str!("../src/ui/snippets/skeleton/card.rs");

    for needle in [
        "use fret::{UiChild, UiCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "shadcn::Skeleton::new()",
        ".refine_style(ChromeRefinement::default().rounded(Radius::Full))",
        ".h_px(Px(20.0))",
        ".w_px(Px(100.0))",
        ".test_id(\"ui-gallery-skeleton-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "skeleton usage snippet should remain a complete copyable docs-path example; missing `{needle}`",
        );
    }

    for needle in [
        "use fret::{UiChild, UiCx};",
        ".w_px(Px(48.0))",
        ".w_px(Px(250.0))",
        ".w_px(Px(200.0))",
        ".test_id(\"ui-gallery-skeleton-demo\")",
    ] {
        assert!(
            demo.contains(needle),
            "skeleton demo snippet should stay aligned with the upstream top preview while remaining copyable; missing `{needle}`",
        );
    }

    for needle in [
        "use fret::{UiChild, UiCx};",
        "shadcn::card(",
        "shadcn::card_header(",
        "shadcn::card_content(",
        ".w_fraction(2.0 / 3.0)",
        ".aspect_ratio(1.0)",
        ".max_w(Px(320.0))",
        ".test_id(\"ui-gallery-skeleton-card\")",
    ] {
        assert!(
            card.contains(needle),
            "skeleton card snippet should remain a complete copyable card example; missing `{needle}`",
        );
    }
}

#[test]
fn skeleton_docs_diag_script_covers_docs_path_and_notes_follow_up() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/skeleton/ui-gallery-skeleton-docs-smoke.json"
    );

    for needle in [
        "\"ui-gallery-skeleton-demo-content\"",
        "\"ui-gallery-skeleton-usage-content\"",
        "\"ui-gallery-skeleton-avatar-content\"",
        "\"ui-gallery-skeleton-card-content\"",
        "\"ui-gallery-skeleton-text-content\"",
        "\"ui-gallery-skeleton-form-content\"",
        "\"ui-gallery-skeleton-table-content\"",
        "\"ui-gallery-skeleton-rtl-content\"",
        "\"ui-gallery-skeleton-api-reference-content\"",
        "\"ui-gallery-skeleton-notes-content\"",
        "\"ui-gallery-skeleton-docs-smoke\"",
    ] {
        assert!(
            script.contains(needle),
            "skeleton docs diag script should cover the docs path and the Fret-only notes follow-up; missing `{needle}`",
        );
    }
}
