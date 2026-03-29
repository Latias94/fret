fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn collapsible_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/collapsible.rs");

    for needle in [
        "repo-ref/ui/apps/v4/content/docs/components/base/collapsible.mdx",
        "repo-ref/ui/apps/v4/registry/new-york-v4/ui/collapsible.tsx",
        "repo-ref/ui/apps/v4/registry/new-york-v4/examples/collapsible-demo.tsx",
        "repo-ref/ui/apps/v4/examples/base/{collapsible-basic,collapsible-settings,collapsible-file-tree,collapsible-rtl}.tsx",
        "repo-ref/primitives/packages/react/collapsible/src/collapsible.tsx",
        "repo-ref/base-ui/packages/react/src/collapsible/root/CollapsibleRoot.tsx",
        "repo-ref/base-ui/packages/react/src/collapsible/trigger/CollapsibleTrigger.tsx",
        "repo-ref/base-ui/packages/react/src/collapsible/panel/CollapsiblePanel.tsx",
        "`shadcn::Collapsible` remains the compact Fret-first builder for the common trigger/content lane, while `shadcn::CollapsibleRoot`, `shadcn::CollapsibleTriggerPart`, and `shadcn::CollapsibleContentPart` cover the copyable composable children lane on the curated facade.",
        "A broader generic `Collapsible::children([...])` / `compose()` root API is not currently warranted here",
        "docs/public-surface alignment rather than a `fret-ui` mechanism bug",
        "Preview mirrors the shadcn/base Collapsible docs path first after skipping `Installation`: `Demo`, `Usage`, `Controlled State`, `Basic`, `Settings Panel`, `File Tree`, `RTL`, and `API Reference`.",
        ".test_id_prefix(\"ui-gallery-collapsible-demo\")",
        ".test_id_prefix(\"ui-gallery-collapsible-usage\")",
        ".test_id_prefix(\"ui-gallery-collapsible-controlled\")",
        ".test_id_prefix(\"ui-gallery-collapsible-basic\")",
        ".test_id_prefix(\"ui-gallery-collapsible-settings\")",
        ".test_id_prefix(\"ui-gallery-collapsible-file-tree\")",
        ".test_id_prefix(\"ui-gallery-collapsible-rtl\")",
        ".test_id_prefix(\"ui-gallery-collapsible-api-reference\")",
        ".test_id_prefix(\"ui-gallery-collapsible-notes\")",
    ] {
        assert!(
            source.contains(needle),
            "collapsible page should document the source axes and children-api decision; missing `{needle}`"
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            controlled_state,
            basic,
            settings,
            file_tree,
            rtl,
            api_reference,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "collapsible page should keep the docs-path sections before the notes follow-up"
    );
}

