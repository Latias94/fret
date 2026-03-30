fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn hover_card_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/hover_card.rs");

    for needle in [
        "repo-ref/ui/apps/v4/content/docs/components/base/hover-card.mdx",
        "repo-ref/ui/apps/v4/content/docs/components/radix/hover-card.mdx",
        "repo-ref/ui/apps/v4/registry/new-york-v4/ui/hover-card.tsx",
        "repo-ref/primitives/packages/react/hover-card/src/hover-card.tsx",
        "repo-ref/base-ui/packages/react/src/preview-card/root/PreviewCardRoot.tsx",
        "`HoverCard::new(cx, trigger, content)` remains the recipe-level entry point and already covers the upstream nested `<HoverCard><HoverCardTrigger /><HoverCardContent /></HoverCard>` composition plus the custom-trigger / `asChild` story, because `trigger` can be any landed or late-landed element.",
        "Hover-card behavior itself is already covered by the existing Radix/web geometry, chrome, and UI Gallery interaction gates; the remaining work here is docs/public-surface alignment rather than a `fret-ui` mechanism bug.",
        "Preview now mirrors the shadcn Hover Card docs path directly: `Demo`, `Usage`, `Trigger Delays`, `Positioning`, `Basic`, `Sides`, `RTL`, and `API Reference`. `Children (Fret)` and `Notes` stay as the explicit follow-ups.",
        ".test_id_prefix(\"ui-gallery-hover-card-demo-section\")",
        ".test_id_prefix(\"ui-gallery-hover-card-usage-section\")",
        ".test_id_prefix(\"ui-gallery-hover-card-trigger-delays-section\")",
        ".test_id_prefix(\"ui-gallery-hover-card-positioning-section\")",
        ".test_id_prefix(\"ui-gallery-hover-card-basic-section\")",
        ".test_id_prefix(\"ui-gallery-hover-card-sides-section\")",
        ".test_id_prefix(\"ui-gallery-hover-card-rtl-section\")",
    ] {
        assert!(
            source.contains(needle),
            "hover_card page should document the source axes and children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            trigger_delays,
            positioning,
            basic,
            sides,
            rtl,
            api_reference,
            children,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "hover_card page should keep the docs-path sections before the Fret follow-ups",
    );
}

#[test]
fn hover_card_snippets_stay_copyable_and_docs_aligned() {
    let demo = include_str!("../src/ui/snippets/hover_card/demo.rs");
    let usage = include_str!("../src/ui/snippets/hover_card/usage.rs");
    let trigger_delays = include_str!("../src/ui/snippets/hover_card/trigger_delays.rs");
    let children = include_str!("../src/ui/snippets/hover_card/children.rs");

    for needle in [
        "use fret::{UiChild, UiCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "shadcn::HoverCard::new(",
        "shadcn::HoverCardTrigger::build(ui::raw_text(\"Hover\"))",
        "shadcn::HoverCardContent::build(cx, |cx| {",
    ] {
        assert!(
            usage.contains(needle),
            "hover_card usage snippet should remain a complete copyable docs-path example; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::Button::new(\"@nextjs\")",
        ".variant(shadcn::ButtonVariant::Link)",
        ".refine_layout(LayoutRefinement::default().max_w(Px(320.0)))",
        ".test_id(\"ui-gallery-hover-card-demo-content\")",
    ] {
        assert!(
            demo.contains(needle),
            "hover_card demo snippet should keep the upstream-shaped demo surface and stable selectors; missing `{needle}`",
        );
    }

    for needle in [
        ".open_delay_frames(0)",
        ".close_delay_frames(0)",
        ".open_delay(Duration::from_millis(700))",
        ".close_delay(Duration::from_millis(300))",
    ] {
        assert!(
            trigger_delays.contains(needle),
            "hover_card trigger-delays snippet should keep the root-owned timing surface visible; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::HoverCardContent::new([title, summary, meta])",
        ".test_id(\"ui-gallery-hover-card-children-demo-content\")",
        ".open_delay_frames(8)",
        ".close_delay_frames(8)",
    ] {
        assert!(
            children.contains(needle),
            "hover_card children snippet should remain the focused content-slot follow-up; missing `{needle}`",
        );
    }

    let combined = [demo, usage, trigger_delays, children].join("\n");
    assert!(
        !combined.contains("compose()"),
        "hover_card snippets should keep the default teaching lane on `HoverCard::new(...)` instead of inventing a compose() surface",
    );
}

#[test]
fn hover_card_docs_diag_script_covers_docs_path_and_follow_ups() {
    let docs_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/hover-card/ui-gallery-hover-card-docs-smoke.json"
    );
    let docs_stub =
        include_str!("../../../tools/diag-scripts/ui-gallery-hover-card-docs-smoke.json");
    let docs_suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-hover-card/suite.json");

    for needle in [
        "\"ui-gallery-hover-card-demo-section-content\"",
        "\"ui-gallery-hover-card-usage-section-content\"",
        "\"ui-gallery-hover-card-trigger-delays-section-content\"",
        "\"ui-gallery-hover-card-positioning-section-content\"",
        "\"ui-gallery-hover-card-basic-section-content\"",
        "\"ui-gallery-hover-card-sides-section-content\"",
        "\"ui-gallery-hover-card-rtl-section-content\"",
        "\"ui-gallery-hover-card-api-reference-content\"",
        "\"ui-gallery-hover-card-children-title\"",
        "\"ui-gallery-hover-card-children-content\"",
        "\"ui-gallery-hover-card-notes-content\"",
        "\"ui-gallery-hover-card-docs-smoke\"",
    ] {
        assert!(
            docs_script.contains(needle),
            "hover_card docs diag script should cover the docs path and the Fret follow-ups; missing `{needle}`",
        );
    }

    assert!(
        docs_stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/hover-card/ui-gallery-hover-card-docs-smoke.json\""
        ),
        "hover_card docs smoke top-level redirect stub should point at the canonical hover-card script",
    );
    assert!(
        docs_suite.contains(
            "\"tools/diag-scripts/ui-gallery/hover-card/ui-gallery-hover-card-docs-smoke.json\""
        ),
        "hover_card docs smoke script should stay promoted in the dedicated hover-card suite",
    );
}
