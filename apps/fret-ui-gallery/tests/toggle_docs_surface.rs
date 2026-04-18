fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn toggle_page_documents_docs_path_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/toggle.rs");

    for needle in [
        "`toggle_uncontrolled(cx, false, |cx| ..)` and `toggle(cx, model, |cx| ..)` are the default first-party entry points; `variant(...)`, `size(...)`, `disabled(...)`, and `a11y_label(...)` cover the documented control surface.",
        "`toggle_uncontrolled(cx, ..)` and `toggle(cx, ..)` already provide the composable-children lane for source-shaped examples, so `Toggle::children([...])` stays the landed-content follow-up instead of widening the root API further.",
        "Pressed semantics, keyboard activation, and focus-visible treatment are already covered by the existing toggle semantics/chrome gates; the remaining parity work here is docs/public-surface alignment rather than a `fret-ui` mechanism gap.",
        "No extra generic `asChild` / `compose()` surface is needed here: `children([...])` already covers the composable content story without widening the primitive contract.",
        "Preview mirrors the shadcn Toggle docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Outline`, `With Text`, `Size`, `Disabled`, and `RTL`. `Children (Fret)`, `Label Association`, and `API Reference` stay as explicit Fret follow-ups.",
        "DocSection::build(cx, \"Children (Fret)\", children)",
        "DocSection::build(cx, \"Label Association\", label)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
    ] {
        assert!(
            source.contains(needle),
            "toggle page should document the docs-path order and children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            outline,
            with_text,
            size,
            disabled,
            rtl,
            children,
            label,
            api_reference,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "toggle page should keep the docs-path sections before the Fret follow-ups and API summary",
    );
}

#[test]
fn toggle_snippets_stay_copyable_and_docs_aligned() {
    let usage = include_str!("../src/ui/snippets/toggle/usage.rs");
    let demo = include_str!("../src/ui/snippets/toggle/demo.rs");
    let children = include_str!("../src/ui/snippets/toggle/children.rs");
    let rtl = include_str!("../src/ui/snippets/toggle/rtl.rs");

    for needle in [
        "use fret::{UiChild, AppComponentCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "shadcn::toggle_uncontrolled(cx, false, |cx| ui::children![cx; ui::text(\"Toggle\")])",
        ".a11y_label(\"Toggle formatting\")",
        ".test_id(\"ui-gallery-toggle-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "toggle usage snippet should remain a complete copyable docs-path example; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::toggle_uncontrolled(cx, false, |cx| {",
        "IconId::new_static(\"lucide.bookmark\")",
        ".variant(shadcn::ToggleVariant::Outline)",
        ".size(shadcn::ToggleSize::Sm)",
        ".a11y_label(\"Toggle bookmark\")",
        ".test_id(\"ui-gallery-toggle-demo\")",
    ] {
        assert!(
            demo.contains(needle),
            "toggle demo snippet should stay aligned with the upstream top preview while remaining copyable; missing `{needle}`",
        );
    }

    for needle in [
        "let bookmark_children = vec![",
        "let underline_children = vec![",
        ".children(bookmark_children)",
        ".children(underline_children)",
        ".test_id(\"ui-gallery-toggle-children\")",
    ] {
        assert!(
            children.contains(needle),
            "toggle children snippet should remain the focused landed-content follow-up; missing `{needle}`",
        );
    }

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl, |cx| {",
        "ui::text(\"إشارة مرجعية\")",
        ".variant(shadcn::ToggleVariant::Outline)",
        ".size(shadcn::ToggleSize::Sm)",
        ".a11y_label(\"Toggle bookmark\")",
        ".test_id(\"ui-gallery-toggle-rtl\")",
    ] {
        assert!(
            rtl.contains(needle),
            "toggle RTL snippet should keep a readable translated label while staying copyable; missing `{needle}`",
        );
    }
}

#[test]
fn toggle_docs_diag_script_covers_docs_path_and_fret_follow_ups() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/toggle/ui-gallery-toggle-docs-smoke.json"
    );

    for needle in [
        "\"ui-gallery-toggle-demo-content\"",
        "\"ui-gallery-toggle-usage-content\"",
        "\"ui-gallery-toggle-outline-content\"",
        "\"ui-gallery-toggle-with-text-content\"",
        "\"ui-gallery-toggle-size-content\"",
        "\"ui-gallery-toggle-disabled-content\"",
        "\"ui-gallery-toggle-rtl-content\"",
        "\"ui-gallery-toggle-children-content\"",
        "\"ui-gallery-toggle-label-content\"",
        "\"ui-gallery-toggle-api-reference-title\"",
        "\"ui-gallery-toggle-api-reference-content\"",
        "\"ui-gallery-toggle-docs-smoke\"",
    ] {
        assert!(
            script.contains(needle),
            "toggle docs diag script should cover the docs path and the Fret follow-ups; missing `{needle}`",
        );
    }
}