#[test]
fn collapsible_docs_path_snippets_stay_copyable_and_docs_aligned() {
    let usage = include_str!("../src/ui/snippets/collapsible/usage.rs");
    let demo = include_str!("../src/ui/snippets/collapsible/demo.rs");
    let basic = include_str!("../src/ui/snippets/collapsible/basic.rs");
    let settings = include_str!("../src/ui/snippets/collapsible/settings_panel.rs");
    let rtl = include_str!("../src/ui/snippets/collapsible/rtl.rs");

    for needle in [
        "use fret::{UiChild, UiCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "shadcn::CollapsibleRoot::new().into_element(cx, |cx| {",
        "shadcn::CollapsibleTriggerPart::new([",
        "shadcn::CollapsibleContentPart::new([",
        "\"Can I use this in my project?\"",
    ] {
        assert!(
            usage.contains(needle),
            "collapsible usage snippet should remain a complete copyable composable example; missing `{needle}`"
        );
    }

    for needle in [
        "@peduarte starred 3 repositories",
        "shadcn::CollapsibleTriggerPart::new([button])",
        ".as_child(true)",
        "\"ui-gallery-collapsible-demo-trigger\"",
        "\"ui-gallery-collapsible-demo-repo-colors\"",
    ] {
        assert!(
            demo.contains(needle),
            "collapsible demo snippet should keep the upstream-shaped repository-list surface; missing `{needle}`"
        );
    }

    for needle in [
        "\"Product details\"",
        "\"Learn More\"",
        "shadcn::ButtonSize::Xs",
        "\"ui-gallery-collapsible-basic-trigger\"",
        "\"ui-gallery-collapsible-basic-content\"",
    ] {
        assert!(
            basic.contains(needle),
            "collapsible basic snippet should keep the current docs-aligned CTA surface; missing `{needle}`"
        );
    }

    for needle in [
        "use shadcn::raw::collapsible::primitives as shadcn_col;",
        "String::from(\"0\")",
        "\"ui-gallery-collapsible-settings-trigger\"",
        "\"ui-gallery-collapsible-settings-content\"",
    ] {
        assert!(
            settings.contains(needle),
            "collapsible settings snippet should keep the source-aligned primitive lane explicit; missing `{needle}`"
        );
    }

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl, |cx| {",
        "\"الطلب #4189\"",
        "\"ui-gallery-collapsible-rtl-trigger\"",
        "\"ui-gallery-collapsible-rtl-content\"",
    ] {
        assert!(
            rtl.contains(needle),
            "collapsible rtl snippet should keep the readable RTL example and stable selectors; missing `{needle}`"
        );
    }

    let combined = [usage, demo, basic, settings, rtl].join("\n");
    assert!(
        !combined.contains("Collapsible::children("),
        "collapsible docs-path snippets should not widen the root surface into a generic children API",
    );
    assert!(
        !combined.contains("compose()"),
        "collapsible docs-path snippets should keep `compose()` out of the default teaching lane",
    );
}

#[test]
fn collapsible_docs_diag_scripts_cover_docs_smoke_and_existing_notes_follow_ups() {
    let docs_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/collapsible/ui-gallery-collapsible-docs-smoke.json"
    );
    let docs_stub =
        include_str!("../../../tools/diag-scripts/ui-gallery-collapsible-docs-smoke.json");
    let docs_suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json");
    let notes_scroll = include_str!(
        "../../../tools/diag-scripts/ui-gallery/collapsible/ui-gallery-collapsible-rtl-code-tab-scroll-range.json"
    );
    let notes_screenshot = include_str!(
        "../../../tools/diag-scripts/ui-gallery/collapsible/ui-gallery-collapsible-notes-bottom-screenshot.json"
    );

    for needle in [
        "\"ui-gallery-page-collapsible\"",
        "\"ui-gallery-collapsible-demo\"",
        "\"ui-gallery-collapsible-usage-content\"",
        "\"ui-gallery-collapsible-controlled\"",
        "\"ui-gallery-collapsible-basic\"",
        "\"ui-gallery-collapsible-settings-panel\"",
        "\"ui-gallery-collapsible-file-tree\"",
        "\"ui-gallery-collapsible-rtl-demo\"",
        "\"ui-gallery-collapsible-api-reference-content\"",
        "\"ui-gallery-collapsible-notes-content\"",
        "\"ui-gallery-collapsible-docs-smoke\"",
    ] {
        assert!(
            docs_script.contains(needle),
            "collapsible docs smoke script should cover the docs path plus notes; missing `{needle}`"
        );
    }

    assert!(
        docs_stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/collapsible/ui-gallery-collapsible-docs-smoke.json\""
        ),
        "collapsible docs smoke stub should redirect to the canonical docs smoke script",
    );
    assert!(
        docs_suite.contains("\"tools/diag-scripts/ui-gallery-collapsible-docs-smoke.json\""),
        "shadcn conformance suite should include the collapsible docs smoke gate",
    );

    for needle in [
        "\"ui-gallery-collapsible-notes\"",
        "\"docsec-rtl-tabs-trigger-code\"",
        "\"ui-gallery-collapsible-rtl-code-tab-scroll-range\"",
    ] {
        assert!(
            notes_scroll.contains(needle),
            "collapsible RTL code-tab gate should keep the notes section anchors stable; missing `{needle}`"
        );
    }

    for needle in [
        "\"ui-gallery-collapsible-notes\"",
        "\"ui-gallery-collapsible-notes-bottom\"",
    ] {
        assert!(
            notes_screenshot.contains(needle),
            "collapsible notes screenshot gate should keep the notes section anchors stable; missing `{needle}`"
        );
    }
}
