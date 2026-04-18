fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn alert_page_keeps_docs_path_before_fret_extras_and_documents_children_api_decision() {
    let source = include_str!("../src/ui/pages/alert.rs");

    for needle in [
        "`Alert::new([...])` and `Alert::build(...)` cover the default root composition lane.",
        "`AlertAction::build(...)` is the preferred typed slot surface for top-end actions; `AlertAction::new([...])` remains valid when the action subtree is already landed.",
        "`AlertTitle::new(...)` keeps the compact title lane, while `AlertTitle::new_children(...)` and `AlertTitle::build(...)` cover attributed or precomposed title content.",
        "`AlertDescription::new(...)` keeps the plain-text lane, while `AlertDescription::new_children(...)` and `AlertDescription::build(...)` cover multi-paragraph or composed description content.",
        "This audit lands on the recipe/docs axis, not the runtime mechanism axis: `Alert` is a static callout and does not need new `fret-ui` substrate work.",
        "The `Demo` section keeps the upstream three-row `alert-demo` surface intact; richer title/description/link teaching stays under explicit `Fret Extras` sections instead of being mixed into the docs path.",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Fret Extras\", extras)",
        "DocSection::build(cx, \"Rich Title\", rich_title)",
        "DocSection::build(cx, \"Rich Description\", rich_description)",
        "DocSection::build(cx, \"Interactive Links\", interactive_links)",
    ] {
        assert!(
            source.contains(needle),
            "alert page should document ownership and children-api decisions; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            basic,
            destructive,
            action,
            custom_colors,
            rtl,
            api_reference,
            extras,
            rich_title,
            rich_description,
            interactive_links,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "alert page should keep the docs-path sections before explicit Fret follow-ups",
    );
}

#[test]
fn alert_demo_snippet_matches_upstream_docs_surface_and_keeps_link_logic_in_extras() {
    let demo = include_str!("../src/ui/snippets/alert/demo.rs");
    let interactive_links = include_str!("../src/ui/snippets/alert/interactive_links.rs");
    let rich_title = include_str!("../src/ui/snippets/alert/rich_title.rs");
    let rich_description = include_str!("../src/ui/snippets/alert/rich_description.rs");
    let docs_smoke = include_str!(
        "../../../tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-docs-smoke.json"
    );

    for needle in [
        "use fret::{UiChild, AppComponentCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "fn bullet_row(text: &'static str) -> impl UiChild + use<>",
        "shadcn::AlertTitle::new(\"Success! Your changes have been saved\")",
        "shadcn::AlertTitle::new(\"This Alert has a title and an icon. No description.\")",
        "shadcn::AlertTitle::new(\"Unable to process your payment.\")",
        "shadcn::AlertDescription::build(|cx, out| {",
        "bullet_row(\"Check your card details\")",
        "bullet_row(\"Ensure sufficient funds\")",
        "bullet_row(\"Verify billing address\")",
        ".variant(shadcn::AlertVariant::Destructive)",
        ".test_id(\"ui-gallery-alert-demo-success\")",
        ".test_id(\"ui-gallery-alert-demo-title-only\")",
        ".test_id(\"ui-gallery-alert-demo-payment-error\")",
    ] {
        assert!(
            normalize_ws(demo).contains(&normalize_ws(needle)),
            "alert demo snippet should stay on the upstream docs-shaped surface; missing `{needle}`",
        );
    }

    for needle in [
        "fn interactive_link<H: UiHost + 'static>(",
        "Effect::OpenUrl",
        "\"ui-gallery-alert-link-billing\"",
        "\"ui-gallery-alert-link-support\"",
    ] {
        assert!(
            interactive_links.contains(needle),
            "interactive-link follow-up should keep the Fret-only link logic explicit; missing `{needle}`",
        );
    }

    assert!(
        !demo.contains("interactive_link_text("),
        "alert demo snippet should not keep the Fret-only link helper inline anymore",
    );
    assert!(
        !demo.contains("Effect::OpenUrl"),
        "alert demo snippet should not own external-link side effects",
    );

    for needle in [
        "shadcn::AlertTitle::build(|cx, out| {",
        "ui-gallery-alert-rich-title",
    ] {
        assert!(
            rich_title.contains(needle),
            "rich-title follow-up should keep the composed title lane visible; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::AlertDescription::build(|cx, out| {",
        "ui-gallery-alert-rich-description-card",
    ] {
        assert!(
            rich_description.contains(needle),
            "rich-description follow-up should keep the composed description lane visible; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-alert-demo-success\"",
        "\"ui-gallery-alert-demo-title-only\"",
        "\"ui-gallery-alert-demo-payment-error\"",
        "\"ui-gallery-alert-api-reference-content\"",
        "\"ui-gallery-alert-rich-title-content\"",
        "\"ui-gallery-alert-docs-smoke\"",
    ] {
        assert!(
            docs_smoke.contains(needle),
            "alert docs smoke script should cover the docs-path demo rows and later follow-ups; missing `{needle}`",
        );
    }
}

#[test]
fn alert_action_snippet_stays_on_the_upstream_with_actions_surface() {
    let action = include_str!("../src/ui/snippets/alert/action.rs");
    let page = include_str!("../src/ui/pages/alert.rs");
    let action_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-action-text-non-overlap.json"
    );

    for needle in [
        "shadcn::Button::new(\"Undo\")",
        ".size(shadcn::ButtonSize::Xs)",
        ".test_id(\"ui-gallery-alert-action-undo\")",
        "shadcn::Badge::new(\"Badge\")",
        ".variant(shadcn::BadgeVariant::Secondary)",
        ".test_id(\"ui-gallery-alert-action-badge-chip\")",
    ] {
        assert!(
            normalize_ws(action).contains(&normalize_ws(needle)),
            "alert action snippet should stay aligned to the upstream With Actions example; missing `{needle}`",
        );
    }

    for needle in [".shadow_none()", "ui-gallery-alert-action-enable"] {
        assert!(
            !action.contains(needle),
            "alert action snippet should not keep old Fret-specific action drift `{needle}`",
        );
    }

    assert!(
        page.contains(
            "Current docs-aligned `Action` section tracks the upstream two-row `With Actions` example (`Undo` button row + `Badge` row) instead of a Fret-specific variant."
        ),
        "alert page notes should record the current action-surface alignment choice",
    );

    for needle in [
        "\"ui-gallery-alert-action-undo\"",
        "\"ui-gallery-alert-action-badge-chip\"",
        "\"ui-gallery-alert-action-text-non-overlap\"",
    ] {
        assert!(
            action_script.contains(needle),
            "alert action diag script should keep the aligned action anchors; missing `{needle}`",
        );
    }
}
