#[test]
fn label_page_keeps_docs_order_before_fret_followups() {
    let source = include_str!("../src/ui/pages/label.rs");

    for needle in [
        "Preview mirrors the current shadcn Label docs path first: Demo and Usage. Label in Field, RTL, Composable Content, and API Reference are Fret follow-ups.",
        "`Label::new(text)` remains the copyable docs-path surface; the lead demo now mirrors the official checkbox preview, while `for_control(...)` covers the documented association path.",
        "`Label::children(...)` is the Fret-specific inline composition lane for shadcn's generic child slot: it prepends inline content while preserving the label text as the accessible name and control-facing association label.",
        "`Label::for_control(...)` plus a control-side `control_id(...)` is the Fret bridge for the upstream `htmlFor` / `id` pairing and keeps click-to-focus behavior out of page code.",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Label in Field\", label_in_field)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"Composable Content\", children)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
    ] {
        assert!(
            source.contains(needle),
            "label page should document the docs-path order and Fret follow-ups; missing `{needle}`",
        );
    }

    let render_order = source
        .find("vec![demo, usage, label_in_field, rtl, children, api_reference]")
        .expect("label page should render sections in the documented order");
    let docs_path_note = source
        .find("Preview mirrors the current shadcn Label docs path first")
        .expect("label page should label the docs path before follow-ups");
    assert!(
        docs_path_note < render_order,
        "label page should explain the docs path before rendering the ordered sections",
    );
}

#[test]
fn label_snippets_keep_curated_facade_and_association_surface() {
    let demo = include_str!("../src/ui/snippets/label/demo.rs");
    let usage = include_str!("../src/ui/snippets/label/usage.rs");
    let label_in_field = include_str!("../src/ui/snippets/label/label_in_field.rs");
    let rtl = include_str!("../src/ui/snippets/label/rtl.rs");
    let children = include_str!("../src/ui/snippets/label/children.rs");

    for needle in [
        "use fret::{AppComponentCx, UiChild};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "shadcn::Checkbox::new(checked)",
        ".control_id(id.clone())",
        "shadcn::Label::new(\"Accept terms and conditions\")",
        ".for_control(id.clone())",
        ".test_id(\"ui-gallery-label-demo-label\")",
        ".test_id(\"ui-gallery-label-demo-checkbox\")",
    ] {
        assert!(
            demo.contains(needle),
            "label Demo snippet should keep the docs-path label/control association; missing `{needle}`",
        );
    }

    for needle in [
        "let control_id = ControlId::from(\"ui-gallery-label-usage\")",
        "shadcn::Label::new(\"Your email address\")",
        ".for_control(control_id.clone())",
        "shadcn::Input::new(email)",
        ".control_id(control_id)",
        ".test_id(\"ui-gallery-label-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "label Usage snippet should remain a copyable facade example; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::FieldDescription::new(",
        "For forms, prefer Field + FieldLabel for built-in description/error structure.",
        "shadcn::Field::new([",
        "shadcn::FieldLabel::new(\"Work email\")",
        ".for_control(control_id)",
        "shadcn::Input::new(email)",
        ".control_id(control_id)",
        "shadcn::FieldDescription::new(\"We use this email for notifications.\")",
        ".test_id(\"ui-gallery-label-field\")",
    ] {
        assert!(
            label_in_field.contains(needle),
            "label Field follow-up should use the curated Field surface; missing `{needle}`",
        );
    }
    assert!(
        !label_in_field.contains("shadcn::raw::"),
        "label Field follow-up should not expose the raw shadcn namespace in copyable docs",
    );

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl",
        "shadcn::Label::new(\"الاسم الكامل\")",
        ".for_control(control_id)",
        "shadcn::Input::new(name)",
        ".control_id(control_id)",
        ".test_id(\"ui-gallery-label-rtl\")",
    ] {
        assert!(
            rtl.contains(needle),
            "label RTL follow-up should keep the association path under direction context; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::Checkbox::new(checked)",
        "shadcn::Label::new(\"Email me product updates\")",
        ".for_control(control_id.clone())",
        ".children([icon::icon(cx, IconId::new_static(\"lucide.sparkles\"))])",
        ".test_id(\"ui-gallery-label-children-label\")",
        ".test_id(\"ui-gallery-label-children-checkbox\")",
    ] {
        assert!(
            children.contains(needle),
            "label Composable Content follow-up should keep the narrow children lane and association; missing `{needle}`",
        );
    }
}

#[test]
fn label_diag_scripts_cover_docs_and_association_followups() {
    let docs = include_str!(
        "../../../tools/diag-scripts/ui-gallery/label/ui-gallery-label-docs-smoke.json"
    );
    let demo_click = include_str!(
        "../../../tools/diag-scripts/ui-gallery/label/ui-gallery-label-click-label-toggles-checkbox.json"
    );
    let children_click = include_str!(
        "../../../tools/diag-scripts/ui-gallery/label/ui-gallery-label-children-click-label-toggles-checkbox.json"
    );
    let focus_stub = include_str!(
        "../../../tools/diag-scripts/ui-gallery/label/ui-gallery-label-click-focus-input.json"
    );
    let legacy_demo_stub = include_str!(
        "../../../tools/diag-scripts/ui-gallery/label/ui-gallery-label-demo-click-label-focuses-input.json"
    );

    for needle in [
        "\"ui-gallery-label-docs-smoke\"",
        "\"type_text\", \"text\": \"label\"",
        "\"ui-gallery-nav-label\"",
        "\"ui-gallery-page-label\"",
        "\"ui-gallery-label-demo\"",
    ] {
        assert!(
            docs.contains(needle),
            "label docs smoke script should keep the docs page reachable; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-label-click-label-toggles-checkbox\"",
        "\"ui-gallery-label-demo-label\"",
        "\"ui-gallery-label-demo-checkbox\"",
        "\"checked_is\"",
        "\"checked\": true",
        "\"checked\": false",
    ] {
        assert!(
            demo_click.contains(needle),
            "label demo click script should gate label-to-control activation; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-label-children-click-label-toggles-checkbox\"",
        "\"ui-gallery-label-children-label\"",
        "\"ui-gallery-label-children-checkbox\"",
        "\"checked_is\"",
    ] {
        assert!(
            children_click.contains(needle),
            "label children click script should gate the composable label association; missing `{needle}`",
        );
    }

    let canonical = "\"to\": \"tools/diag-scripts/ui-gallery/label/ui-gallery-label-click-label-toggles-checkbox.json\"";
    assert!(
        focus_stub.contains(canonical),
        "label click-focus redirect should point at the canonical label click script",
    );
    assert!(
        legacy_demo_stub.contains(canonical),
        "legacy label demo redirect should point at the canonical label click script",
    );
}
